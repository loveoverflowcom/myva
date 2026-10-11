import { test, expect } from '@playwright/test';
import fs from 'node:fs/promises';
import os from 'node:os';
import { execFileSync } from 'node:child_process';

// A real Bevy/WebGL2 render workload, using the existing test server and SwiftShader setup.
// Update intervals are explicitly not presented-frame evidence. Physical mobile, GPU timing,
// input-to-photon and driver-memory reclamation remain unmeasured.
const config = JSON.parse(await fs.readFile('benchmarks/asset-performance.json', 'utf8'));
const requestedSeconds = Number(process.env.MYVA_BENCHMARK_SECONDS || config.run.sample_seconds);
if (!Number.isFinite(requestedSeconds) || requestedSeconds <= 0) throw new Error('Invalid MYVA_BENCHMARK_SECONDS');
const cycles = config.run.load_unload_cycles;

async function call(frame, name, args = []) {
    return frame.evaluate(async ({ name, args }) => {
        const game = await import('./pkg/myva_web_game.js');
        return game[name](...args);
    }, { name, args });
}
async function state(frame) { return JSON.parse(await call(frame, 'telemetry')); }
async function runtime(frame) { return JSON.parse(await call(frame, 'stream_runtime')); }
async function benchmark(frame) { return JSON.parse(await call(frame, 'benchmark_telemetry')); }
async function enter(page, artReady = true) {
    await page.goto('/');
    const clickedAt = Date.now();
    await page.locator('#enter-game').click();
    await expect(page.locator('#shell-status')).toContainText(/sẵn sàng|Đánh boss/);
    const frame = page.frames().find(item => item.url().includes('/game/?'));
    expect(frame).toBeTruthy();
    await expect.poll(async () => (await state(frame)).paused).toBe(false);
    if (artReady) await expect.poll(async () => (await state(frame)).art.status).toBe('ready');
    return { frame, clickToArtReadyMs: Date.now() - clickedAt };
}
async function select(frame, population, tier) {
    expect(await call(frame, 'set_benchmark', [population, tier === 'Low', config.seed])).toBe(true);
    await expect.poll(async () => {
        const value = await benchmark(frame);
        return value.ready && value.preset === `stress_${population}` && value.tier === tier;
    }).toBe(true);
    const value = await benchmark(frame);
    expect(value.actual_actors).toBe(config.presets[`stress_${population}`].render_only_actors);
    expect(value.actual_monsters).toBe(config.presets[`stress_${population}`].render_only_monsters);
    expect(value.actual_particles).toBe(config.tiers[tier].cosmetic_particles);
    expect(value.additional_gameplay_rivals).toBe(1);
    expect(value.actual_body_sprites).toBe(population + 1);
    expect(value.applied_render_scale).toBe(config.tiers[tier].render_scale);
    return value;
}
async function writeEvidence(name, value) {
    await fs.mkdir('target/web-evidence', { recursive: true });
    await fs.writeFile(`target/web-evidence/${name}.json`, JSON.stringify(value, null, 2) + '\n');
}
function unavailable(reason) { return { status: 'NOT_RUN', reason }; }
function classifyRenderer(adapter) {
    return /swiftshader|llvmpipe|lavapipe|software/i.test(adapter || '') ? 'software'
        : /nvidia|radeon|amd|intel|apple|qualcomm|adreno|mali|powervr/i.test(adapter || '') ? 'hardware_gpu'
        : null;
}

async function inspect(frame) {
    return frame.evaluate(async () => {
        const game = await import('./pkg/myva_web_game.js');
        const wasm = await game.default();
        const canvas = document.querySelector('#game-canvas');
        const gl = canvas.getContext('webgl2');
        const debug = gl?.getExtension('WEBGL_debug_renderer_info');
        return {
            wasmMemoryBytes: wasm.memory?.buffer.byteLength ?? null,
            renderer: debug ? gl.getParameter(debug.UNMASKED_RENDERER_WEBGL) : null,
            width: canvas.width, height: canvas.height, dpr: devicePixelRatio,
            metrics: window.__myvaMetrics,
        };
    });
}

