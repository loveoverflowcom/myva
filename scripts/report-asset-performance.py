#!/usr/bin/env python3
"""Validate D05-B captures and report only explicitly acquired measurements.

No acquisition or benchmark is performed by this stdlib-only program. A capture
contains schema_version, evidence_kind (benchmark or unit_fixture), metadata,
pipeline and metrics. Run --help for required metadata and metric formats.
"""
import argparse
from datetime import datetime
import json
import math
from pathlib import Path
import re
import statistics
import sys


STATUS = {"MEASURED", "ESTIMATED", "NOT_RUN", "BLOCKED"}
ENVIRONMENTS = {"browser", "native_desktop", "android_device", "ios_device"}
SAMPLE_METRICS = {
    "frame_time_ms", "cpu_time_ms", "gpu_time_ms", "gpu_upload_ms",
    "rss_bytes", "wasm_memory_bytes", "encoded_asset_bytes", "decoded_image_bytes", "staging_bytes",
    "cold_start_ms", "warm_start_ms", "load_latency_ms", "unload_latency_ms",
    "input_latency_ms", "live_handles", "live_bundles",
}
SCALAR_METRICS = {
    "estimated_texture_bytes", "estimated_buffer_bytes", "estimated_render_target_bytes",
    "bundle_raw_bytes", "bundle_gzip_bytes", "actual_transfer_bytes", "p1_compressed_bytes",
}
ESTIMATE_METRICS = {
    "estimated_texture_bytes", "estimated_buffer_bytes", "estimated_render_target_bytes",
    "bundle_gzip_bytes",
}
METRICS = SAMPLE_METRICS | SCALAR_METRICS
EXTRA_INPUT_KEYS = {
    "frame_time_ms": {"source", "contiguous", "window_ms", "presentation_verified"},
    "input_latency_ms": {"endpoint"},
    "cold_start_ms": {"cache_state", "cache_method", "endpoint", "included_phases"},
    "warm_start_ms": {"cache_state", "cache_method", "endpoint", "included_phases"},
    "rss_bytes": {"scope"},
    "p1_compressed_bytes": {"scope"},
    "actual_transfer_bytes": {"acquisition", "scope", "responses", "cache_state"},
}


class CaptureError(ValueError):
    """The capture does not support the claims it asks this tool to report."""


def require(condition, message):
    if not condition:
        raise CaptureError(message)


def string(value):
    return isinstance(value, str) and bool(value.strip())


def numeric(value, *, positive=False, integer=False):
    if not isinstance(value, (int, float)) or isinstance(value, bool):
        return False
    try:
        finite = math.isfinite(value)
    except OverflowError:
        return False
    return (finite and (value > 0 if positive else value >= 0)
            and (not integer or isinstance(value, int)))


def finite_sum(values, label):
    try:
        total = math.fsum(values)
    except (OverflowError, ValueError) as error:
        raise CaptureError(f"{label} exceeds the supported finite numeric range.") from error
    require(numeric(total), f"{label} must have a finite sum.")
    return total


def nearest_rank(values, percentile):
    """Nearest rank: sorted sample at ceil(p/100 * n), using a 1-based rank."""
    require(bool(values), "Percentiles require at least one sample.")
    require(0 < percentile <= 100, "Percentile must be in (0, 100].")
    return sorted(values)[math.ceil(percentile / 100 * len(values)) - 1]


