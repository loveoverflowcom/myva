"""Synthetic unit fixtures exercise the contract; they are never benchmark evidence."""
from copy import deepcopy
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location("performance_report", ROOT / "scripts/report-asset-performance.py")
reporter = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(reporter)
CONFIG = json.loads((ROOT / "benchmarks/asset-performance.json").read_text())


def unit_fixture():
    """All numbers below are artificial and only test report arithmetic/validation."""
    scene = {"preset": "baseline", "tier": "High", "seed": 7,
             "width_px": 1280, "height_px": 720, "dpr": 1.0,
             **CONFIG["presets"]["baseline"], **CONFIG["tiers"]["High"]}
    return {
        "schema_version": 1,
        "evidence_kind": "unit_fixture",
        "metadata": {
            "captured_at": "2026-10-11T00:00:00Z", "commit": "0" * 40,
            "dirty": False, "profile": "unit_fixture", "seed": 7,
            "environment": "browser",
            "hardware": {key: "Synthetic unit fixture" for key in ("device", "os", "cpu", "ram", "gpu")},
            "renderer": {"class": "software", "backend": "unit_fixture", "adapter": "Synthetic software fixture"},
            "versions": {"bevy": "0.20.0", "rust": "1.97.1", "browser": "unit_fixture"},
            "acquisition_method": "Synthetic values in a unit test; no measurement occurred.",
            "commands": ["python3 -m unittest discover -s tests -p 'test_asset_performance_report.py'"],
            "config": scene,
        },
        "pipeline": {"status": "BLOCKED", "reason": "Synthetic unit fixture; no asset pipeline ran."},
        "metrics": {},
    }


def frame_fixture(source="raf", contiguous=True):
    result = {"status": "MEASURED", "method": "Synthetic unit fixture intervals.",
              "samples": [10, 20, 30, 40], "source": source,
              "contiguous": contiguous, "window_ms": 100}
    if source == "presented":
        result["presentation_verified"] = True
    return result


def start_fixture(cache):
    return {"status": "MEASURED", "method": "Synthetic unit fixture only.", "samples": [10, 20, 30],
            "cache_state": cache, "cache_method": "Synthetic reset/warm operation; never executed.",
            "endpoint": "click_play_to_first_controllable", "included_phases": ["fetch", "decode", "gpu_upload"]}


def wire_fixture():
    return {"status": "MEASURED", "method": "Synthetic response records; no network ran.",
            "value": 150, "acquisition": "resource_timing_transfer_size", "cache_state": "cold",
            "scope": "runtime_and_initial_region", "responses": [
                {"kind": "runtime", "url": "/unit-fixture/runtime.wasm", "transfer_bytes": 100, "content_encoding": "gzip"},
                {"kind": "initial_region", "url": "/unit-fixture/region.png", "transfer_bytes": 50, "content_encoding": "identity"},
            ]}