async function capture(page, browser, frame, initial, population, tier, seconds, name) {
    const build = await (await page.request.get('/bundle-report.json')).json();
    const memory = [];
    const bundles = [];
    const start = Date.now();
    let last;
    // Collect raw engine intervals, including slow frames, rather than averaging 250ms counts.
    // Stop when the actual post-warmup engine sample spans the requested observation window.
    do {
        await page.waitForTimeout(1000);
        last = await benchmark(frame);
        const observed = await inspect(frame);
        if (observed.wasmMemoryBytes !== null) memory.push(observed.wasmMemoryBytes);
        bundles.push((await runtime(frame)).bundles);
        if (last.intervals_truncated || Date.now() - start >= (seconds + 90) * 1000) {
            await writeEvidence(`${name}-incomplete`, {
                evidence_kind: 'raw_runtime_acquisition', status: 'INCOMPLETE',
                reason: last.intervals_truncated ? 'Bounded interval storage truncated' : 'Wall-clock progress deadline exceeded',
                benchmark: last, runtime: await runtime(frame),
            });
            throw new Error('Complete contiguous engine sample unavailable; saved explicit incomplete diagnostic');
        }
    } while (last.window_ms < seconds * 1000);
    expect(last.intervals_truncated).toBe(false);
    await call(frame, 'request_benchmark_capture');
    let raw;
    await expect.poll(async () => {
        raw = await call(frame, 'take_benchmark_capture');
        return raw !== undefined && raw !== null;
    }).toBe(true);
    last = JSON.parse(raw);
    expect(last.interval_samples_included).toBe(true);
    expect(last.frame_intervals_ms).toHaveLength(last.frame_sample_count);
    const observed = await inspect(frame);
    const stream = await runtime(frame);
    const resolved = {
        preset: `stress_${population}`, tier, seed: config.seed,
        width_px: observed.width, height_px: observed.height, dpr: observed.dpr,
        ...config.presets[`stress_${population}`], ...config.tiers[tier],
    };
    const adapter = observed.renderer;
    // Launch flags request SwiftShader, but are not evidence of the active adapter.
    // Only publish a classified benchmark when the actual WebGL renderer is exposed.
    const rendererClass = classifyRenderer(adapter);
    if (rendererClass === null) {
        await writeEvidence(`${name}-incomplete`, {
            schema_version: 1, evidence_kind: 'raw_runtime_acquisition', status: 'BLOCKED',
            reason: 'Active renderer could not be classified from observed WebGL adapter; launch flags are not measurement evidence',
            observed, benchmark: last, runtime: stream,
        });
        throw new Error('Observed renderer classification unavailable; saved explicit blocked diagnostic');
    }
    const artifact = {
        schema_version: 1, evidence_kind: 'benchmark',
        metadata: {
            captured_at: new Date().toISOString(), commit: build.revision, dirty: build.dirty,
            profile: build.profile, seed: config.seed, environment: 'browser',
            hardware: {
                device: `${os.hostname()} cloud desktop container`, os: `${os.type()} ${os.release()} ${os.arch()}`,
                cpu: os.cpus()[0]?.model || 'Unavailable: Node OS CPU model not exposed',
                ram: `${os.totalmem()} host-reported bytes; container limit may differ`, gpu: adapter,
            },
            renderer: { class: rendererClass, backend: 'WebGL2', adapter },
            versions: { bevy: '0.20.0', rust: build.rustc, browser: browser.version() },
            acquisition_method: 'Playwright invokes the existing WASM bridge; engine Time<Real> intervals after 5s warmup, WASM memory buffer byteLength sampled once/sec; real ECS atlas sprites with observed WebGL adapter classification',
            commands: [`MYVA_WEB_TEST_PORT=${process.env.MYVA_WEB_TEST_PORT || '8080'} MYVA_BENCHMARK_SECONDS=${seconds} cargo xtask verify web -- tests/web/runtime-performance.spec.mjs --grep '${name}'`],
            config: resolved,
        },
        pipeline: { status: 'PASS', reason: 'Validated D05-A bundles loaded through the D05-B AssetServer runtime; this is placeholder fixture evidence, not production art or device qualification.' },
        metrics: {
            frame_time_ms: {
                status: 'MEASURED', method: 'Contiguous differences in engine Time<Real>.elapsed_secs_f64, observed in Last after Update; no presentation/GPU instrumentation',
                source: 'bevy_update', contiguous: true, window_ms: last.window_ms,
                samples: last.frame_intervals_ms,
            },
            wasm_memory_bytes: memory.length ? { status: 'MEASURED', method: 'wasm-bindgen initialized export memory.buffer.byteLength once/sec; linear-memory allocation, not browser RSS', samples: memory } : unavailable('WASM memory export unavailable'),
            live_bundles: { status: 'MEASURED', method: 'StreamingPlanner retained loaded-bundle count sampled once/sec; not GPU allocation count', samples: bundles },
            load_latency_ms: stream.load_latency_ms.length ? { status: 'MEASURED', method: stream.load_latency_source, samples: stream.load_latency_ms } : unavailable('No completed bundle latency observed'),
            estimated_texture_bytes: { status: 'ESTIMATED', method: stream.gpu_estimate_method, value: stream.estimated_texture_bytes },
            estimated_buffer_bytes: unavailable('No renderer-buffer allocation instrumentation'),
            estimated_render_target_bytes: unavailable('Only the low-tier offscreen color target is known; Bevy internal color/MSAA/depth/swapchain targets are not fully enumerated'),
            input_latency_ms: observed.metrics.inputSamples.length ? { status: 'MEASURED', method: 'DOM keydown timestamp to RAF observation of the fixed tick consuming the press in the existing runtime observer; no physical input or photon instrumentation', endpoint: 'dom_to_observed_state', samples: observed.metrics.inputSamples } : unavailable('No DOM action press latency sampled in this run'),
            cold_start_ms: unavailable('Observed click-to-art-ready diagnostic excludes proof of GPU upload/first-controllable presentation and no 15Mbps/100ms network profile was applied'),
            warm_start_ms: unavailable('No verified cache-warm click-through first-controllable acquisition including GPU upload'),
            rss_bytes: unavailable('No browser-process/native game+CMP RSS acquisition'),
            gpu_time_ms: unavailable('No GPU timestamp instrumentation'),
            cpu_time_ms: unavailable('Update intervals include scheduling/render waits, not isolated CPU work'),
            gpu_upload_ms: unavailable('Loaded-with-dependencies does not establish completed GPU upload'),
            decoded_image_bytes: unavailable('CPU decode/staging retention not instrumented'),
            encoded_asset_bytes: unavailable('Encoded asset RAM retention not instrumented'),
            staging_bytes: unavailable('Upload staging memory not instrumented'),
            live_handles: unavailable('Owner diagnostics do not enumerate all strong handles or render-world owners'),
        },
    };
    await writeEvidence(name, artifact);
    await writeEvidence(`${name}-diagnostics`, {
        schema_version: 1, evidence_kind: 'raw_runtime_acquisition', benchmark: last, runtime: stream,
        gameplay: await state(frame),
        click_to_art_ready_ms: initial.clickToArtReadyMs,
        startup_endpoint: 'Click enter-game to observation of art Ready and host unpaused; not a verified first-controllable presented frame or GPU-upload endpoint',
        offscreen_rgba8_color_bytes: last.offscreen_rgba8_color_bytes,
        known_texture_and_offscreen_rgba8_bytes: stream.estimated_texture_bytes + last.offscreen_rgba8_color_bytes,
        gpu_estimate_scope: 'Fixture base textures plus actual low-tier offscreen RGBA8 color only; excludes mipmaps/buffers/internal renderer targets/driver memory',
        screenshots_inspected: false, mobile: 'BLOCKED: no physical Android/iOS device',
    });
    execFileSync('python3', ['scripts/report-asset-performance.py', `target/web-evidence/${name}.json`,
        '--output-json', `target/web-evidence/${name}-report.json`,
        '--output-markdown', `target/web-evidence/${name}-report.md`], { encoding: 'utf8' });
    await page.screenshot({ path: `target/web-evidence/${name}.png`, fullPage: true });
}