def validate_metadata(meta, config):
    require(isinstance(meta, dict), "metadata must be an object.")
    for key in ("captured_at", "profile", "acquisition_method"):
        require(string(meta.get(key)), f"metadata.{key} is required.")
    try:
        timestamp = datetime.fromisoformat(meta["captured_at"].replace("Z", "+00:00"))
    except ValueError as error:
        raise CaptureError("captured_at must be an ISO-8601 timestamp.") from error
    require(timestamp.tzinfo is not None, "captured_at must include its timezone.")
    require(bool(re.fullmatch(r"[0-9a-f]{40}", str(meta.get("commit", "")))),
            "metadata.commit must be the full 40-character SHA.")
    require(isinstance(meta.get("dirty"), bool), "metadata.dirty must be explicit.")
    require(isinstance(meta.get("seed"), int) and not isinstance(meta["seed"], bool),
            "metadata.seed must be an integer.")
    require(meta.get("environment") in ENVIRONMENTS, "Unknown environment class.")
    hardware, renderer, versions = (meta.get(key) for key in ("hardware", "renderer", "versions"))
    require(isinstance(hardware, dict), "metadata.hardware is required.")
    for key in ("device", "os", "cpu", "ram", "gpu"):
        require(string(hardware.get(key)), f"hardware.{key} is required; state unavailable with a reason.")
    require(isinstance(renderer, dict), "metadata.renderer is required.")
    require(renderer.get("class") in {"hardware_gpu", "software"}, "renderer.class must be hardware_gpu or software.")
    for key in ("backend", "adapter"):
        require(string(renderer.get(key)), f"renderer.{key} is required.")
    software_names = r"swiftshader|llvmpipe|lavapipe|software|softpipe|warp"
    require(not (renderer["class"] == "hardware_gpu" and
                 re.search(software_names, renderer["adapter"], re.I)),
            "A software adapter cannot be classified as hardware_gpu.")
    require(isinstance(versions, dict), "metadata.versions is required.")
    for key in ("bevy", "rust"):
        require(string(versions.get(key)), f"versions.{key} is required.")
    if meta["environment"] == "browser":
        require(string(versions.get("browser")), "versions.browser is required for browser captures.")
    if meta["environment"] in {"android_device", "ios_device"}:
        require(hardware.get("physical_device") is True, "Mobile evidence requires a physical device.")
        require(string(hardware.get("device_id")), "Mobile evidence requires a device identifier.")
        require(renderer["class"] == "hardware_gpu", "Real mobile hardware evidence cannot use a software adapter.")
    commands = meta.get("commands")
    require(isinstance(commands, list) and commands and all(string(item) for item in commands),
            "metadata.commands must contain the exact acquisition commands.")
    scene = meta.get("config")
    require(isinstance(scene, dict), "metadata.config must contain the resolved scene configuration.")
    preset, tier = scene.get("preset"), scene.get("tier")
    require(preset in config["presets"] and tier in config["tiers"], "Unknown scene preset/tier.")
    expected = {**config["presets"][preset], **config["tiers"][tier]}
    for key, value in expected.items():
        require(key in scene and type(scene[key]) is type(value) and scene[key] == value,
                f"Resolved config.{key} must match the selected preset/tier.")
    require(isinstance(scene.get("seed"), int) and not isinstance(scene["seed"], bool)
            and scene["seed"] == meta["seed"], "Resolved scene seed must match metadata.seed.")
    for key in ("width_px", "height_px"):
        require(numeric(scene.get(key), positive=True, integer=True), f"config.{key} is required.")
    require(numeric(scene.get("dpr"), positive=True), "config.dpr is required.")


