// Dokime — SPDX-License-Identifier: Apache-2.0
import assert from "node:assert/strict";
import { type ChildProcess, spawn } from "node:child_process";
import { once } from "node:events";
import { cp, mkdir, mkdtemp, readdir, realpath, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join, resolve, sep } from "node:path";
import { createInterface } from "node:readline";
import { fileURLToPath } from "node:url";
import { setTimeout } from "node:timers/promises";
import type { Session } from "../../e2e/native-fixture.ts";

// The Harness Parses Its Own Arguments; Criterion Arguments Belong To The Rust Owner.
process.argv = [process.execPath, fileURLToPath(import.meta.url), "--profile", "bench"];
const { startSession, prepareNativeFixture, discardHostCredentials, terminate } = await import("../../e2e/native-fixture.ts");
type NativeProcess = ChildProcess & { failure?: Error };
type Fixture = {
  shutdown_ms: number;
  endpoint: string;
  execution: string;
  cycle: string;
  origin_s: number;
  end_s: number;
  sample_rate_hz: number;
  viewport?: unknown;
};
type Observation = {
  errors: string[];
  visibleFailures: number;
  failedPlots: number;
  loadedPlots: number;
  paintedCount: number;
  visibleReady: number;
  allVisibleMs: number | null;
};
type Publication = { handoffWallMs: number; stages: Record<string, number[]> };
const DRAW_WAIT_MS = 600000;
const native = `${process.platform === "win32" ? "windows" : "linux"}-${process.arch === "x64" ? "x86_64" : "aarch64"}`;
const output = resolve(process.env.CRITERION_HOME!, "desktop");
const fixtureRoot = join(tmpdir(), "dokime-desktop-bench-fixtures");
await mkdir(fixtureRoot, { recursive: true });
await mkdir(output, { recursive: true });
const resolvedFixtureRoot = await realpath(fixtureRoot);
const work = await realpath(await mkdtemp(join(fixtureRoot, "run-")));
assert.ok(work.toLowerCase().startsWith(`${resolvedFixtureRoot}${sep}`.toLowerCase()), "Owned fixture stays inside its target directory");
const keepEvidence = process.env.DOKIME_KEEP_EVIDENCE === "1";
const results: (Record<string, unknown> & { outcome: string })[] = [];
const resources: { epoch: number; time_ms: number; cpu_s: number; memory_bytes: number }[] = [];
const captured = new Set();
let desktop: Session | undefined;
let cleanup: (() => Promise<void>) | undefined;
let host: NativeProcess | undefined;
let sampler: ChildProcess | undefined;
let finished = false;
let failure: unknown;
let fixture: Fixture | undefined;
let nativeRoot: string | undefined;
let nativeTemporary: string | undefined;
let seedRoot: string | undefined;
let backendMode: string | undefined;
let epoch = 0;
let warmReady: number | undefined;
let continuousReady: string | undefined;
let resizeWidth = 1920;
let historyRows = 2073600;
let measuring: string | undefined;

