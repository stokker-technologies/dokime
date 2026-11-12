// Dokime — SPDX-License-Identifier: Apache-2.0
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { once } from "node:events";
import { cp, mkdir, mkdtemp, readdir, realpath, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve, sep } from "node:path";
import { createInterface } from "node:readline";
import { fileURLToPath } from "node:url";
import { setTimeout as delay } from "node:timers/promises";

// The Harness Parses Its Own Arguments; Criterion Arguments Belong To The Rust Owner.
process.argv = [process.execPath, fileURLToPath(import.meta.url), "--profile", "bench"];
const { startSession, prepareNativeFixture, terminate } = await import("../../e2e/native-desktop.mjs");
const root = fileURLToPath(new URL("../../../../", import.meta.url));
const native = `${process.platform === "win32" ? "windows" : "linux"}-${process.arch === "x64" ? "x86_64" : "aarch64"}`;
const output = resolve(process.env.CRITERION_HOME, "desktop");
const fixtureRoot = join(root, "target/desktop-bench-fixtures");
await mkdir(fixtureRoot, { recursive: true });
await mkdir(output, { recursive: true });
const resolvedFixtureRoot = await realpath(fixtureRoot);
const work = await realpath(await mkdtemp(join(fixtureRoot, "run-")));
assert.ok(work.toLowerCase().startsWith(`${resolvedFixtureRoot}${sep}`.toLowerCase()), "Owned fixture stays inside its target directory");
const keepEvidence = process.env.DOKIME_KEEP_EVIDENCE === "1";
const results = [];
const resources = [];
const captured = new Set();
let desktop;
let cleanup;
let host;
let sampler;
let finished = false;
let failure;
let fixture;
let nativeRoot;
let nativeTemporary;
let seedRoot;
let backendMode;
let epoch = 0;
let warmReady;
let continuousReady;
let resizeWidth = 1920;

async function command(line) {
  if (host.failure) throw host.failure;
  const pending = hostLines.next();
  const written = new Promise((resolve, reject) => host.stdin.write(`${line}\n`, (error) => (error ? reject(error) : resolve())));
  const [response] = await Promise.all([pending, written]);
  if (response.done) throw new Error("Fixture host closed before command completion");
  return JSON.parse(response.value);
}

// Resource Samples Include The Owned Fixture Host, Shell And Descendants, Including WebView Workers.
const samplerCode =
  process.platform === "win32"
    ? `
  $ids = @([int]$env:DOKIME_BENCH_HOST_PID, [int]$env:DOKIME_BENCH_DRIVER_PID);
  while ($true) {
    $all = @(Get-CimInstance Win32_Process);
    $owned = @($ids);
    do { $new = @($all | Where-Object { $_.ParentProcessId -in $owned -and $_.ProcessId -notin $owned } | ForEach-Object { [int]$_.ProcessId }); $owned += $new } while ($new.Count);
    $processes = @(Get-Process -Id $owned -ErrorAction SilentlyContinue);
    @{ time_ms = [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds(); cpu_s = ($processes | Measure-Object CPU -Sum).Sum; memory_bytes = ($processes | Measure-Object WorkingSet64 -Sum).Sum } | ConvertTo-Json -Compress;
    Start-Sleep -Milliseconds 200;
  }`
    : null;

let hostLines;

async function closeNative() {
  const errors = [];
  for (const close of [
    async () => {
      await terminate(sampler);
      sampler = undefined;
    },
    async () => {
      if (!desktop) return;
      try {
        await desktop.close();
      } finally {
        await terminate(desktop.process);
        desktop = undefined;
      }
    },
    async () => {
      if (!host) return;
      const child = host;
      const started = performance.now();
      const exited = child.exitCode !== null || child.signalCode !== null ? Promise.resolve() : once(child, "exit");
      child.stdin.end();
      const timer = new AbortController();
      let drained;
      try {
        drained = await Promise.race([exited.then(() => true), delay(fixture?.shutdown_ms ?? 10000, false, { signal: timer.signal })]);
      } finally {
        timer.abort();
      }
      if (!drained) {
        await terminate(child);
        await exited;
      }
      host = undefined;
      if (!drained || child.exitCode !== 0) {
        await writeFile(
          join(output, "host-shutdown.json"),
          JSON.stringify({ drained, durationMs: performance.now() - started, code: child.exitCode, signal: child.signalCode }),
        );
        throw new Error(`Fixture host exited ${child.exitCode} (${child.signalCode ?? "no signal"}; drained=${drained})`);
      }
    },
    async () => {
      if (nativeTemporary && !keepEvidence) await rm(nativeTemporary, { recursive: true, force: true, maxRetries: 5, retryDelay: 200 });
      nativeTemporary = undefined;
    },
  ]) {
    try {
      await close();
    } catch (error) {
      errors.push(String(error));
    }
  }
  if (errors.length) throw new Error(errors.join("; "));
}