def validate_metric(name, metric):
    require(isinstance(metric, dict), f"{name} must be an object.")
    require(metric.get("status") in STATUS, f"{name} has an invalid status.")
    status = metric["status"]
    if status in {"NOT_RUN", "BLOCKED"}:
        require(string(metric.get("reason")), f"{name} needs an unavailability reason.")
        require(set(metric) <= {"status", "reason"},
                f"{name} may contain only status/reason when {status}.")
        return
    allowed = {"status", "method", "samples" if name in SAMPLE_METRICS else "value"}
    allowed |= EXTRA_INPUT_KEYS.get(name, set())
    require(not set(metric) - allowed, f"{name} has unsupported input keys: {sorted(set(metric) - allowed)}")
    require(string(metric.get("method")), f"{name} requires its acquisition/estimation method.")
    allowed_status = ({"MEASURED", "ESTIMATED"} if name == "p1_compressed_bytes"
                      else {"ESTIMATED"} if name in ESTIMATE_METRICS else {"MEASURED"})
    require(status in allowed_status, f"{name} requires status in {sorted(allowed_status)}.")
    integer = name.endswith("_bytes") or name in {"live_handles", "live_bundles"}
    if name in SAMPLE_METRICS:
        samples = metric.get("samples")
        require(isinstance(samples, list) and samples, f"{name} requires nonempty samples.")
        require(all(numeric(value, positive=name == "frame_time_ms", integer=integer) for value in samples),
                f"{name} has invalid samples.")
    else:
        require(numeric(metric.get("value"), integer=integer), f"{name} requires a nonnegative value.")
    if name == "frame_time_ms":
        require(metric.get("source") in {"presented", "raf", "bevy_update"}, "Frame source must be explicit.")
        require(isinstance(metric.get("contiguous"), bool), "Frame interval continuity must be explicit.")
        require(numeric(metric.get("window_ms"), positive=True), "Frame sample window_ms is required.")
        if metric["source"] == "presented":
            require(metric.get("presentation_verified") is True, "Presented FPS requires verified presentation instrumentation.")
        if metric["contiguous"]:
            require(math.isclose(finite_sum(samples, name), metric["window_ms"], rel_tol=0.01, abs_tol=1),
                    "Contiguous frame intervals must cover the declared sample window.")
    if name == "input_latency_ms":
        require(metric.get("endpoint") in {"dom_to_tick", "dom_to_observed_state", "hardware_to_photon"},
                "Input latency needs an explicit endpoint; do not imply input-to-photon.")
    if name == "p1_compressed_bytes":
        require(metric.get("scope") == "runtime_and_initial_region", "P1 must include runtime and initial-region assets.")
    if name in {"cold_start_ms", "warm_start_ms"}:
        require(metric.get("cache_state") == ("cold" if name == "cold_start_ms" else "warm"),
                f"{name} requires its matching cache state.")
        require(string(metric.get("cache_method")), f"{name} requires its cache reset/warming method.")
        require(metric.get("endpoint") == "click_play_to_first_controllable",
                f"{name} must span click-play through first-controllable.")
        phases = metric.get("included_phases")
        require(isinstance(phases, list) and all(string(phase) for phase in phases)
                and len(phases) == len(set(phases))
                and {"fetch", "decode", "gpu_upload"} <= set(phases),
                f"{name} must cover fetch/decode/GPU upload, including cache-hit or skipped work.")
    if name == "rss_bytes":
        require(metric.get("scope") in {"game_and_cmp", "game_only", "browser_process"},
                "RSS requires an explicit process scope.")
    if name == "actual_transfer_bytes":
        require(metric.get("acquisition") in {"resource_timing_transfer_size", "network_log", "packet_capture"},
                "Wire bytes require a documented observed acquisition.")
        require(metric.get("scope") in {"runtime_and_initial_region", "all_responses", "runtime_only", "initial_region_only"},
                "Wire bytes require explicit response coverage.")
        require(metric.get("cache_state") in {"cold", "warm"}, "Wire bytes require a cache state.")
        responses = metric.get("responses")
        require(isinstance(responses, list) and responses, "Wire bytes require covered response records.")
        for response in responses:
            require(isinstance(response, dict) and set(response) == {"kind", "url", "transfer_bytes", "content_encoding"},
                    "A wire response requires kind/url/transfer_bytes/content_encoding only.")
            require(response["kind"] in {"runtime", "initial_region", "other"} and string(response["url"]),
                    "Wire response category and URL are required.")
            require(numeric(response["transfer_bytes"], integer=True), "Wire response bytes must be nonnegative integers.")
            require(response["content_encoding"] in {"identity", "gzip", "br", "deflate", "zstd"},
                    "Wire content encoding must be documented.")
        require(sum(response["transfer_bytes"] for response in responses) == metric["value"],
                "Covered wire responses must sum to the reported transfer value.")
        if metric["scope"] == "runtime_and_initial_region":
            require({response["kind"] for response in responses} == {"runtime", "initial_region"},
                    "P1 wire scope must cover both runtime and initial-region responses only.")


def summarize(name, metric, fixture):
    if metric["status"] in {"NOT_RUN", "BLOCKED"}:
        return dict(metric)
    result = dict(metric)
    if fixture:
        result["status"] = "UNIT_FIXTURE"
        result["reason"] = "Synthetic unit fixture; not runtime evidence."
    if name in SAMPLE_METRICS:
        samples = result.pop("samples")
        result.update(sample_count=len(samples), minimum=min(samples), maximum=max(samples),
                      mean=statistics.mean(samples),
                      p50=nearest_rank(samples, 50), p95=nearest_rank(samples, 95), p99=nearest_rank(samples, 99))
    if name == "frame_time_ms":
        labels = {"presented": "presented_fps", "raf": "raf_callback_rate_hz", "bevy_update": "bevy_update_rate_hz"}
        result["interval_source"] = result["source"]
        if result["contiguous"]:
            rate = result["sample_count"] * 1000 / result["window_ms"]
            require(numeric(rate), "Observed frame/callback rate must be finite.")
            result[labels[result["source"]]] = rate
        else:
            result["rate_status"] = "NOT_RUN: noncontiguous samples do not establish an observed rate."
        if result["source"] == "presented":
            instantaneous = [1000 / value for value in samples]
            require(all(numeric(value) for value in instantaneous), "Instantaneous presented FPS must be finite.")
            for percentile in (50, 95, 99):
                result[f"instantaneous_presented_fps_p{percentile}"] = nearest_rank(instantaneous, percentile)
    return result