for (const population of [50, 200, 500]) {
    for (const tier of ['High', 'Low']) {
        const name = `d05b-${population}-${tier}`;
        test(`${name}: real render preset, raw Update/memory capture`, async ({ page, browser }) => {
            test.setTimeout((requestedSeconds + 240) * 1000);
            const initial = await enter(page);
            await select(initial.frame, population, tier);
            // Exercise the existing observed-input path before enabling the repeatable bot.
            await initial.frame.locator('#game-canvas').click();
            await page.keyboard.press('KeyJ');
            await page.waitForTimeout(500);
            await call(initial.frame, 'set_autopilot', [true]);
            await capture(page, browser, initial.frame, initial, population, tier, requestedSeconds, name);
        });
    }
}

test('tiers preserve paused gameplay hash while resizing the actual world target', async ({ page }) => {
    const { frame } = await enter(page);
    await call(frame, 'set_paused', [true]);
    await expect.poll(async () => (await state(frame)).paused).toBe(true);
    const before = await state(frame);
    for (const population of [50, 200, 500]) {
        for (const tier of ['High', 'Low']) {
            const workload = await select(frame, population, tier);
            const observed = await inspect(frame);
            expect(workload.render_width_px).toBe(Math.round(observed.width * config.tiers[tier].render_scale));
            expect(workload.render_height_px).toBe(Math.round(observed.height * config.tiers[tier].render_scale));
            const after = await state(frame);
            expect(after.hash).toBe(before.hash);
            expect(after.tick).toBe(before.tick);
            expect(after.player).toEqual(before.player);
            expect(after.rival).toEqual(before.rival);
        }
    }
    expect(await call(frame, 'set_benchmark', [123, false, 7])).toBe(false);
    expect((await benchmark(frame)).preset).toBe('stress_500');
    await call(frame, 'set_benchmark', [0, false, 7]);
    await expect.poll(async () => (await benchmark(frame)).active).toBe(false);
});

