// ***
//
// Dokime
//
// SPDX-License-Identifier: Apache-2.0
//
// ***

// Plot Stream Throughput Through The Native WebView: IPC Bridge Versus gRPC-Web.
//
// Starts a backend with the OBC charger plugin and gRPC-Web enabled for this
// WebView's origin, attaches the desktop shell to it, and injects two builds of
// the GUI's own transport and plot adapter into the page: one bound to the IPC
// bridge the desktop ships, one bound to direct gRPC-Web, the previous path.
// Each measured window starts a fresh continuous OBC startup run publishing
// telemetry at a one-millisecond step and watches its plot through
// `openNativePlotStream` on one transport, alternating the order per round. The
// page records, for every delivered frame, when it arrived and when the backend
// generated it. Unary round trips are timed the same way.
//
//   node crates/dokime-gui/e2e/stream-throughput.mjs [--seconds 15] [--rounds 2] [--points 2000] [--window 2]
//
// Prerequisites match native-desktop.mjs: a running display (or a headless
// compositor), WebKitWebDriver, and the dev-built shell (with
// `tauri/custom-protocol`), backend and OBC bundle under the target directory.
// Results print as JSON and land in `<target>/stream-throughput.json`.

/* === Imports === */

// Standard Library Imports
import { spawn } from "node:child_process";
import { once } from "node:events";
import { mkdir, mkdtemp, open, rm, writeFile } from "node:fs/promises";
import net from "node:net";
import { dirname, join, resolve } from "node:path";
import { setTimeout as delay } from "node:timers/promises";
import { fileURLToPath } from "node:url";
import { parseArgs } from "node:util";

// Third Party Imports
import { build } from "vite";

/* === Constants === */

const gui = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const workspace = resolve(gui, "../..");
const targetRoot = resolve(workspace, process.env.CARGO_TARGET_DIR ?? "target");
const backendBinary = process.env.DOKIME_E2E_BACKEND ?? join(targetRoot, "debug/dokime-server");
const shellBinary = process.env.DOKIME_E2E_SHELL ?? join(targetRoot, "debug/dokime-gui");
const pluginBundle = process.env.DOKIME_E2E_PLUGIN ?? join(targetRoot, "debug/plugins/obc-plugin.dokime");
const origin = process.platform === "win32" ? "http://tauri.localhost" : "tauri://localhost";
const STREAM = "mini.charger.startup.samples";
const TEMPLATE = "Power Up the Charger";
const STEP_S = 0.001;
const { values: options } = parseArgs({
  options: {
    seconds: { type: "string", default: "15" },
    rounds: { type: "string", default: "2" },
    calls: { type: "string", default: "300" },
    points: { type: "string", default: "2000" },
    window: { type: "string", default: "2" },
  },
});

/* === Private Functions === */

async function unusedPort() {
  const socket = net.createServer();
  socket.listen(0, "127.0.0.1");
  await once(socket, "listening");
  const port = socket.address().port;
  await new Promise((done) => socket.close(done));
  return port;
}

async function until(description, check, timeout = 60000) {
  const deadline = Date.now() + timeout;
  let last;
  while (Date.now() < deadline) {
    try {
      const value = await check();
      if (value) return value;
    } catch (failure) {
      last = failure;
    }
    await delay(200);
  }
  throw new Error(`Timed out: ${description}${last ? ` (${last.message})` : ""}`);
}