def observed_p1_wire(p1, wire):
    """Only matching cold, observed runtime+region wire bytes qualify for P1."""
    return (p1.get("status") == "MEASURED" and wire.get("status") == "MEASURED"
            and wire.get("scope") == "runtime_and_initial_region"
            and wire.get("cache_state") == "cold" and p1.get("value") == wire.get("value")
            and bool(wire.get("responses"))
            and all(response["transfer_bytes"] > 0 for response in wire["responses"]))


def native_rss_budget_reason(environment, scope):
    if environment == "browser":
        return "Browser process RSS is not the native game/CMP budget."
    if scope != "game_and_cmp":
        return "Game-only or other RSS does not establish the game+CMP budget."
    return None


def report_capture(capture, config):
    require(isinstance(capture, dict) and capture.get("schema_version") == 1, "Unsupported capture schema.")
    require(config.get("schema_version") == 1, "Unsupported scene config schema.")
    require(capture.get("evidence_kind") in {"benchmark", "unit_fixture"}, "evidence_kind must be explicit.")
    validate_metadata(capture.get("metadata"), config)
    pipeline = capture.get("pipeline")
    require(isinstance(pipeline, dict) and pipeline.get("status") in {"PASS", "NOT_RUN", "BLOCKED"}
            and string(pipeline.get("reason")), "pipeline needs an explicit status and explanation.")
    supplied = capture.get("metrics", {})
    require(isinstance(supplied, dict), "metrics must be an object.")
    require(not set(supplied) - METRICS, f"Unknown metrics: {sorted(set(supplied) - METRICS)}")
    fixture = capture["evidence_kind"] == "unit_fixture"
    metrics = {}
    for name in sorted(METRICS):
        metric = supplied.get(name, {"status": "NOT_RUN", "reason": "Not supplied in this capture."})
        validate_metric(name, metric)
        metrics[name] = summarize(name, metric, fixture)
    meta, budgets = capture["metadata"], config["budgets"]
    rows = []

    def budget(name, target, metric_name, field, reason=None):
        metric = metrics[metric_name]
        actual = metric.get(field) if not reason else None
        available = metric["status"] in {"MEASURED", "ESTIMATED"} and actual is not None
        unavailable = "BLOCKED" if metric["status"] == "BLOCKED" else "NOT_RUN"
        status = "PASS" if available and actual <= target else "FAIL" if available else unavailable
        rows.append({"budget": name, "maximum": target, "actual": actual if available else None,
                     "status": status, "reason": reason or metric.get("reason", "Measured only by the stated method.")})

    frames, tier = metrics["frame_time_ms"], meta["config"]["tier"]
    frame_reason = None
    if frames["status"] in {"NOT_RUN", "BLOCKED"}:
        frame_reason = frames["reason"]
    elif frames.get("source") != "presented":
        frame_reason = "RAF/Bevy Update intervals do not establish presented-frame budgets."
    elif not frames.get("contiguous"):
        frame_reason = "A contiguous presentation sample is required for the FPS goal."
    elif frames["window_ms"] < config["run"]["soak_seconds"] * 1000:
        frame_reason = "Sample is shorter than the configured 600-second soak; it does not establish the render goals."
    for level in ("High", "Low"):
        reason = "Capture uses the other quality tier." if level != tier else frame_reason
        target = budgets[level]
        budget(f"{level} presented frame p95 (ms)", target["frame_p95_ms_max"], "frame_time_ms", "p95", reason)
        if "frame_p99_ms_max" in target:
            budget(f"{level} presented frame p99 (ms)", target["frame_p99_ms_max"], "frame_time_ms", "p99", reason)
        fps = frames.get("presented_fps") if not reason else None
        available = frames["status"] == "MEASURED" and fps is not None
        unavailable = "BLOCKED" if frames["status"] == "BLOCKED" else "NOT_RUN"
        rows.append({"budget": f"{level} presented FPS", "minimum": target["presented_fps_min"],
                     "actual": fps if available else None,
                     "status": "PASS" if available and fps >= target["presented_fps_min"] else "FAIL" if available else unavailable,
                     "reason": reason or frames.get("reason", "Rate over contiguous presentation intervals.")})
    budget("WASM memory peak (bytes)", budgets["wasm_memory_bytes_max"], "wasm_memory_bytes", "maximum",
           None if meta["environment"] == "browser" else "Capture is not a WASM browser runtime.")
    rss_reason = native_rss_budget_reason(meta["environment"], metrics["rss_bytes"].get("scope"))
    budget("Native RSS peak (bytes)", budgets["native_rss_bytes_max"], "rss_bytes", "maximum", rss_reason)
    gpu_parts = [metrics[name] for name in ("estimated_texture_bytes", "estimated_buffer_bytes", "estimated_render_target_bytes")]
    gpu_available = all(part["status"] == "ESTIMATED" for part in gpu_parts)
    gpu = sum(part["value"] for part in gpu_parts) if gpu_available else None
    gpu_unavailable = "BLOCKED" if any(part["status"] == "BLOCKED" for part in gpu_parts) else "NOT_RUN"
    rows.append({"budget": "GPU textures/buffers/targets estimate (bytes)", "maximum": budgets["estimated_gpu_bytes_max"],
                 "actual": gpu, "status": "WITHIN_ESTIMATE" if gpu_available and gpu <= budgets["estimated_gpu_bytes_max"] else "FAIL" if gpu_available else gpu_unavailable,
                 "reason": "All three allocation estimates required; this does not measure physical GPU memory release."})
    p1, wire = metrics["p1_compressed_bytes"], metrics["actual_transfer_bytes"]
    budget("Compressed P1 observed cold wire (bytes)", budgets["p1_compressed_bytes_max"], "p1_compressed_bytes", "value",
           None if observed_p1_wire(p1, wire) else "Requires matching observed cold-wire value, content encodings and covered runtime/initial-region responses; file/gzip estimates or cache hits do not qualify.")
    budget("Warm start p95 (ms)", budgets["warm_start_p95_ms_max"], "warm_start_ms", "p95")
    network = meta.get("network", {})
    cold_network = config["run"]["cold_network"]
    network_ok = (isinstance(network, dict) and all(network.get(k) == v for k, v in cold_network.items())
                  and string(network.get("method")))
    budget("Cold start p95 at 15 Mbps/100 ms RTT (ms)", budgets["cold_start_p95_ms_max"], "cold_start_ms", "p95",
           None if network_ok else "Required cold-network profile was not measured.")
    blocker = config.get("pipeline_prerequisite", {})
    pipeline_ready = pipeline["status"] == "PASS" and blocker.get("status") == "PASS"
    limits = list(config.get("limits", [])) + [
        "Nearest-rank percentiles describe this sample only; small samples do not establish tail reliability.",
        "CPU/GPU timings, memory and input endpoints are not inferred from frame intervals.",
        "Gzip estimate is not wire transfer; bundle bytes are not automatically P1 bytes.",
        "Encoded asset bytes in RAM are separate from decoded images, upload staging and GPU estimates.",
        "Instantaneous presented FPS percentiles use 1000/frame_interval; high FPS percentiles describe faster frames, distinct from overall observed FPS and frame-time tails.",
        "This report alone does not verify screenshot inspection, gameplay parity, soak or lifecycle QA."
    ]
    no_go = not pipeline_ready or fixture or any(row["status"] == "FAIL" for row in rows)
    return {"schema_version": 1, "evidence_kind": capture["evidence_kind"], "metadata": meta,
            "config_id": config["id"], "pipeline": pipeline, "configured_pipeline_prerequisite": blocker,
            "metrics": metrics, "budget_vs_actual": rows,
            "art_pipeline": {"decision": "NO_GO" if no_go else "REVIEW_REQUIRED",
                             "reason": "Pipeline missing, evidence synthetic, or a measured budget failed." if no_go else "Measurement report cannot replace all applicable QA and device gates."},
            "limits": limits}