test('30 real bundle load/unload cycles drain owners; rapid transitions keep the latest scene', async ({ page }) => {
    test.setTimeout(600000);
    const { frame } = await enter(page);
    await select(frame, 50, 'High');
    await call(frame, 'set_paused', [true]);
    await expect.poll(async () => (await state(frame)).paused).toBe(true);
    await call(frame, 'set_art_enabled', [false]);
    await expect.poll(async () => {
        const value = await runtime(frame);
        return value.fixture_assets === 0 && value.table_layout_owners === 0 && value.sprite_image_owners === 0 && value.flights === 0;
    }).toBe(true);
    // Let Bevy's asset events/renderer cleanup drain before establishing the store baseline.
    // This census is main-world asset storage, never a driver-memory reclamation assertion.
    await page.waitForTimeout(1000);
    const baseline = await runtime(frame);
    await call(frame, 'set_art_enabled', [true]);
    await expect.poll(async () => (await state(frame)).art.status).toBe('ready');
    const snapshots = [];
    const unloadLatencyMs = [];
    for (let index = 0; index < cycles; index++) {
        await select(frame, 50, index % 2 === 0 ? 'High' : 'Low');
        const started = Date.now();
        await call(frame, 'set_art_enabled', [false]);
        await expect.poll(async () => {
            const value = await runtime(frame);
            return value.bundles === 0 && value.flights === 0 && value.planner_owners === 0
                && value.sprite_image_owners === 0 && value.table_image_owners === 0
                && value.table_layout_owners === 0 && value.fixture_assets === 0
                && value.atlas_assets === baseline.atlas_assets && value.image_assets === baseline.image_assets;
        }).toBe(true);
        unloadLatencyMs.push(Date.now() - started);
        const off = await runtime(frame);
        expect(off.cancelled_draining).toBe(0);
        await call(frame, 'set_art_enabled', [true]);
        await expect.poll(async () => (await state(frame)).art.status).toBe('ready');
        await expect.poll(async () => (await benchmark(frame)).actual_actors).toBe(19);
        const on = await runtime(frame);
        expect(on.bundles).toBeLessThanOrEqual(4);
        expect(on.flights).toBeLessThanOrEqual(2);
        snapshots.push({ cycle: index + 1, off, on });
    }
    // Multiple changes before a frame are intentionally collapsed to the final workload.
    await frame.evaluate(async () => {
        const game = await import('./pkg/myva_web_game.js');
        game.set_benchmark(500, false, 7);
        game.set_benchmark(200, true, 7);
        game.set_benchmark(50, false, 7);
        game.set_art_enabled(false);
        game.set_art_enabled(true);
        game.switch_mode();
        game.switch_mode();
    });
    await expect.poll(async () => (await benchmark(frame)).preset).toBe('stress_50');
    await expect.poll(async () => (await state(frame)).mode).toBe('boss');
    await expect.poll(async () => (await runtime(frame)).flights).toBe(0);
    const observed = await inspect(frame);
    const build = await (await page.request.get('/bundle-report.json')).json();
    await writeEvidence('d05b-lifecycle', {
        captured_at: new Date().toISOString(), revision: build.revision, dirty: build.dirty,
        evidence_kind: 'runtime_lifecycle', environment: 'browser WebGL2',
        renderer: { backend: 'WebGL2', class: classifyRenderer(observed.renderer) || 'unknown', adapter: observed.renderer },
        cycles, baseline, snapshots, unload_latency_ms: unloadLatencyMs,
        unload_endpoint: 'DOM lifecycle command to observed zero planner owners/bundles/flights and main-world sprite/table image owners; does not establish render-world or driver memory reclamation',
        final: await runtime(frame), mobile: 'BLOCKED: no physical Android/iOS device',
    });
});