/** The GUI's transport and plot adapter, built once per transport selection. */
async function bundle(transport) {
  const entry = join(gui, "node_modules/.cache", `stream-throughput-${transport}.ts`);
  await mkdir(dirname(entry), { recursive: true });
  await writeFile(
    entry,
    `import * as api from "@/lib/rpc/generated/client";
import { bindConnection } from "@/lib/rpc/grpc-web";
import { openNativePlotStream } from "@/lib/api/native-plots";
const bench = ((globalThis as any).__dokimeBench ??= {});
bench[${JSON.stringify(transport)}] = { api, bindConnection, openNativePlotStream };
`,
  );
  const output = await build({
    configFile: false,
    root: gui,
    logLevel: "warn",
    define: {
      __APP_VERSION__: JSON.stringify("bench"),
      __DOKIME_UI_MODE__: JSON.stringify("live"),
      __DOKIME_TRANSPORT__: JSON.stringify(transport),
    },
    resolve: { alias: { "@": join(gui, "src") } },
    build: {
      write: false,
      minify: false,
      lib: { entry, formats: ["iife"], name: `DokimeBench_${transport.replace("-", "_")}` },
    },
  });
  const chunks = (Array.isArray(output) ? output : [output]).flatMap((result) => result.output);
  return chunks.find((chunk) => chunk.type === "chunk").code;
}

async function startBackend(work) {
  const port = await unusedPort();
  const log = await open(join(work, "backend.log"), "w");
  const child = spawn(
    backendBinary,
    [
      "--shutdown-stdin",
      "--grpc-web",
      "--allowed-origin",
      origin,
      "--plugin",
      pluginBundle,
      "--port",
      String(port),
      "--output-root",
      join(work, "data"),
      "--application-root",
      join(work, "application"),
    ],
    { cwd: work, stdio: ["pipe", log.fd, log.fd] },
  );
  const endpoint = `http://127.0.0.1:${port}`;
  await until("backend listener", async () => {
    if (child.exitCode !== null) throw new Error("the backend exited");
    return new Promise((done) => {
      const socket = net.connect(port, "127.0.0.1");
      socket.once("connect", () => {
        socket.destroy();
        done(true);
      });
      socket.once("error", () => done(false));
    });
  });
  return { child, endpoint, log };
}

async function startSession(work, endpoint) {
  const home = join(work, "shell");
  for (const directory of ["config", "data", "state", "cache"]) {
    await mkdir(join(home, directory), { recursive: true });
  }
  const port = await unusedPort();
  const driver = spawn("WebKitWebDriver", ["--host=127.0.0.1", `--port=${port}`], {
    env: {
      ...process.env,
      TAURI_WEBVIEW_AUTOMATION: "true",
      DOKIME_APPLICATION_ROOT: join(home, "application"),
      HOME: home,
      XDG_CACHE_HOME: join(home, "cache"),
      XDG_CONFIG_HOME: join(home, "config"),
      XDG_DATA_HOME: join(home, "data"),
      XDG_STATE_HOME: join(home, "state"),
    },
    stdio: ["ignore", "ignore", "inherit"],
  });
  const base = `http://127.0.0.1:${port}`;
  async function request(method, path, body) {
    const response = await fetch(base + path, {
      method,
      headers: { "content-type": "application/json" },
      ...(body === undefined ? {} : { body: JSON.stringify(body) }),
      signal: AbortSignal.timeout(300000),
    });
    const result = (await response.json()).value;
    if (!response.ok || result?.error) throw new Error(`${path}: ${result?.message ?? response.status}`);
    return result;
  }
  await until("WebDriver", () => request("GET", "/status"));
  const created = await request("POST", "/session", {
    capabilities: {
      alwaysMatch: {
        browserName: "wry",
        "webkitgtk:browserOptions": { args: ["--endpoint", endpoint], binary: shellBinary },
      },
    },
  });
  const id = created.sessionId;
  await request("POST", `/session/${id}/timeouts`, { script: 600000 });
  return {
    driver,
    execute: (script, args = []) => request("POST", `/session/${id}/execute/sync`, { script, args }),
    executeAsync: (script, args = []) => request("POST", `/session/${id}/execute/async`, { script, args }),
    close: () => request("DELETE", `/session/${id}`).catch(() => undefined),
  };
}