def markdown(report):
    meta = report["metadata"]
    lines = [f"D05-B measurement report: {report['art_pipeline']['decision']}", "",
             f"Evidence: {report['evidence_kind']}; SHA `{meta['commit']}`; profile `{meta['profile']}`; seed {meta['seed']}.",
             f"Environment: {meta['environment']}; renderer {meta['renderer']['class']}: {meta['renderer']['adapter']}.", ""]
    if report["evidence_kind"] == "unit_fixture":
        lines += ["Synthetic unit fixture; no benchmark or runtime measurement occurred.", ""]
    lines += [f"Pipeline: {report['pipeline']['status']}; {report['pipeline']['reason']}", "",
              "Full acquisition metadata:", "", "```json", json.dumps(meta, indent=2), "```", "",
             "| Budget | Target | Actual | Status |", "| --- | --- | --- | --- |"]
    for row in report["budget_vs_actual"]:
        target = f">= {row['minimum']}" if "minimum" in row else f"<= {row['maximum']}"
        actual = "—" if row["actual"] is None else str(row["actual"])
        lines.append(f"| {row['budget']} | {target} | {actual} | {row['status']} |")
    lines += ["", "Metrics (methods and unavailable reasons):", ""]
    for name, metric in report["metrics"].items():
        detail = metric.get("reason", metric.get("method", ""))
        values = {key: metric[key] for key in ("value", "sample_count", "p50", "p95", "p99", "maximum", "presented_fps", "instantaneous_presented_fps_p50", "instantaneous_presented_fps_p95", "instantaneous_presented_fps_p99", "raf_callback_rate_hz", "bevy_update_rate_hz", "endpoint", "cache_state", "cache_method", "included_phases", "scope", "acquisition", "responses") if key in metric}
        lines.append(f"- `{name}`: {metric['status']}; {json.dumps(values, sort_keys=True)}; {detail}")
    lines += ["", "Exact acquisition commands:", "", "```text", *meta["commands"], "```", "", "Limits:", ""]
    lines.extend(f"- {limit}" for limit in report["limits"])
    lines.extend(["", "Budget qualification:", ""])
    lines.extend(f"- {row['budget']}: {row['reason']}" for row in report["budget_vs_actual"])
    return "\n".join(lines) + "\n"


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, epilog=(
        "metadata requires captured_at, commit (40 hex), dirty, profile, seed, environment, hardware "
        "(device/os/cpu/ram/gpu), renderer (class/backend/adapter), versions (bevy/rust/browser), "
        "acquisition_method, exact commands and resolved config (preset/tier/seed/width_px/height_px/dpr "
        "plus every selected preset/tier field). Mobile also needs physical_device=true and device_id. "
        "Each measured metric needs status, method and samples (time/memory/count series) or value "
        "(bundle/transfer/estimates). Missing metrics become NOT_RUN. Frame intervals additionally "
        "need source, contiguous, window_ms and presentation_verified=true when source=presented. "
        "Starts also need matching cache_state, cache_method, endpoint=click_play_to_first_controllable "
        "and included_phases covering fetch/decode/gpu_upload (including cache-hit/skipped work). "
        "RSS needs scope=game_and_cmp/game_only/browser_process. Actual wire needs acquisition "
        "(resource_timing_transfer_size/network_log/packet_capture), cache_state, scope and responses "
        "with kind/url/transfer_bytes/content_encoding. Their sum must match value. P1 budget requires "
        "matching observed cold wire covering runtime plus initial region. Unavailable metrics allow "
        "status/reason only; all caller-supplied derived fields are rejected. "
        "pipeline requires status and reason. Unit fixture reports can never establish a performance gate."))
    parser.add_argument("capture", type=Path)
    parser.add_argument("--config", type=Path, default=Path(__file__).resolve().parents[1] / "benchmarks/asset-performance.json")
    parser.add_argument("--output-json", type=Path)
    parser.add_argument("--output-markdown", type=Path)
    args = parser.parse_args(argv)
    try:
        report = report_capture(json.loads(args.capture.read_text()), json.loads(args.config.read_text()))
    except (OSError, ValueError, KeyError, TypeError, OverflowError) as error:
        parser.exit(2, f"Invalid capture/config: {error}\n")
    encoded = json.dumps(report, indent=2, allow_nan=False) + "\n"
    if args.output_json:
        args.output_json.write_text(encoded)
    else:
        sys.stdout.write(encoded)
    if args.output_markdown:
        args.output_markdown.write_text(markdown(report))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