async function waitReady() {
  const deadline = performance.now() + 600000;
  while (true) {
    const status = await desktop.execute("return {ready: typeof desktopCase === 'function', failure: window.desktopFailure ?? null}");
    if (status.failure) throw new Error(`Benchmark frontend failed: ${status.failure}`);
    if (status.ready) return;
    if (performance.now() > deadline) throw new Error("Benchmark frontend did not initialize");
    await delay(100);
  }
}

async function waitDrawing(count, condition) {
  const deadline = performance.now() + 600000;
  while (true) {
    const observation = await desktop.execute("return desktopResult(true)");
    if (!observation) throw new Error(`${count}/${condition}: native benchmark window closed before drawing completed`);
    const refused = observation.failedPlots === count && observation.loadedPlots === 0;
    const partial =
      observation.visibleFailures > 0 &&
      observation.loadedPlots + observation.failedPlots === count &&
      observation.paintedCount >= observation.visibleReady;
    if (refused || partial || observation.allVisibleMs !== null) return desktop.execute("return desktopResult()");
    if (performance.now() > deadline) throw new Error(`${count}/${condition}: no completed drawing or explicit refusal`);
    await delay(25);
  }
}

async function resourceBoundary(endedMs) {
  const deadline = performance.now() + 5000;
  while (
    sampler?.exitCode === null &&
    performance.now() < deadline &&
    !resources.some((sample) => sample.epoch === epoch && sample.time_ms >= endedMs)
  ) {
    await delay(25);
  }
}