/** Everything the page runs, as one async function source. */
const PAGE = `
const [phase, argument, done] = arguments;
const bench = globalThis.__dokimeBench;
const internals = globalThis.__TAURI_INTERNALS__;
const fail = (error) => done({ error: String(error && (error.stack || error.message) || error) });
async function connection() {
  const status = await internals.invoke("desktop_session_status");
  const backend = status.connection.backend;
  const session = { generation: status.connection.generation, isCurrent: () => true };
  bench["grpc-web"].bindConnection(session, {
    generation: session.generation,
    endpoint: status.connection.endpoint,
    authorization: null,
    backendId: backend.backendId,
    epoch: backend.epoch,
    limits: backend.limits,
    methods: backend.methods,
  });
  return session;
}
const value = (kind, field0) => ({ values: [{ nodes: [{ kind: { HarmosVariant: { case: kind, value: { field0 } } } }] }] });
(async () => {
  const session = await connection();
  const api = bench.ipc.api;
  if (phase === "start") {
    const suites = await api.catalogListSuites(session, { limit: 100 });
    for (const listed of suites.suites) {
      const suite = await api.catalogGetSuite(session, { suiteId: listed.suiteId });
      const template = suite.templates.find((item) => item.metadata.label === argument.template);
      if (!template) continue;
      const cycle = await api.executionCreateCycle(session, {
        metadata: { identifier: "bench.throughput." + argument.run, label: "Stream Throughput" },
        plan: {
          maxCases: 1,
          selections: [{
            suiteId: suite.identity.suiteId,
            templateId: template.templateId,
            parameters: [
              { key: "continuous", value: value("Bool", true) },
              { key: "sample_interval_s", value: value("Number", argument.step) },
            ],
          }],
        },
      });
      await api.executionRunCycle(session, { cycleId: cycle.cycleId, idempotencyKey: cycle.cycleId });
      return done({ cycleId: cycle.cycleId });
    }
    return fail("template not found");
  }
  if (phase === "unary") {
    const timings = [];
    const client = bench[argument.transport].api;
    for (let index = 0; index < argument.calls; index += 1) {
      const started = performance.now();
      await client.backendGetHealth(session, {});
      timings.push(performance.now() - started);
    }
    return done({ timings });
  }
  if (phase === "watch") {
    const stream = await bench[argument.transport].openNativePlotStream(session, [{
      plotId: "bench",
      stream: argument.stream,
      yKeys: ["voltage_v", "current_a"],
      xKey: "elapsed_s",
      plotType: "xy_line",
      maxPoints: argument.points,
      windowS: argument.window,
      scope: { test_cycle_id: argument.cycleId },
    }]);
    const frames = [];
    const deadline = performance.now() + argument.seconds * 1000;
    const iterator = stream[Symbol.asyncIterator]();
    while (performance.now() < deadline) {
      const next = await Promise.race([
        iterator.next(),
        new Promise((resolve) => setTimeout(() => resolve(null), Math.max(0, deadline - performance.now()))),
      ]);
      if (!next || next.done) break;
      const message = next.value;
      const series = message.payload.series ?? [];
      const last = series[0]?.points?.at(-1);
      frames.push({
        arrived: Date.now(),
        generated: Date.parse(message.payload.generated_at),
        kind: message.kind,
        delta: message.diagnostics?.delta ?? null,
        points: series.reduce((total, item) => total + (item.points?.length ?? 0), 0),
        raw: message.diagnostics?.raw_point_count ?? null,
        x: last ? Number(last.x) : null,
      });
    }
    await stream.cancel();
    // A window without new rows means the run was not producing, not that the
    // transport stalled; the run's own status says which.
    const cycle = await api.executionGetCycle(session, { cycleId: argument.cycleId });
    return done({ frames, status: cycle.status?.HarmosVariant?.case ?? null });
  }
  if (phase === "stop") {
    await api.executionStopCycle(session, { cycleId: argument.cycleId });
    return done({});
  }
  fail("unknown phase");
})().catch(fail);
`;

function percentile(values, fraction) {
  if (!values.length) return null;
  const sorted = [...values].sort((left, right) => left - right);
  return sorted[Math.min(sorted.length - 1, Math.floor(fraction * sorted.length))];
}