if (process.env.MYVA_RUN_SOAK === '1') test('600-second runtime soak, explicitly opted in', async ({ page, browser }) => {
    test.setTimeout((config.run.soak_seconds + 240) * 1000);
    const initial = await enter(page);
    await select(initial.frame, 500, 'Low');
    await call(initial.frame, 'set_autopilot', [true]);
    await capture(page, browser, initial.frame, initial, 500, 'Low', config.run.soak_seconds, 'd05b-soak-500-Low');
});
else console.info('NOT_RUN: D05-B 600-second soak is not requested; set MYVA_RUN_SOAK=1 for that observation window.');


test('in-flight cancellation drains real AssetServer flights before latest scene becomes ready', async ({ page }) => {
    let release;
    const gate = new Promise(resolve => { release = resolve; });
    await page.route('**/game/assets/fixture/long_luu/atlas.png', async route => {
        await gate;
        const response = await route.fetch();
        await route.fulfill({ response });
    });
    try {
        const { frame } = await enter(page, false);
        await expect.poll(async () => (await runtime(frame)).flights).toBeGreaterThan(0);
        await call(frame, 'set_art_enabled', [false]);
        await expect.poll(async () => (await runtime(frame)).cancelled_draining).toBeGreaterThan(0);
        const cancelled = await runtime(frame);
        expect(cancelled.flights).toBeLessThanOrEqual(2);
        expect(cancelled.planner_owners).toBe(0);
        expect(cancelled.sprite_image_owners).toBe(0);
        // Cross actual simulation/render frames while the old fetch is still delayed.
        await call(frame, 'set_art_enabled', [true]);
        await call(frame, 'switch_mode');
        await expect.poll(async () => (await state(frame)).mode).toBe('duel');
        await call(frame, 'switch_mode');
        await expect.poll(async () => (await state(frame)).mode).toBe('boss');
        expect((await runtime(frame)).flights).toBeLessThanOrEqual(2);
        release();
        await expect.poll(async () => (await state(frame)).art.status).toBe('ready');
        await expect.poll(async () => (await runtime(frame)).cancelled_draining).toBe(0);
        await expect.poll(async () => (await runtime(frame)).flights).toBe(0);
        const final = await runtime(frame);
        expect(final.bundles).toBe(4);
        expect((await state(frame)).mode).toBe('boss');
        await writeEvidence('d05b-in-flight-cancellation', {
            evidence_kind: 'runtime_lifecycle', captured_at: new Date().toISOString(),
            acquisition: 'Playwright delayed real PNG route; SetEnabled false and mode changes observed across actual frames; release network response and drain AssetServer terminal state',
            cancelled, final, physical_gpu_reclamation: 'NOT_RUN', mobile: 'BLOCKED: no physical device',
        });
    } finally { release(); }
});