async function command(line: string): Promise<Publication> {
  if (host!.failure) throw host!.failure;
  const pending = hostLines!.next();
  const handoffWallMs = performance.timeOrigin + performance.now();
  const written = new Promise<void>((resolve, reject) => host!.stdin!.write(`${line}\n`, (error) => (error ? reject(error) : resolve())));
  const [response] = await Promise.all([pending, written]);
  if (response.done) throw new Error("Fixture host closed before command completion");
  return { ...JSON.parse(response.value), handoffWallMs };
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

let hostLines: AsyncIterator<string> | undefined;

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
        await desktop!.close();
      } finally {
        await terminate(desktop!.process);
        desktop = undefined;
      }
    },
    async () => {
      if (!host) return;
      const child = host;
      const started = performance.now();
      const exited = child.exitCode !== null || child.signalCode !== null ? Promise.resolve() : once(child, "exit");
      child.stdin!.end();
      const timer = new AbortController();
      let drained: boolean;
      try {
        drained = await Promise.race([exited.then(() => true), setTimeout(fixture?.shutdown_ms ?? 10000, false, { signal: timer.signal })]);
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

// Capture One Case Screenshot; When the Capture Fails, Record What the Page and the WebView Processes Were Doing.
async function screenshot(label: string) {
  const requested = performance.now();
  try {
    await desktop!.screenshot(label);
  } catch (error) {
    const elapsedMs = performance.now() - requested;
    const page = await desktop!
      .execute(
        `return {
          visibility: document.visibilityState,
          focused: document.hasFocus(),
          width: innerWidth,
          height: innerHeight,
          ratio: devicePixelRatio,
          lastFrameAgeMs: desktopFrameAge(),
          status: desktopResult(true),
        };`,
      )
      .catch((cause) => ({ error: String(cause) }));
    const threads = process.platform === "linux" ? await webviewThreads().catch((cause) => String(cause)) : null;
    await writeFile(
      join(output, `screenshot-failure-${label}.json`),
      JSON.stringify({ label, elapsedMs, error: String(error), page, threads }, null, 2),
    );
    throw error;
  }
}

// Each WebKit and Desktop Thread With Its State and the Kernel Function It Waits In.
async function webviewThreads() {
  const ps = spawn("ps", ["-eLo", "pid,tid,stat,wchan:32,comm", "--no-headers"], { stdio: ["ignore", "pipe", "inherit"] });
  let text = "";
  ps.stdout.setEncoding("utf8").on("data", (chunk) => (text += chunk));
  await once(ps, "close");
  return text.split(/\r?\n/).filter((line) => /webkit|dokime/i.test(line));
}

async function waitReady() {
  const deadline = performance.now() + 600000;
  while (true) {
    const status = await desktop!.execute<{ ready: boolean; failure?: string }>(
      "return {ready: typeof desktopCase === 'function', failure: window.desktopFailure ?? null}",
    );
    if (status.failure) throw new Error(`Benchmark frontend failed: ${status.failure}`);
    if (status.ready) return;
    if (performance.now() > deadline) throw new Error("Benchmark frontend did not initialize");
    await setTimeout(100);
  }
}

async function waitDrawing(count: number, condition: string, deadline = performance.now() + DRAW_WAIT_MS) {
  while (true) {
    const observation = await desktop!.execute<Observation>("return desktopResult(true)");
    if (!observation) throw new Error(`${count}/${condition}: native benchmark window closed before drawing completed`);
    if (observation.errors.length || observation.visibleFailures)
      throw new Error(`Draw Document ${count}/${condition}: ${observation.errors.join("; ")}`);
    const refused = observation.failedPlots === count && observation.loadedPlots === 0;
    const partial =
      observation.visibleFailures > 0 &&
      observation.loadedPlots + observation.failedPlots === count &&
      observation.paintedCount >= observation.visibleReady;
    if (refused || partial || observation.allVisibleMs !== null) return desktop!.execute<Observation>("return desktopResult()");
    if (performance.now() > deadline) throw new Error(`${count}/${condition}: no completed drawing or explicit refusal`);
    await setTimeout(25);
  }
}

async function resourceBoundary(endedMs: number) {
  const deadline = performance.now() + 5000;
  while (
    sampler?.exitCode === null &&
    performance.now() < deadline &&
    !resources.some((sample) => sample.epoch === epoch && sample.time_ms >= endedMs)
  ) {
    await setTimeout(25);
  }
}

async function startNative(mode: string) {
  // Close Every Reader Before the Next Host Opens the Saved Root.
  await closeNative();
  if (nativeRoot && seedRoot && !seedRoot.startsWith(`${nativeRoot}${sep}`)) {
    assert.ok(nativeRoot.startsWith(`${work}${sep}`), "Owned native profile stays inside its fixture directory");
    // Reopened Histories Share The Seed's Credential Slots Until Final Cleanup.
    if (backendMode === "active") await discardHostCredentials(nativeRoot);
    else await discardHostCredentials(join(nativeRoot, "shell"));
    await rm(nativeRoot, { recursive: true, force: true });
  }
  nativeRoot = await realpath(await mkdtemp(join(work, "native-")));
  // A Saved History Reopens the Seed Root Itself: Root Identity Refuses a Copied Root.
  const restored = mode === "saved" ? seedRoot : "";
  const fixtures = restored ? dirname(seedRoot!) : nativeRoot;
  nativeTemporary = await mkdtemp(join(tmpdir(), "dokime-desktop-bench-"));
  host = spawn(process.env.DOKIME_DESKTOP_BENCH_HOST!, ["--fixture-host"], {
    env: {
      ...process.env,
      TEMP: nativeTemporary,
      TMP: nativeTemporary,
      TMPDIR: nativeTemporary,
      DOKIME_DESKTOP_BENCH_FIXTURES: fixtures,
      DOKIME_DESKTOP_BENCH_RESTORE: restored,
      DOKIME_DESKTOP_HISTORY_ROWS: String(historyRows),
    },
    stdio: ["pipe", "pipe", "inherit"],
    detached: process.platform !== "win32",
    windowsHide: true,
  });
  const child = host;
  child.on("error", (error) => {
    child.failure = error;
  });
  child.stdin!.on("error", (error) => {
    child.failure = error;
  });
  hostLines = createInterface({ input: host.stdout! })[Symbol.asyncIterator]();
  const ready = await hostLines!.next();
  assert.ok(!ready.done, "fixture host is ready");
  const { root: storage, ...metadata } = JSON.parse(ready.value);
  assert.ok(Number.isFinite(metadata.shutdown_ms) && metadata.shutdown_ms > 0, "Fixture host owns its checkpoint shutdown budget");
  const storageRoot = await realpath(storage);
  assert.ok(storageRoot.toLowerCase().startsWith(`${fixtures}${sep}`.toLowerCase()), "Owned storage stays inside its native fixture");
  seedRoot ??= storageRoot;
  fixture = metadata;
  backendMode = mode;
  warmReady = undefined;
  continuousReady = undefined;
  epoch += 1;
  desktop = await startSession(nativeRoot, fixture!.endpoint, output, undefined, { startLocal: false });
  await waitReady();
  await command("prepare");
  await desktop!.executeAsync(`const done = arguments[arguments.length - 1];
    const invoke = globalThis.__TAURI_INTERNALS__.invoke;
    invoke('plugin:window|is_maximized', {label: 'main'}).then(async maximized => {
      if (maximized) await invoke('plugin:window|internal_toggle_maximize', {label: 'main'});
      done({maximized});
    }).catch(error => done({error: String(error)}));`);
  await desktop!.resize(1920, 1000);
  resizeWidth = 1920;
  fixture!.viewport = await desktop!.execute("return {width: innerWidth, height: innerHeight, deviceScale: devicePixelRatio}");
  if (samplerCode) {
    sampler = spawn("pwsh", ["-NoProfile", "-Command", samplerCode], {
      env: { ...process.env, DOKIME_BENCH_HOST_PID: String(host.pid), DOKIME_BENCH_DRIVER_PID: String(desktop!.process.pid) },
      stdio: ["ignore", "pipe", "inherit"],
      windowsHide: true,
    });
  } else {
    sampler = spawn(
      process.execPath,
      [fileURLToPath(new URL("resources.ts", import.meta.url)), String(host.pid), String(desktop!.process.pid)],
      {
        stdio: ["ignore", "pipe", "inherit"],
        detached: true,
      },
    );
  }
  const generation = epoch;
  createInterface({ input: sampler.stdout! }).on("line", (line) => resources.push({ ...JSON.parse(line), epoch: generation }));
  await resourceBoundary(Date.now());
  await writeFile(
    join(output, "fixture.json"),
    JSON.stringify({ ...fixture, native, rowsPerSecond: fixture!.sample_rate_hz, historyDays: 3, kinds: 10 }, null, 2),
  );
}

try {
  // Prepare The Native Harness And Millions Of Real Stored Sample Rows.
  cleanup = await prepareNativeFixture(work, output);
  await startNative("active");
  process.stdout.write("ready\n");
  for await (const line of createInterface({ input: process.stdin })) {
    const [rowsText, countText, condition, repetitions] = line.split(" ");
    measuring = `${rowsText}/${countText}/${condition}`;
    if (Number(rowsText) !== historyRows) {
      await closeNative();
      nativeRoot = undefined;
      seedRoot = undefined;
      backendMode = undefined;
      historyRows = Number(rowsText);
      await startNative("active");
    }
    const count = Number(countText);
    let elapsed = 0;
    for (let iteration = 0; iteration < Number(repetitions); iteration++) {
      // Cold Uses A Fresh Backend And Desktop Over The Same Closed History.
      const desired = ["cold", "warm"].includes(condition) ? "saved" : "active";
      if (condition === "cold" || backendMode !== desired) await startNative(desired);
      if (condition === "warm" && warmReady !== count) {
        await desktop!.execute("return desktopCase(...arguments)", [
          count,
          "warm",
          fixture!.execution,
          fixture!.cycle,
          fixture!.origin_s,
          fixture!.end_s + 600,
        ]);
        await waitDrawing(count, "warm-up");
        warmReady = count;
      }
      await command("drain");
      let resourceStart = Date.now();
      let started = performance.now();
      let drawingDeadline: number | undefined;
      let producerHandoffLeadMs: number | undefined;
      const operations = [];
      const to = fixture!.end_s + 600;
      const from = fixture!.origin_s;
      const continuous = ["live", "late", "stalled"].includes(condition);
      const retained = continuous && continuousReady === `${count}/${condition}`;
      if (!retained)
        await desktop!.execute("return desktopCase(...arguments)", [count, "warm", fixture!.execution, fixture!.cycle, from, to]);
      // Live And Late Publications Cross The Same Runtime Admission And Save Path.
      if (["live", "late", "stalled", "zoom", "resize", "navigation"].includes(condition)) {
        const initial = retained ? await desktop!.execute<Observation>("return desktopResult(true)") : await waitDrawing(count, "prepare");
        if (initial.loadedPlots > 0) {
          continuousReady = continuous && initial.loadedPlots === count ? `${count}/${condition}` : undefined;
          await command("drain");
          if (condition === "stalled") {
            await desktop!.execute(`
              const bench = globalThis.__desktopBench;
              let duration = 0;
              let until = 0;
              bench.stallStartedAt = 0;
              Object.defineProperty(bench, 'stallUntil', {
                configurable: true,
                get() {
                  if (duration > 0 && until === 0 && bench.queueDepth > 0) {
                    bench.stallStartedAt = performance.now();
                    until = bench.stallStartedAt + duration;
                  }
                  return until;
                },
                set(value) {
                  duration = Math.max(0, value - performance.now());
                  until = 0;
                  bench.stallStartedAt = 0;
                },
              });`);
          }
          resourceStart = Date.now();
          started = performance.now();
          const actionStartedWallMs = await desktop!.execute<number>(
            "desktopAction(...arguments); return performance.timeOrigin + __desktopBench.started;",
            [condition, fixture!.end_s - 150, fixture!.end_s - 120],
          );
          if (["live", "late", "stalled"].includes(condition)) {
            const publication = await command(condition === "late" ? "late" : "live");
            producerHandoffLeadMs = publication.handoffWallMs - actionStartedWallMs;
            operations.push(publication);
          }
          if (condition === "stalled") {
            drawingDeadline = started + DRAW_WAIT_MS;
            let status: { paused: boolean; refused: boolean };
            while (true) {
              status = await desktop!.execute<{ paused: boolean; refused: boolean }>(
                "return {paused: Boolean(__desktopBench.stallStartedAt), refused: __desktopBench.failedPlots === arguments[0] && __desktopBench.loadedPlots === 0}",
                [count],
              );
              if (!status) throw new Error("Native benchmark window closed before the queued-frame pause");
              if (status.paused || status.refused) break;
              if (performance.now() > drawingDeadline) throw new Error("No plot frame arrived for the stalled consumer");
              await setTimeout(25);
            }
            for (let batch = 0; status.paused && batch < 12; batch++) {
              operations.push(await command("live"));
              await setTimeout(100);
            }
          }
        }
      }
      if (condition === "resize") {
        resizeWidth = resizeWidth === 1920 ? 1600 : 1920;
        await desktop!.resize(resizeWidth, 1000);
      }
      const observation = await waitDrawing(count, condition, drawingDeadline);
      if (observation.errors.length) continuousReady = undefined;
      operations.push(await command("drain"));
      const stages: Record<string, number[]> = {};
      for (const operation of operations) {
        for (const [stage, values] of Object.entries(operation.stages)) {
          stages[stage] ??= [];
          stages[stage].push(...values);
        }
      }
      const durationMs = performance.now() - started;
      // Align The WebView's Monotonic Origin With The Producer's Handoff Marker.
      const producerHandoffToChangedDrawingMs =
        producerHandoffLeadMs === undefined ? null : observation.allVisibleMs! - producerHandoffLeadMs;
      const timedBoundaryMs = producerHandoffToChangedDrawingMs ?? observation.allVisibleMs;
      if (timedBoundaryMs === null) throw new Error(`Draw Document ${count}/${condition}: No Completed Visible Drawing`);
      elapsed += timedBoundaryMs / 1000;
      await resourceBoundary(Date.now());
      const samples = resources.filter((sample) => sample.epoch === epoch && sample.time_ms >= resourceStart);
      const boundary = resources.findLast((sample) => sample.epoch === epoch && sample.time_ms <= resourceStart);
      if (boundary) samples.unshift(boundary);
      results.push({
        rows: historyRows,
        count,
        condition,
        iteration,
        epoch,
        backendMode,
        fixture: { ...fixture },
        ...observation,
        timedBoundaryMs,
        producerHandoffToChangedDrawingMs,
        durationMs,
        queryMs: stages["plot.query"] ?? [],
        backendStages: stages,
        peakMemoryBytes: samples.length ? Math.max(...samples.map((sample) => sample.memory_bytes)) : null,
        cpuSeconds: samples.length > 1 ? Math.max(0, samples.at(-1)!.cpu_s - samples[0].cpu_s) : null,
        outcome: "visible",
        resourceSamples: samples.length,
        resourceWindowMs: samples.length > 1 ? samples.at(-1)!.time_ms - samples[0].time_ms : null,
      });
      await writeFile(join(output, "observations.json"), JSON.stringify({ fixture, native, results, resources }, null, 2));
      const capture = `${historyRows}-${count}-${condition}`;
      if (!captured.has(capture)) {
        await screenshot(capture);
        captured.add(capture);
      }
      // Traverse Every Plot Outside the Timing Boundary, Including All Mixed Kinds.
      {
        await command("drain");
        const traversalStarted = performance.now();
        const traversal = await desktop!.executeAsync<{ error?: string; verifiedPlots: number }>(
          `const done = arguments[arguments.length - 1];
          desktopTraverse(arguments[0], arguments[1]).then(done).catch(error => done({error: String(error)}));`,
          [count, DRAW_WAIT_MS],
        );
        if (traversal.error) throw new Error(traversal.error);
        assert.equal(traversal.verifiedPlots, count);
        Object.assign(results.at(-1)!, traversal);
        results.at(-1)!.traversalDriverMs = performance.now() - traversalStarted;
        results.at(-1)!.traversalBackendStages = (await command("drain")).stages;
        results.at(-1)!.outcome = "drawn";
        await desktop!.executeAsync(`const done = arguments[arguments.length - 1];
          window.scrollTo(0, 0); requestAnimationFrame(() => requestAnimationFrame(() => done(true)));`);
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
  let credentialsRemoved = !cleanup;
  for (const close of [
    closeNative,
    async () => {
      await cleanup?.();
      credentialsRemoved = true;
    },
  ]) {
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
  for (const directory of keepEvidence || !credentialsRemoved ? [] : [work]) {
    try {
      await rm(directory, { recursive: true, force: true, maxRetries: 5, retryDelay: 200 });
    } catch (error) {
      errors.push(String(error));
    }
  }
  if (errors.length) failure ??= new Error(`Native benchmark cleanup failed: ${errors.join("; ")}`);
  await writeFile(join(output, "observations.json"), JSON.stringify({ fixture: fixture ?? null, native, results, resources }, null, 2));
  await writeFile(
    join(output, "completion.json"),
    JSON.stringify({
      finished,
      results: results.length,
      errors,
      failure: (failure as Error | undefined)?.stack ?? null,
      measuring,
      keptEvidence: keepEvidence || !credentialsRemoved ? [work] : [],
    }),
  );
}
if (failure) {
  process.stdout.write(
    `error ${JSON.stringify(`Measure Desktop ${measuring ?? "Initialization"}: ${(failure as Error).message ?? failure}`)}\n`,
  );
  throw failure;
}