function summarize(frames, seconds) {
  const latencies = frames.filter((frame) => Number.isFinite(frame.generated)).map((frame) => frame.arrived - frame.generated);
  const xs = frames.map((frame) => frame.x).filter(Number.isFinite);
  // Rows are counted by how far the plotted sample axis advanced: each row is
  // one simulated step, so the advance divided by the step is rows delivered.
  const rows = xs.length > 1 ? Math.round((Math.max(...xs) - xs[0]) / STEP_S) : 0;
  const span = frames.length > 1 ? (frames.at(-1).arrived - frames[0].arrived) / 1000 : Number(seconds);
  const kinds = {};
  for (const frame of frames) kinds[frame.kind] = (kinds[frame.kind] ?? 0) + 1;
  return {
    kinds,
    sample: process.env.DOKIME_BENCH_DEBUG ? frames.slice(0, 4) : undefined,
    frames: frames.length,
    framesPerSecond: Number((frames.length / span).toFixed(1)),
    rows,
    rowsPerSecond: Math.round(rows / span),
    pointsDelivered: frames.reduce((total, frame) => total + frame.points, 0),
    latencyMs: {
      p50: percentile(latencies, 0.5),
      p95: percentile(latencies, 0.95),
      max: latencies.length ? Math.max(...latencies) : null,
    },
  };
}

/* === Entry Point === */

const work = await mkdtemp(join(targetRoot, "stream-throughput-"));
let backend;
let session;
try {
  const [ipc, web] = await Promise.all([bundle("ipc"), bundle("grpc-web")]);
  backend = await startBackend(work);
  session = await startSession(work, backend.endpoint);
  await until(
    "attached session",
    async () =>
      (await session.executeAsync(
        `const done = arguments[0];
         globalThis.__TAURI_INTERNALS__.invoke('desktop_session_status')
           .then((status) => done(status.connection.phase === 'connected'), () => done(false));`,
      )) === true,
    120000,
  );
  await session.execute(ipc);
  await session.execute(web);
  const call = async (phase, argument) => {
    const result = await session.executeAsync(PAGE, [phase, argument]);
    if (result?.error) throw new Error(`${phase}: ${result.error}`);
    return result;
  };
  const results = {
    stepS: STEP_S,
    seconds: Number(options.seconds),
    maxPoints: Number(options.points),
    windowS: Number(options.window),
    unary: {},
    watch: [],
  };
  for (const transport of ["ipc", "grpc-web"]) {
    const { timings } = await call("unary", { transport, calls: Number(options.calls) });
    results.unary[transport] = {
      calls: timings.length,
      p50Ms: Number(percentile(timings, 0.5).toFixed(2)),
      p95Ms: Number(percentile(timings, 0.95).toFixed(2)),
    };
  }
  for (let round = 0; round < Number(options.rounds); round += 1) {
    // Alternate the order, and give every window a fresh run of the same age:
    // the backend rebuilds each frame from the whole run, so an older run
    // produces slower frames whichever transport carries them.
    const order = round % 2 ? ["grpc-web", "ipc"] : ["ipc", "grpc-web"];
    for (const [index, transport] of order.entries()) {
      const run = round * 2 + index;
      const { cycleId } = await call("start", { template: TEMPLATE, step: STEP_S, run });
      await delay(2000);
      const { frames, status } = await call("watch", {
        transport,
        cycleId,
        stream: STREAM,
        seconds: Number(options.seconds),
        points: Number(options.points),
        window: Number(options.window),
      });
      await call("stop", { cycleId });
      results.watch.push({ round, transport, status, ...summarize(frames, options.seconds) });
    }
  }
  const report = JSON.stringify(results, null, 2);
  process.stdout.write(`${report}\n`);
  await writeFile(join(targetRoot, "stream-throughput.json"), `${report}\n`);
} finally {
  await session?.close();
  session?.driver.kill();
  backend?.child.stdin.end();
  if (backend) await Promise.race([once(backend.child, "exit"), delay(30000)]);
  backend?.child.kill();
  await backend?.log.close();
  if (!process.exitCode) await rm(work, { recursive: true, force: true });
}