async function startNative(mode) {
  // Close Every Reader Before Copying Its Durable Checkpoint.
  await closeNative();
  if (nativeRoot && seedRoot && !seedRoot.startsWith(`${nativeRoot}${sep}`)) {
    assert.ok(nativeRoot.startsWith(`${work}${sep}`), "Owned native profile stays inside its fixture directory");
    await rm(nativeRoot, { recursive: true, force: true });
  }
  nativeRoot = await realpath(await mkdtemp(join(work, "native-")));
  const restored = mode === "saved" ? join(nativeRoot, "data") : "";
  if (restored) await cp(seedRoot, restored, { recursive: true, errorOnExist: true, force: false });
  nativeTemporary = await mkdtemp(join(tmpdir(), "dokime-desktop-bench-"));
  host = spawn(process.env.DOKIME_DESKTOP_BENCH_HOST, ["--fixture-host"], {
    env: {
      ...process.env,
      TEMP: nativeTemporary,
      TMP: nativeTemporary,
      TMPDIR: nativeTemporary,
      DOKIME_DESKTOP_BENCH_FIXTURES: nativeRoot,
      DOKIME_DESKTOP_BENCH_RESTORE: restored,
    },
    stdio: ["pipe", "pipe", "inherit"],
    detached: process.platform !== "win32",
    windowsHide: true,
  });
  const child = host;
  child.on("error", (error) => {
    child.failure = error;
  });
  child.stdin.on("error", (error) => {
    child.failure = error;
  });
  hostLines = createInterface({ input: host.stdout })[Symbol.asyncIterator]();
  const ready = await hostLines.next();
  assert.ok(!ready.done, "fixture host is ready");
  const { root: storage, ...metadata } = JSON.parse(ready.value);
  assert.ok(Number.isFinite(metadata.shutdown_ms) && metadata.shutdown_ms > 0, "Fixture host owns its checkpoint shutdown budget");
  const storageRoot = await realpath(storage);
  assert.ok(storageRoot.toLowerCase().startsWith(`${nativeRoot}${sep}`.toLowerCase()), "Owned storage stays inside its native fixture");
  seedRoot ??= storageRoot;
  fixture = metadata;
  backendMode = mode;
  warmReady = undefined;
  continuousReady = undefined;
  epoch += 1;
  desktop = await startSession(nativeRoot, fixture.endpoint, output, undefined, { startLocal: false });
  await waitReady();
  await command("prepare");
  await desktop.executeAsync(`const done = arguments[arguments.length - 1];
    const invoke = globalThis.__TAURI_INTERNALS__.invoke;
    invoke('plugin:window|is_maximized', {label: 'main'}).then(async maximized => {
      if (maximized) await invoke('plugin:window|internal_toggle_maximize', {label: 'main'});
      done({maximized});
    }).catch(error => done({error: String(error)}));`);
  await desktop.resize(1920, 1000);
  resizeWidth = 1920;
  fixture.viewport = await desktop.execute("return {width: innerWidth, height: innerHeight, deviceScale: devicePixelRatio}");
  if (samplerCode) {
    sampler = spawn("pwsh", ["-NoProfile", "-Command", samplerCode], {
      env: { ...process.env, DOKIME_BENCH_HOST_PID: String(host.pid), DOKIME_BENCH_DRIVER_PID: String(desktop.process.pid) },
      stdio: ["ignore", "pipe", "inherit"],
      windowsHide: true,
    });
  } else {
    sampler = spawn(
      process.execPath,
      [fileURLToPath(new URL("resources.mjs", import.meta.url)), String(host.pid), String(desktop.process.pid)],
      {
        stdio: ["ignore", "pipe", "inherit"],
        detached: true,
      },
    );
  }
  const generation = epoch;
  createInterface({ input: sampler.stdout }).on("line", (line) => resources.push({ ...JSON.parse(line), epoch: generation }));
  await resourceBoundary(Date.now());
  await writeFile(
    join(output, "fixture.json"),
    JSON.stringify({ ...fixture, native, rowsPerSecond: 8, historyDays: 3, kinds: 10 }, null, 2),
  );
}

