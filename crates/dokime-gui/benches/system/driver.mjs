// Dokime — SPDX-License-Identifier: Apache-2.0
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { once } from "node:events";
import { mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";
import { join, resolve } from "node:path";
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
const work = await mkdtemp(join(fixtureRoot, "run-"));
const results = [];
const resources = [];
let desktop;
let cleanup;
let host;
let sampler;
let finished = false;
let failure;

async function command(line) {
  const pending = hostLines.next();
  host.stdin.write(`${line}\n`);
  const response = await pending;
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
try {
  // Prepare The Native Harness And Millions Of Real Stored Sample Rows.
  cleanup = await prepareNativeFixture(work, output);
  host = spawn(process.env.DOKIME_DESKTOP_BENCH_HOST, ["--fixture-host"], {
    env: { ...process.env, DOKIME_DESKTOP_BENCH_FIXTURES: work },
    stdio: ["pipe", "pipe", "inherit"],
    windowsHide: true,
  });
  hostLines = createInterface({ input: host.stdout })[Symbol.asyncIterator]();
  const ready = await hostLines.next();
  assert.ok(!ready.done, "fixture host is ready");
  const fixture = JSON.parse(ready.value);
  await writeFile(
    join(output, "fixture.json"),
    JSON.stringify({ ...fixture, native, rowsPerSecond: 8, historyDays: 3, kinds: 10 }, null, 2),
  );
  desktop = await startSession(work, fixture.endpoint, output, undefined, { startLocal: false });
  const waitReady = async () => {
    const deadline = performance.now() + 600000;
    while (!(await desktop.execute("return typeof desktopCase === 'function'"))) {
      if (performance.now() > deadline) throw new Error("Benchmark frontend did not initialize");
      await delay(100);
    }
  };
  const waitDrawing = async (count, condition) => {
    const deadline = performance.now() + 600000;
    while (true) {
      const observation = await desktop.execute("return desktopResult(true)");
      if (observation.errors.length || observation.allVisibleMs !== null) return desktop.execute("return desktopResult()");
      if (performance.now() > deadline) throw new Error(`${count}/${condition}: no completed drawing or explicit refusal`);
      await delay(25);
    }
  };
  await waitReady();
  await desktop.executeAsync(`const done = arguments[arguments.length - 1];
    const invoke = globalThis.__TAURI_INTERNALS__.invoke;
    invoke('plugin:window|is_maximized', {label: 'main'}).then(async maximized => {
      if (maximized) await invoke('plugin:window|internal_toggle_maximize', {label: 'main'});
      done({maximized});
    }).catch(error => done({error: String(error)}));`);
  await desktop.resize(1920, 1000);
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
      },
    );
  }
  const sampled = createInterface({ input: sampler.stdout });
  sampled.on("line", (line) => resources.push(JSON.parse(line)));
  process.stdout.write("ready\n");
  for await (const line of createInterface({ input: process.stdin })) {
    const [countText, condition, repetitions] = line.split(" ");
    const count = Number(countText);
    let elapsed = 0;
    for (let iteration = 0; iteration < Number(repetitions); iteration++) {
      // Cold Means A Fresh WebView/JS State, Not A Claim Of Evicted OS Disk Caches.
      if (condition === "cold") {
        await desktop.request("POST", "/refresh", {});
        await waitReady();
      }
      await command("drain");
      let resourceStart = Date.now();
      let started = performance.now();
      const to = fixture.end_s + 600;
      const from = fixture.origin_s;
      await desktop.execute("desktopCase(...arguments)", [count, "warm", fixture.execution, fixture.cycle, from, to]);
      // Live And Late Publications Cross The Same Runtime Admission And Save Path.
      if (count === 30 && ["live", "late", "stalled", "zoom", "resize", "navigation"].includes(condition)) {
        const initial = await waitDrawing(count, "prepare");
        if (!initial.errors.length) {
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
      if (condition === "resize") await desktop.resize(iteration % 2 ? 1600 : 1920, 1000);
      const observation = await waitDrawing(count, condition);
      const queries = await command("drain");
      const durationMs = performance.now() - started;
      elapsed += durationMs / 1000;
      const samples = resources.filter((sample) => sample.time_ms >= resourceStart);
      const boundary = resources.findLast((sample) => sample.time_ms <= resourceStart);
      if (boundary) samples.unshift(boundary);
      results.push({
        count,
        condition,
        iteration,
        ...observation,
        durationMs,
        queryMs: queries.stages["plot.query"] ?? [],
        backendStages: queries.stages,
        peakMemoryBytes: samples.length ? Math.max(...samples.map((sample) => sample.memory_bytes)) : null,
        cpuSeconds: samples.length > 1 ? Math.max(0, samples.at(-1).cpu_s - samples[0].cpu_s) : null,
        outcome: observation.errors.length ? "refused" : "drawn",
        resourceSamples: samples.length,
      });
    }
    await writeFile(join(output, "observations.json"), JSON.stringify({ fixture, native, results, resources }, null, 2));
    process.stdout.write(`${elapsed}\n`);
  }
  finished = true;
} catch (error) {
  failure = error;
} finally {
  const errors = [];
  for (const close of [
    () => terminate(sampler),
    async () => {
      if (!desktop) return;
      try {
        await desktop.close();
      } finally {
        await terminate(desktop.process);
      }
    },
    async () => {
      if (!host) return;
      host.stdin.end();
      if (host.exitCode === null) await Promise.race([once(host, "exit"), delay(10000)]);
      if (host.exitCode === null) await terminate(host);
    },
    async () => cleanup?.(),
  ]) {
    try {
      await close();
    } catch (error) {
      errors.push(String(error));
    }
  }
  await writeFile(join(output, "completion.json"), JSON.stringify({ finished, results: results.length, errors }));
  // Heavy Fixtures Are Owned, Disposable And Kept Under Target Until Processes Close.
  if (!errors.length) await rm(work, { recursive: true, force: true });
  if (errors.length) failure ??= new Error(`Native benchmark cleanup failed: ${errors.join("; ")}`);
}
if (failure) throw failure;