class PerformanceContractTests(unittest.TestCase):
    def setUp(self):
        self.capture = unit_fixture()

    def report(self):
        return reporter.report_capture(self.capture, CONFIG)

    def test_nearest_rank_and_small_tail(self):
        values = list(range(1, 101))
        self.assertEqual([reporter.nearest_rank(values, p) for p in (50, 95, 99)], [50, 95, 99])
        self.assertEqual(reporter.nearest_rank([10, 20, 30], 95), 30)
        with self.assertRaises(reporter.CaptureError):
            reporter.nearest_rank([], 95)

    def test_absent_metrics_stay_not_run_not_zero(self):
        output = self.report()
        self.assertEqual(set(output["metrics"]), reporter.METRICS)
        for metric in output["metrics"].values():
            self.assertEqual(metric["status"], "NOT_RUN")
            self.assertNotIn("value", metric)
            self.assertNotIn("mean", metric)
        self.assertEqual(output["art_pipeline"]["decision"], "NO_GO")

    def test_raf_and_update_rates_are_not_fps(self):
        for source, rate in (("raf", "raf_callback_rate_hz"), ("bevy_update", "bevy_update_rate_hz")):
            self.capture["metrics"] = {"frame_time_ms": frame_fixture(source)}
            output = self.report()
            frames = output["metrics"]["frame_time_ms"]
            self.assertEqual(frames[rate], 40)
            self.assertNotIn("presented_fps", frames)
            self.assertEqual(frames["p50"], 20)
            self.assertEqual(frames["p95"], 40)
            self.assertEqual(output["metrics"]["cpu_time_ms"]["status"], "NOT_RUN")
            self.assertEqual(output["metrics"]["gpu_time_ms"]["status"], "NOT_RUN")

    def test_presented_fixture_is_explicitly_synthetic(self):
        self.capture["metrics"]["frame_time_ms"] = frame_fixture("presented")
        output = self.report()
        self.assertEqual(output["metrics"]["frame_time_ms"]["presented_fps"], 40)
        self.assertEqual(output["metrics"]["frame_time_ms"]["status"], "UNIT_FIXTURE")
        self.assertTrue(all(row["status"] not in {"PASS", "WITHIN_ESTIMATE"} for row in output["budget_vs_actual"]))
        self.assertEqual(output["art_pipeline"]["decision"], "NO_GO")

    def test_noncontiguous_samples_do_not_establish_a_rate(self):
        self.capture["metrics"]["frame_time_ms"] = frame_fixture("presented", contiguous=False)
        frames = self.report()["metrics"]["frame_time_ms"]
        self.assertNotIn("presented_fps", frames)
        self.assertIn("NOT_RUN", frames["rate_status"])

    def test_frame_window_must_cover_contiguous_intervals(self):
        self.capture["metrics"]["frame_time_ms"] = frame_fixture()
        self.capture["metrics"]["frame_time_ms"]["window_ms"] = 600000
        with self.assertRaises(reporter.CaptureError):
            self.report()

    def test_presented_frames_require_instrumentation(self):
        self.capture["metrics"]["frame_time_ms"] = frame_fixture("presented")
        del self.capture["metrics"]["frame_time_ms"]["presentation_verified"]
        with self.assertRaises(reporter.CaptureError):
            self.report()

    def test_software_cannot_be_relabelled_as_hardware(self):
        self.capture["metadata"]["renderer"].update({"class": "hardware_gpu", "adapter": "ANGLE SwiftShader"})
        with self.assertRaises(reporter.CaptureError):
            self.report()

    def test_mobile_needs_device_and_hardware(self):
        self.capture["metadata"]["environment"] = "android_device"
        with self.assertRaises(reporter.CaptureError):
            self.report()
        self.capture["metadata"]["hardware"].update({"physical_device": True, "device_id": "synthetic-test-id"})
        with self.assertRaises(reporter.CaptureError):
            self.report()

    def test_required_metadata_rejects_missing_and_abbreviated_values(self):
        for key in ("hardware", "renderer", "versions", "acquisition_method", "commands", "config"):
            capture = unit_fixture()
            del capture["metadata"][key]
            with self.assertRaises(reporter.CaptureError, msg=key):
                reporter.report_capture(capture, CONFIG)
        self.capture["metadata"]["commit"] = "33c77e1"
        with self.assertRaises(reporter.CaptureError):
            self.report()

    def test_scene_fixture_separates_gameplay_from_render_entities(self):
        baseline = CONFIG["presets"]["baseline"]
        self.assertEqual(baseline, {"gameplay_players": 1, "render_only_actors": 19, "render_only_monsters": 30})
        for total in (50, 200, 500):
            preset = CONFIG["presets"][f"stress_{total}"]
            self.assertEqual(preset["gameplay_players"], 1)
            self.assertEqual(preset["gameplay_players"] + preset["render_only_actors"] + preset["render_only_monsters"], total)
        self.assertEqual(CONFIG["presets"]["stress_50"], baseline)
        for tier in CONFIG["tiers"].values():
            self.assertEqual(tier["boss_telegraphs"], "unchanged")
            self.assertEqual(tier["combat_feedback"], "unchanged")
            self.assertIs(tier["hud"], True)

    def test_resolved_scene_cannot_hide_density_or_tier_changes(self):
        self.capture["metadata"]["config"]["render_only_monsters"] = 0
        with self.assertRaises(reporter.CaptureError):
            self.report()
        self.capture = unit_fixture()
        self.capture["metadata"]["config"]["tier"] = "Low"
        with self.assertRaises(reporter.CaptureError):
            self.report()

    def test_missing_metrics_cannot_carry_stale_numbers(self):
        self.capture["metrics"]["rss_bytes"] = {"status": "NOT_RUN", "reason": "No sampler.", "samples": [100]}
        with self.assertRaises(reporter.CaptureError):
            self.report()

    def test_blocked_metric_reason_is_preserved(self):
        metric = {"status": "BLOCKED", "reason": "Device instrumentation unavailable."}
        self.capture["metrics"]["gpu_time_ms"] = metric
        self.assertEqual(self.report()["metrics"]["gpu_time_ms"], metric)

    def test_blocked_frame_budgets_are_not_reported_as_a_measurement(self):
        self.capture["metrics"]["frame_time_ms"] = {"status": "BLOCKED", "reason": "No presentation sampler."}
        rows = self.report()["budget_vs_actual"]
        high = [row for row in rows if row["budget"].startswith("High")]
        self.assertTrue(all(row["status"] == "BLOCKED" and row["actual"] is None for row in high))

    def test_boolean_seed_cannot_masquerade_as_integer_seed(self):
        self.capture["metadata"]["seed"] = 1
        self.capture["metadata"]["config"]["seed"] = True
        with self.assertRaises(reporter.CaptureError):
            self.report()

    def test_invalid_and_empty_samples_are_rejected(self):
        for samples in ([], [float("nan")], [float("inf")], [-1], [True], [0]):
            self.capture["metrics"]["frame_time_ms"] = {**frame_fixture(), "samples": samples}
            with self.assertRaises(reporter.CaptureError, msg=str(samples)):
                self.report()

    def test_metric_method_and_units_are_required(self):
        self.capture["metrics"]["cpu_time_ms"] = {"status": "MEASURED", "samples": [1]}
        with self.assertRaises(reporter.CaptureError):
            self.report()
        self.capture["metrics"] = {"rss_bytes": {"status": "MEASURED", "method": "Synthetic fixture.", "samples": [1.5]}}
        with self.assertRaises(reporter.CaptureError):
            self.report()

    def test_gzip_and_texture_estimates_do_not_become_wire_or_total_gpu(self):
        self.capture["metrics"] = {
            "bundle_gzip_bytes": {"status": "ESTIMATED", "method": "Synthetic fixture.", "value": 100},
            "estimated_texture_bytes": {"status": "ESTIMATED", "method": "Synthetic fixture.", "value": 100},
        }
        output = self.report()
        self.assertEqual(output["metrics"]["actual_transfer_bytes"]["status"], "NOT_RUN")
        self.assertEqual(output["metrics"]["p1_compressed_bytes"]["status"], "NOT_RUN")
        gpu = next(row for row in output["budget_vs_actual"] if row["budget"].startswith("GPU"))
        self.assertIsNone(gpu["actual"])
        self.capture["metrics"]["actual_transfer_bytes"] = {"status": "ESTIMATED", "method": "Synthetic fixture.", "value": 100}
        with self.assertRaises(reporter.CaptureError):
            self.report()

    def test_input_endpoint_and_p1_scope_cannot_be_implicit(self):
        self.capture["metrics"]["input_latency_ms"] = {"status": "MEASURED", "method": "Synthetic fixture.", "samples": [12]}
        with self.assertRaises(reporter.CaptureError):
            self.report()
        self.capture["metrics"] = {"p1_compressed_bytes": {"status": "MEASURED", "method": "Synthetic fixture.", "value": 100}}
        with self.assertRaises(reporter.CaptureError):
            self.report()

    def test_unknown_metric_cannot_silently_hide_a_typo(self):
        self.capture["metrics"]["gpu_frames_ms"] = {"status": "NOT_RUN", "reason": "Synthetic typo."}
        with self.assertRaises(reporter.CaptureError):
            self.report()

    def test_start_metrics_need_cache_endpoint_and_covered_phases(self):
        for cache in ("cold", "warm"):
            name = f"{cache}_start_ms"
            for key in ("cache_state", "cache_method", "endpoint", "included_phases"):
                metric = start_fixture(cache)
                del metric[key]
                self.capture["metrics"] = {name: metric}
                with self.assertRaises(reporter.CaptureError, msg=f"{name}/{key}"):
                    self.report()
            self.capture["metrics"] = {name: start_fixture(cache)}
            self.assertEqual(self.report()["metrics"][name]["p95"], 30)

    def test_start_cache_and_endpoint_cannot_be_relabelled(self):
        for change in ({"cache_state": "warm"}, {"endpoint": "module_init_to_ready"}, {"included_phases": ["fetch", "decode"]}):
            self.capture["metrics"] = {"cold_start_ms": {**start_fixture("cold"), **change}}
            with self.assertRaises(reporter.CaptureError):
                self.report()

    def test_all_stale_fields_are_rejected_when_unavailable(self):
        for status in ("NOT_RUN", "BLOCKED"):
            for key in ("value", "samples", "p50", "p95", "p99", "maximum", "sample_count", "presented_fps", "method", "unknown"):
                self.capture["metrics"] = {"frame_time_ms": {"status": status, "reason": "Synthetic unavailable fixture.", key: 1}}
                with self.assertRaises(reporter.CaptureError, msg=f"{status}/{key}"):
                    self.report()

    def test_available_inputs_cannot_inject_derived_output_fields(self):
        derived = ("presented_fps", "raf_callback_rate_hz", "bevy_update_rate_hz", "sample_count", "minimum", "maximum", "mean", "p50", "p95", "p99", "rate_status", "interval_source")
        for source, contiguous in (("raf", True), ("presented", False)):
            for key in derived:
                self.capture["metrics"] = {"frame_time_ms": {**frame_fixture(source, contiguous), key: 999}}
                with self.assertRaises(reporter.CaptureError, msg=f"{source}/{key}"):
                    self.report()
        self.capture["metrics"] = {"bundle_raw_bytes": {"status": "MEASURED", "method": "Synthetic unit fixture.", "value": 10, "p95": 999}}
        with self.assertRaises(reporter.CaptureError):
            self.report()

    def test_encoded_ram_bytes_do_not_imply_decoded_or_gpu_memory(self):
        self.capture["metrics"] = {"encoded_asset_bytes": {"status": "MEASURED", "method": "Synthetic RAM fixture.", "samples": [4, 8, 2]}}
        metrics = self.report()["metrics"]
        self.assertEqual(metrics["encoded_asset_bytes"]["maximum"], 8)
        for name in ("decoded_image_bytes", "staging_bytes", "estimated_texture_bytes", "wasm_memory_bytes"):
            self.assertEqual(metrics[name]["status"], "NOT_RUN")

    def test_rss_requires_scope_and_game_only_does_not_qualify(self):
        metric = {"status": "MEASURED", "method": "Synthetic RSS fixture.", "samples": [100]}
        self.capture["metrics"] = {"rss_bytes": metric}
        with self.assertRaises(reporter.CaptureError):
            self.report()
        for scope in ("game_only", "game_and_cmp", "browser_process"):
            self.capture["metrics"]["rss_bytes"] = {**metric, "scope": scope}
            self.assertEqual(self.report()["metrics"]["rss_bytes"]["maximum"], 100)
        self.assertIsNotNone(reporter.native_rss_budget_reason("native_desktop", "game_only"))
        self.assertIsNone(reporter.native_rss_budget_reason("native_desktop", "game_and_cmp"))
        self.assertIsNotNone(reporter.native_rss_budget_reason("browser", "game_and_cmp"))

    def test_p1_needs_matching_cold_observed_wire(self):
        p1 = {"status": "MEASURED", "method": "Synthetic P1 unit fixture.", "scope": "runtime_and_initial_region", "value": 150}
        wire = wire_fixture()
        self.capture["metrics"] = {"p1_compressed_bytes": p1, "actual_transfer_bytes": wire}
        output = self.report()
        self.assertEqual(output["metrics"]["actual_transfer_bytes"]["status"], "UNIT_FIXTURE")
        self.assertTrue(reporter.observed_p1_wire(p1, wire))
        for change in ({"cache_state": "warm"}, {"scope": "all_responses"}, {"value": 151}, {"status": "ESTIMATED"}):
            self.assertFalse(reporter.observed_p1_wire(p1, {**wire, **change}))
        self.assertFalse(reporter.observed_p1_wire(p1, {"status": "NOT_RUN"}))
        self.assertFalse(reporter.observed_p1_wire({**p1, "status": "ESTIMATED"}, wire))
        del self.capture["metrics"]["actual_transfer_bytes"]
        p1_budget = next(row for row in self.report()["budget_vs_actual"] if row["budget"].startswith("Compressed P1"))
        self.assertEqual(p1_budget["status"], "NOT_RUN")
        self.assertIn("observed cold-wire", p1_budget["reason"])

    def test_wire_requires_acquisition_encoding_coverage_and_exact_sum(self):
        for key in ("acquisition", "scope", "responses", "cache_state"):
            wire = wire_fixture()
            del wire[key]
            self.capture["metrics"] = {"actual_transfer_bytes": wire}
            with self.assertRaises(reporter.CaptureError, msg=key):
                self.report()
        mutations = [
            lambda wire: wire.update(value=151),
            lambda wire: wire["responses"][0].pop("content_encoding"),
            lambda wire: wire["responses"][0].update(content_encoding="unknown"),
            lambda wire: wire["responses"][1].update(kind="runtime"),
        ]
        for mutate in mutations:
            wire = wire_fixture()
            mutate(wire)
            self.capture["metrics"] = {"actual_transfer_bytes": wire}
            with self.assertRaises(reporter.CaptureError):
                self.report()

    def test_estimated_p1_is_descriptive_and_zero_cache_transfer_cannot_qualify(self):
        p1 = {"status": "ESTIMATED", "method": "Synthetic gzip unit fixture.", "scope": "runtime_and_initial_region", "value": 150}
        self.capture["metrics"] = {"p1_compressed_bytes": p1}
        self.assertEqual(self.report()["metrics"]["p1_compressed_bytes"]["status"], "UNIT_FIXTURE")
        wire = wire_fixture()
        wire["value"] = 0
        for response in wire["responses"]:
            response["transfer_bytes"] = 0
        self.assertFalse(reporter.observed_p1_wire({**p1, "status": "MEASURED", "value": 0}, wire))

    def test_numeric_overflow_is_rejected_or_safely_summarized(self):
        self.capture["metrics"] = {"cpu_time_ms": {"status": "MEASURED", "method": "Synthetic extreme unit fixture.", "samples": [10 ** 1000]}}
        with self.assertRaises(reporter.CaptureError):
            self.report()
        self.capture["metrics"]["cpu_time_ms"]["samples"] = [1e308, 1e308]
        output = self.report()
        self.assertEqual(output["metrics"]["cpu_time_ms"]["mean"], 1e308)
        json.dumps(output, allow_nan=False)
        self.capture["metrics"] = {"frame_time_ms": {**frame_fixture(), "samples": [1e308, 1e308], "window_ms": 1e308}}
        with self.assertRaises(reporter.CaptureError):
            self.report()

    def test_instantaneous_presented_fps_percentiles_are_distinct(self):
        self.capture["metrics"] = {"frame_time_ms": frame_fixture("presented")}
        frames = self.report()["metrics"]["frame_time_ms"]
        self.assertEqual(frames["presented_fps"], 40)
        self.assertAlmostEqual(frames["instantaneous_presented_fps_p50"], 1000 / 30)
        self.assertEqual(frames["instantaneous_presented_fps_p95"], 100)
        self.assertEqual(frames["instantaneous_presented_fps_p99"], 100)
        self.capture["metrics"]["frame_time_ms"] = frame_fixture("raf")
        self.assertNotIn("instantaneous_presented_fps_p50", self.report()["metrics"]["frame_time_ms"])

    def test_pipeline_and_unit_fixtures_cannot_authorize_art(self):
        config = deepcopy(CONFIG)
        config["pipeline_prerequisite"] = {"status": "PASS", "reason": "Synthetic prerequisite only."}
        self.capture["pipeline"] = {"status": "PASS", "reason": "Synthetic prerequisite only."}
        self.assertEqual(reporter.report_capture(self.capture, config)["art_pipeline"]["decision"], "NO_GO")

    def test_cli_writes_labelled_fixture_json_and_markdown(self):
        with tempfile.TemporaryDirectory(prefix="myva-unit-fixture-") as folder:
            root = Path(folder)
            capture, output_json, output_markdown = (root / name for name in ("unit-fixture.json", "unit-fixture-report.json", "unit-fixture-report.md"))
            capture.write_text(json.dumps(self.capture))
            self.assertEqual(reporter.main([str(capture), "--output-json", str(output_json), "--output-markdown", str(output_markdown)]), 0)
            output = json.loads(output_json.read_text())
            self.assertEqual(output["evidence_kind"], "unit_fixture")
            self.assertIn("NO_GO", output_markdown.read_text())
            self.assertIn("Synthetic unit fixture", output_markdown.read_text())
            self.assertIn(self.capture["metadata"]["commands"][0], output_markdown.read_text())


if __name__ == "__main__":
    unittest.main()