try {
  // Prepare The Native Harness And Millions Of Real Stored Sample Rows.
  cleanup = await prepareNativeFixture(work, output);
  await startNative("active");
  process.stdout.write("ready\n");
  for await (const line of createInterface({ input: process.stdin })) {
    const [countText, condition, repetitions] = line.split(" ");
    const count = Number(countText);
    let elapsed = 0;
    for (let iteration = 0; iteration < Number(repetitions); iteration++) {
      // Cold Uses A Fresh Backend And Desktop Over The Same Closed History.
      const desired = ["cold", "warm"].includes(condition) ? "saved" : "active";
      if (condition === "cold" || backendMode !== desired) await startNative(desired);
      if (condition === "warm" && warmReady !== count) {
        await desktop.execute("desktopCase(...arguments)", [
          count,
          "warm",
          fixture.execution,
          fixture.cycle,
          fixture.origin_s,
          fixture.end_s + 600,
        ]);
        await waitDrawing(count, "warm-up");
        warmReady = count;
      }
      await command("drain");
      let resourceStart = Date.now();
      let started = performance.now();
      const to = fixture.end_s + 600;
      const from = fixture.origin_s;
      const continuous = count === 30 && ["live", "late", "stalled"].includes(condition);
      const retained = continuous && continuousReady === `${count}/${condition}`;
      if (!retained) await desktop.execute("desktopCase(...arguments)", [count, "warm", fixture.execution, fixture.cycle, from, to]);
      // Live And Late Publications Cross The Same Runtime Admission And Save Path.
      if (count === 30 && ["live", "late", "stalled", "zoom", "resize", "navigation"].includes(condition)) {
        const initial = retained ? await desktop.execute("return desktopResult(true)") : await waitDrawing(count, "prepare");
        if (initial.loadedPlots > 0) {
          continuousReady = continuous && initial.loadedPlots === count ? `${count}/${condition}` : undefined;
          await command("drain");
          resourceStart = Date.now();
          started = performance.now();
          await desktop.execute("desktopAction(...arguments)", [condition, fixture.end_s - 150, fixture.end_s - 120]);
          if (["live", "late", "stalled"].includes(condition)) await command(condition === "late" ? "late" : "live");
          if (condition === "stalled") {
            for (let batch = 0; batch < 12; batch++) {
              await command("live");
              await delay(100);
            }
          }
        }
      }
      if (count === 100 && ["live", "late", "stalled"].includes(condition)) {
        await command(condition === "late" ? "late" : "live");
        if (condition === "stalled") {
          for (let batch = 0; batch < 12; batch++) {
            await command("live");
            await delay(100);
          }
        }
      }
      if (condition === "resize") {
        resizeWidth = resizeWidth === 1920 ? 1600 : 1920;
        await desktop.resize(resizeWidth, 1000);
      }
      const observation = await waitDrawing(count, condition);
      if (observation.errors.length) continuousReady = undefined;
      const queries = await command("drain");
      const durationMs = performance.now() - started;
      elapsed += durationMs / 1000;
      await resourceBoundary(Date.now());
      const samples = resources.filter((sample) => sample.epoch === epoch && sample.time_ms >= resourceStart);
      const boundary = resources.findLast((sample) => sample.epoch === epoch && sample.time_ms <= resourceStart);
      if (boundary) samples.unshift(boundary);
      results.push({
        count,
        condition,
        iteration,
        epoch,
        backendMode,
        fixture: { ...fixture },
        ...observation,
        durationMs,
        queryMs: queries.stages["plot.query"] ?? [],
        backendStages: queries.stages,
        peakMemoryBytes: samples.length ? Math.max(...samples.map((sample) => sample.memory_bytes)) : null,
        cpuSeconds: samples.length > 1 ? Math.max(0, samples.at(-1).cpu_s - samples[0].cpu_s) : null,
        outcome: observation.errors.length ? (observation.loadedPlots > 0 ? "partial" : "refused") : "drawn",
        resourceSamples: samples.length,
        resourceWindowMs: samples.length > 1 ? samples.at(-1).time_ms - samples[0].time_ms : null,
      });
      await writeFile(join(output, "observations.json"), JSON.stringify({ fixture, native, results, resources }, null, 2));
      const capture = `${count}-${condition}`;
      if (!captured.has(capture)) {
        await desktop.screenshot(capture);
        captured.add(capture);
      }
    }
    await writeFile(join(output, "observations.json"), JSON.stringify({ fixture, native, results, resources }, null, 2));
    process.stdout.write(`${elapsed}\n`);
  }
  finished = true;
} catch (error) {
  failure = error;
  await desktop?.screenshot("failure").catch(() => {});
} finally {
  const errors = [];
  for (const close of [closeNative, async () => cleanup?.()]) {
    try {
      await close();
    } catch (error) {
      errors.push(String(error));
    }
  }
  if (nativeRoot && (failure || errors.length)) {
    const logs = join(nativeRoot, "shell/application/logs");
    for (const name of await readdir(logs).catch(() => [])) {
      if (!name.endsWith(".log")) continue;
      await cp(join(logs, name), join(output, `desktop-${name}`)).catch((error) => errors.push(`Desktop log: ${error}`));
    }
  }
  // Reclaim Fixtures On Success And Failure After All Owned Children Stop.
  for (const directory of keepEvidence ? [] : [work]) {
    try {
      await rm(directory, { recursive: true, force: true, maxRetries: 5, retryDelay: 200 });
    } catch (error) {
      errors.push(String(error));
    }
  }
  await writeFile(
    join(output, "completion.json"),
    JSON.stringify({
      finished,
      results: results.length,
      errors,
      failure: failure?.stack ?? null,
      keptEvidence: keepEvidence ? [work] : [],
    }),
  );
  if (errors.length) failure ??= new Error(`Native benchmark cleanup failed: ${errors.join("; ")}`);
}
if (failure) throw failure;
