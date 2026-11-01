// ***
//
// Dokime
//
// SPDX-License-Identifier: Apache-2.0
//
// ***

// Real Native WebView Gate For Development And Installed Release Shells. No HTTP facade,
// no mocked domain IPC, no browser fixtures.
//
// The gate starts a throwaway backend with the OBC charger testbench plugin,
// attaches the desktop shell to it with `--endpoint`, and drives the redesigned
// routes -- `/cycles`, `/library`, `/cycles/$cycleId`, `/history`, `/settings`
// -- through the application itself. Elements are addressed by accessible name,
// role and stable copy; never by class names, which the design pass keeps
// moving.
//
// Journeys, each asserting on DOM text and roles and screenshotting
// into the evidence directory when it fails:
//
//   a. startup      the connection screen, then Cycles with empty groups
//   b. library      suite expansion, the template dialog, two templates added
//   c. cycle        the workspace: sidebar, run, prompts, focus, console, overflow
//   c'. execution-stop  a copy: Stop Execution stops one, its siblings finish
//   c''. containers  independent Active cycles, explicit sealing and export availability
//   d. cycles-home  the groups after a run, the Archived view, the filter box
//   e. history      rows, one execution, its checkpoint views, the Back chevron
//   f. settings     every section, the density switch, storage, plugins, shared secrets, diagnostics
//   f'. plugin-install  native bundle picker, direct installation, restart activation and catalog
//   f'. organization  native pack import, restart activation and invalid-pending retention
//   f'. remote-access  listener and TLS setup, one pairing file per client exported from the
//                      backend's Remote Access settings and imported in the desktop Backends
//                      table, restart, independent revocation and lab modes
//   Report-producing journeys finalize by saving and reopening their sealed execution reports.
//   h. touch        the same surfaces at touch density: collapsed rail, Executions sheet
//
// Windows Requires An Interactive Account, WebView2 And A Version-
// Matched Microsoft Edge Driver. --release Requires A Disposable Account And
// DOKIME_E2E_INSTALLER Selects The Native NSIS
// Artifact; DOKIME_E2E_{INSTALLER,SHELL,CLI}_SHA256 Bind Its Installed
// Executables To The Candidate. DOKIME_E2E_PLUGIN And DOKIME_E2E_PLUGIN_SHA256
// Select And Verify Its Prepared OBC Fixture.
// The Optional --junit Path Is Supplied By The Repository Evidence Runner.
//
// Two deliberate substitutions, both of the operating system rather than of the
// product:
//
//   * Report opens use an owned real browser/profile whose tabs close before
//     fixture cleanup. `xdg-open` is a no-op for unrelated file-manager opens.
//   * `desktop_save_document` opens a native GTK/portal save dialog no WebDriver
//     session can reach, which would hang the gate on Download Report. The gate
//     taps inline document saves at the Tauri IPC boundary. Scene-owned report
//     finalization drives the real native save picker and verified pull, then
//     retains the reopened report and its linked artifacts in `<evidence>/downloads`.
//     Every domain RPC still
//     crosses the real transport, and so does `desktop_save_diagnostics_bundle`,
//     which needs no picker: the throwaway home is given an xdg-user-dirs
//     Downloads folder and the shell writes the archive there itself.

/* === Imports === */

// Standard Library Imports

import assert from "node:assert/strict";

import { execFile, spawn } from "node:child_process";
import { createHash, randomUUID } from "node:crypto";
import { once } from "node:events";

import { createReadStream, existsSync } from "node:fs";

import { appendFile, copyFile, cp, mkdir, mkdtemp, open, readdir, readFile, realpath, rm, writeFile } from "node:fs/promises";

import net from "node:net";
import { tmpdir } from "node:os";

import { basename, dirname, join, resolve } from "node:path";

import { createInterface } from "node:readline";

import { setTimeout as delay } from "node:timers/promises";

import { fileURLToPath } from "node:url";

import { parseArgs, promisify } from "node:util";

// Third Party Imports

// Local Imports
import { call } from "./native-rpc.mjs";
import { ReportBrowser } from "./report-browser.ts";
import { traceSpan } from "./test-trace.mjs";

/* === Re-Exports === */

/* === Constants === */

const workspace = resolve(dirname(fileURLToPath(import.meta.url)), "../../..");

const { values: options } = parseArgs({
  options: {
    junit: { type: "string" },
    release: { type: "boolean", default: false },
    profile: { type: "string" },
    workflow: { type: "string" },
  },
});
const profile = options.profile ?? (options.release ? "release" : "dev");
assert.ok(["dev", "release"].includes(profile), "native desktop profile must be dev or release");
const architecture = { x64: "x86_64", arm64: "aarch64" }[process.arch] ?? process.arch;
const nativeTarget = `${process.platform === "win32" ? "windows" : "linux"}-${architecture}`;
const reportsRoot = join(workspace, "target/tests", profile, nativeTarget, "system-typescript");

/** The Single Build Stages The Development Candidates Together. */
const built = join(workspace, "target/build", profile, nativeTarget);
const executableSuffix = process.platform === "win32" ? ".exe" : "";
let backendBinary = process.env.DOKIME_E2E_CLI ?? join(built, `bin/dokime${executableSuffix}`);
let shellBinary = process.env.DOKIME_E2E_SHELL ?? join(built, `bin/dokime-gui${executableSuffix}`);
let cliBinary = process.env.DOKIME_E2E_CLI ?? join(built, `bin/dokime${executableSuffix}`);
const pluginBundle = process.env.DOKIME_E2E_PLUGIN ?? join(built, "fixtures/plugins/plugin-obc.dokime");

const hostBundle = join(built, "plugins/plugin-host.dokime");

const densities = {
  desktop: { density: "comfortable", height: 1000, width: 1600 },
  touch: { density: "touch", height: 1200, width: 1920 },
};

/** The suites and templates the OBC charger plugin registers, by rendered label. */
const PROTECTION_SUITE = "Charger Protection";

const PROTECTION_TEMPLATE = "Overcurrent Protection";

const STARTUP_SUITE = "Charger Start-Up";

const STARTUP_TEMPLATE = "Start-Up Sequence";

/** A cartesian list parameter, so chips and expressions have somewhere to go. */
const LIST_PARAMETER = "load_steps";

/** A plain number, so an edit has a Reset affordance to reveal. */
const NUMBER_PARAMETER = "frequency_hz";

const OPERATOR_NOTE = "Trace steady, no fault indicated.";

/** The startup template's question, which the OBC plugin raises to Warning. */
const STARTUP_PROMPT = "Inspect Powered Charger";

/** The WebDriver code points for the named keys this gate presses. */
const WEBDRIVER_KEYS = {
  Enter: "\uE007",
  Escape: "\uE00C",
  Tab: "\uE004",
  Control: "\uE009",
  Backspace: "\uE003",
  End: "\uE010",
};

/** Journey Filters Retain These Prerequisites Within Their Selected Workflow. */
const REQUIRED_JOURNEYS = new Set(["startup", "plugin-install", "library"]);

/** Independent Workflows Share Only Their Connection And Catalog Prerequisites. */
const workflows = {
  execution: {
    title: "Desktop Execution And Recovery",
    journeys: new Set([
      "startup",
      "plugin-install",
      "library",
      "active-cycle",
      "cycle",
      "touch",
      "execution-stop",
      "containers",
      "cycles-home",
      "history",
      "attachment",
      "restart",
    ]),
  },
  configuration: {
    title: "Desktop Configuration And Access",
    journeys: new Set([
      "owned-startup",
      "startup",
      "plugin-install",
      "idle",
      "settings",
      "distribution-identity",
      "organization",
      "remote-access",
    ]),
  },
};
assert.ok(!options.workflow || Object.hasOwn(workflows, options.workflow), `Unknown desktop workflow: ${options.workflow}`);
const workflow = workflows[options.workflow];

/** Opt out of real key presses when the driver cannot honour the Actions API. */
const syntheticKeys = process.env.DOKIME_E2E_SYNTHETIC_KEYS === "1";

/**
 * Which journeys to run, as a comma-separated list; every one by default.
 *
 * `startup`, `plugin-install` and `library` run when their workflow owns them:
 * they establish the connection and materialize the execution workflow's cycle.
 */
const selectedJourneys = new Set(
  (process.env.DOKIME_E2E_JOURNEYS ?? "")
    .split(",")
    .map((name) => name.trim())
    .filter(Boolean),
);

const releaseValidation = options.release;
const windows = process.platform === "win32";
let reportBrowser;
let nativePickerWorker;
let reportBrowserClosed = false;
const walkthroughStarted = performance.now();
const results = [];
const timings = [];
const finalizedReports = new Set();
const pendingReports = new Set();
let timingScene = "setup";
const failures = [];
const passedJourneys = [];

const notes = [];
const secrets = new Set();

/** What the prompt queue and its counts showed before each prompt was answered. */
const promptSightings = [];

/** What the cycle page said while an execution waited on input. */
const cycleInputSightings = [];

/** The journey whose WebView died, once one has: everything after it is moot. */
let webviewGone = null;

let session;

let driverProcess;

let backend;

let work;

let evidence;

let installation;

let installerAttempted = false;

/* === Types === */

/* === Classes === */

/* === Public Functions === */

/* === Private Functions === */

/* Process And Session Helpers */

async function unusedPort() {
  const socket = net.createServer();
  socket.listen(0, "127.0.0.1");
  await once(socket, "listening");
  const port = socket.address().port;
  await new Promise((done) => socket.close(done));
  return port;
}

function listening(endpoint) {
  const url = new URL(endpoint);
  return new Promise((done) => {
    const socket = net.connect(Number(url.port), url.hostname);
    socket.once("connect", () => {
      socket.destroy();
      done(true);
    });
    socket.once("error", () => done(false));
    socket.setTimeout(120000, () => {
      socket.destroy();
      done(false);
    });
  });
}

async function executeFile(command, args, options) {
  const category = command === cliBinary ? "cli" : command === "python" ? "fixture-preparation" : "native-process";
  return measured(category, basename(command), () => promisify(execFile)(command, args, options));
}

async function measured(category, name, run) {
  const started = performance.now();
  const scene = timingScene;
  try {
    return await run();
  } finally {
    timings.push({ scene, category, name, milliseconds: performance.now() - started });
  }
}

function until(description, check, timeout = 120000, interval = 25) {
  return traceSpan(description, "step", () => waitUntil(description, check, timeout, interval));
}

async function waitUntil(description, check, timeout, interval) {
  return measured("wait", description, async () => {
    const deadline = Date.now() + timeout;
    let last;
    while (Date.now() < deadline) {
      try {
        const value = await measured("step", description, check);
        if (value) return value;
      } catch (failure) {
        last = failure;
      }
      await measured("poll-delay", description, () => delay(interval));
    }
    throw new Error(`Timed out: ${description}${last ? ` (last error: ${last.message})` : ""}`);
  });
}

function xpathLiteral(value) {
  if (!value.includes("'")) return `'${value}'`;
  if (!value.includes('"')) return `"${value}"`;
  return `concat(${value
    .split("'")
    .map((part) => `'${part}'`)
    .join(', "\'", ')})`;
}

function normalize(value) {
  return value.replace(/\s+/g, " ").trim();
}

function redact(value) {
  let text = String(value);
  for (const secret of secrets) text = text.replaceAll(secret, "[REDACTED]");
  return text;
}

function hasText(body, expected) {
  return normalize(body).toLowerCase().includes(normalize(expected).toLowerCase());
}

function parameterGeometry() {
  return [...document.querySelectorAll('[data-testid="execution-parameters"] [data-parameter]')].map((row) => {
    const [label, value, unit] = [...row.children].map((node) => {
      const bounds = node.getBoundingClientRect();
      return {
        left: bounds.left,
        right: bounds.right,
        top: bounds.top,
        width: bounds.width,
        clipped: node.scrollWidth > node.clientWidth + 1,
      };
    });
    return { name: row.dataset.parameter, left: row.getBoundingClientRect().left, label, value, unit };
  });
}

function assertParameterGeometry(rows) {
  assert.ok(rows.length > 1, "The parameter grid must contain multiple values");
  for (const row of rows) {
    assert.ok(!row.label.clipped && !row.value.clipped && !row.unit.clipped, `${row.name}: parameter content must fit`);
    assert.ok(row.label.right <= row.value.left && row.value.right < row.unit.left, `${row.name}: columns must not overlap`);
    assert.ok(row.unit.left - row.value.right <= 12, `${row.name}: units must sit beside their values`);
    const peers = rows.filter((other) => Math.abs(other.left - row.left) < 1);
    for (const peer of peers) {
      assert.ok(Math.abs(peer.value.right - row.value.right) < 1, `${row.name}: figures must align within each column`);
      assert.ok(Math.abs(peer.unit.left - row.unit.left) < 1, `${row.name}: units must align within each column`);
    }
  }
}

async function checkParameterGeometry(session) {
  const rows = await until("mounted parameter grid", async () => {
    const rows = await session.execute(`return (${parameterGeometry.toString()})();`);
    return rows.length > 1 ? rows : null;
  });
  assertParameterGeometry(rows);
}

async function promptHoverLabels() {
  const frame = document.querySelector('[data-plot-id="obc.charger.startup.sequence.signals"]');
  const overlay = frame?.querySelector(".u-over");
  if (!overlay) return [];
  overlay.scrollIntoView({ block: "center", behavior: "instant" });
  const labels = new Set();
  const frameDrawn = () => new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(resolve)));
  await frameDrawn();
  const bounds = overlay.getBoundingClientRect();
  overlay.dispatchEvent(new MouseEvent("mouseenter", { clientX: bounds.left, clientY: bounds.top + bounds.height / 2 }));
  const last = Math.floor(bounds.width / 4);
  for (let index = 0; index <= last; index += 1) {
    const offset = (index % 2 === 0 ? index / 2 : last - (index - 1) / 2) * 4;
    overlay.dispatchEvent(
      new MouseEvent("mousemove", {
        bubbles: true,
        clientX: bounds.left + offset,
        clientY: bounds.top + bounds.height / 2,
      }),
    );
    await frameDrawn();
    const label = frame.querySelector("[role=tooltip]")?.textContent;
    if (label) labels.add(label);
    if ([...labels].some((label) => label.includes("Prompt asked:")) && [...labels].some((label) => label.includes("Prompt answered:")))
      break;
  }
  return [...labels];
}

function assertPromptHovers(labels) {
  assert.ok(
    labels.some((label) => label.includes(`Prompt asked: ${STARTUP_PROMPT}`)),
    `Asked marker must name its prompt: ${labels}`,
  );
  assert.ok(
    labels.some((label) => label.includes(`Prompt answered: ${STARTUP_PROMPT}`)),
    `Answered marker must name its prompt: ${labels}`,
  );
}

function histogramPresentation() {
  const plot = document.querySelector('[data-plot-id="obc.charger.startup.sequence.voltage_distribution"] section');
  if (!plot) return null;
  const bins = [...plot.querySelectorAll("button[aria-label]")].map((button) =>
    (button.getAttribute("aria-label").match(/[-+]?\d+(?:\.\d+)?/g) ?? []).map(Number),
  );
  const samples = Number.parseInt(plot.querySelector("strong + span")?.textContent ?? "", 10);
  return bins.length ? { bins, samples } : null;
}

function assertHistogram(presentation) {
  assert.ok(presentation, "The voltage distribution must render bins");
  assert.ok(Number.isInteger(presentation.samples) && presentation.samples > 0);
  assert.ok(
    presentation.bins.every(
      (bin) => bin.length === 3 && bin.every(Number.isFinite) && bin[0] <= bin[1] && Number.isInteger(bin[2]) && bin[2] >= 0,
    ),
    "Histogram bins expose numeric ranges and nonnegative counts",
  );
  assert.equal(
    presentation.bins.reduce((sum, bin) => sum + bin[2], 0),
    presentation.samples,
  );
  for (let index = 1; index < presentation.bins.length; index += 1) {
    assert.equal(presentation.bins[index - 1][1], presentation.bins[index][0]);
  }
}

async function checkStatusBounds(session) {
  const badges = await session.execute(`
    return [...document.querySelectorAll('main [data-slot=badge]')]
      .filter(node => !node.closest('[aria-hidden=true]') && node.getBoundingClientRect().width > 0)
      .map(node => {
        const bounds = node.getBoundingClientRect();
        const range = document.createRange();
        range.selectNodeContents(node);
        const text = range.getBoundingClientRect();
        return {label: node.innerText, clipped: text.left < bounds.left - 1 || text.right > bounds.right + 1
          || text.top < bounds.top - 1 || text.bottom > bounds.bottom + 1};
      });
  `);
  assert.ok(badges.length, "The mounted view must show status pills");
  assert.deepEqual(
    badges.filter((badge) => badge.clipped),
    [],
    "Status labels must fit inside their pills",
  );
}

async function startBackend(...args) {
  return measured("backend-launch", "startBackend", () => startBackendMeasured(...args));
}

async function startBackendMeasured(
  work,
  evidence,
  endpoint,
  label = "backend",
  launchPlugin = true,
  { dataRoot = join(work, "data") } = {},
) {
  for (const path of [backendBinary, cliBinary, hostBundle, ...(launchPlugin ? [pluginBundle] : [])]) {
    if (!existsSync(path)) throw new Error(`Missing backend artifact at ${path}. Use the same candidate's prepared artifacts.`);
  }
  const port = endpoint ? Number(new URL(endpoint).port) : await unusedPort();
  const log = await open(join(evidence, `${label}.log`), "w");
  const child = spawn(
    backendBinary,
    [
      "server",
      "--shutdown-stdin",
      "--plugin",
      hostBundle,
      ...(launchPlugin ? ["--plugin", pluginBundle] : []),
      "--port",
      String(port),
      "--output-root",
      dataRoot,
      "--application-root",
      join(work, "application"),
    ],
    {
      cwd: work,
      detached: !windows,
      windowsHide: true,
      env: {
        ...process.env,
        DOKIME_TEST_HUB_OPERATOR: "Gate Operator",
        ...(!windows ? { HOME: work } : {}),
      },
      stdio: ["pipe", log.fd, log.fd],
    },
  );
  child.on("error", (error) => {
    child.startupError = error;
  });
  backend = { child, endpoint: `http://127.0.0.1:${port}`, log };
  const readyStarted = Date.now();
  const readiness = { attempts: 0, ready: false };
  try {
    // Normal Startup Verifies The Selected Plugins Before Binding This Listener.
    await until(
      "backend listener ready",
      async () => {
        if (child.startupError) throw child.startupError;
        if (child.exitCode !== null) throw new Error(`Native backend exited during startup: ${child.exitCode}`);
        readiness.attempts += 1;
        readiness.ready = await listening(backend.endpoint);
        return readiness.ready;
      },
      180000,
    );
  } finally {
    await writeFile(join(evidence, `${label}-readiness.json`), JSON.stringify({ ...readiness, elapsedMs: Date.now() - readyStarted }));
  }
  return backend;
}

/**
 * One Native WebDriver Session Over One Attached Shell.
 *
 * Every helper here is user-like: navigate, read text, click a named control,
 * type into a named field. The session keeps one request in flight because
 * WebKitWebDriver 2.52 serializes commands per session and a concurrent pair
 * can leave one of them pending forever.
 */
async function startSession(...args) {
  return measured("gui-launch", "startSession", () => startSessionMeasured(...args));
}

async function startSessionMeasured(work, endpoint, evidence, distribution, { startLocal = true, downloadsRoot } = {}) {
  const home = join(work, "shell");
  const downloads = downloadsRoot ?? join(evidence, "downloads");
  const applicationRoot = distribution ? join(home, "data", distribution.identifier) : join(home, "application");
  const profilesFile = join(home, windows ? "data" : "config", "dokime", "backends.json");
  for (const directory of ["config", "data", "state", "cache", "bin"]) {
    await mkdir(join(home, directory), { recursive: true });
  }
  await mkdir(downloads, { recursive: true });
  if (!windows) {
    await writeFile(join(home, "bin", "xdg-open"), "#!/bin/sh\nexit 0\n", { mode: 0o755 });
    /* `desktop_save_diagnostics_bundle` resolves the Downloads folder through
       Tauri's path API, which reads xdg-user-dirs rather than the
       `XDG_DOWNLOAD_DIR` environment variable. Without this file the throwaway
       home has no Downloads folder and the command fails before it writes. */
    await writeFile(join(home, "config", "user-dirs.dirs"), `XDG_DOWNLOAD_DIR="${downloads}"\nXDG_DOCUMENTS_DIR="${downloads}"\n`);
  }

  const port = await unusedPort();
  const driver =
    process.env.DOKIME_E2E_DRIVER ?? (windows ? join(workspace, "target/tooling/desktop/msedgedriver.exe") : "WebKitWebDriver");
  if (windows && !process.env.DOKIME_E2E_DRIVER) {
    assert.ok(existsSync(driver), `Missing prepared Microsoft WebDriver at ${driver}; run mise run prepare:desktop`);
  }
  driverProcess = spawn(
    driver,
    ["--host=127.0.0.1", `--port=${port}`, ...(windows ? ["--verbose", `--log-path=${join(evidence, "webdriver.log")}`] : [])],
    {
      detached: !windows,
      windowsHide: true,
      env: {
        ...process.env,
        TAURI_WEBVIEW_AUTOMATION: "true",
        DOKIME_AUTOMATION_REPORT_OPENER: reportBrowser.endpoint,
        DOKIME_APPLICATION_ROOT: distribution ? "" : applicationRoot,
        DOKIME_PROFILES_FILE: profilesFile,
        ...(distribution ? { DOKIME_OUTPUT_ROOT: "", DOKIME_PLUGINS_DIR: "" } : {}),
        ...(windows
          ? {
              WEBVIEW2_USER_DATA_FOLDER: "",
              APPDATA: join(home, "data"),
              LOCALAPPDATA: join(home, "data"),
            }
          : {
              GDK_BACKEND: "x11",
              HOME: home,
              PATH: `${join(home, "bin")}:${process.env.PATH ?? ""}`,
              XDG_CACHE_HOME: join(home, "cache"),
              XDG_CONFIG_HOME: join(home, "config"),
              XDG_DATA_HOME: join(home, "data"),
              XDG_DOWNLOAD_DIR: downloads,
              XDG_STATE_HOME: join(home, "state"),
            }),
      },
      stdio: ["ignore", "ignore", windows ? "inherit" : "pipe"],
    },
  );
  const webdriver = driverProcess;
  let driverStderr = Buffer.alloc(0);
  webdriver.stderr?.on("data", (chunk) => {
    driverStderr = Buffer.concat([driverStderr, chunk]).subarray(-65536);
    process.stderr.write(chunk);
  });
  const base = `http://127.0.0.1:${port}`;

  async function request(method, path, body) {
    const response = await fetch(base + path, {
      method,
      headers: { "content-type": "application/json" },
      ...(body === undefined ? {} : { body: JSON.stringify(body) }),
      signal: AbortSignal.timeout(120000),
    }).catch((cause) => {
      throw new Error(`WebDriver ${method} ${path} failed: ${cause.message}`, { cause });
    });
    const result = (await response.json()).value;
    if (!response.ok || result?.error) {
      throw new Error(`WebDriver ${path}: ${result?.error ?? response.status}: ${result?.message ?? ""}`);
    }
    return result;
  }

  webdriver.on("error", (error) => {
    webdriver.startupError = error;
  });
  await until("native WebDriver startup", async () => {
    if (webdriver.startupError) throw webdriver.startupError;
    if (webdriver.exitCode !== null) throw new Error("Native WebDriver exited before startup");
    await request("GET", "/status");
    return true;
  });

  const created = await request("POST", "/session", {
    capabilities: {
      alwaysMatch: windows
        ? {
            browserName: "webview2",
            "ms:edgeChromium": true,
            "ms:edgeOptions": {
              binary: resolve(distribution?.shell ?? shellBinary),
              args: endpoint ? [`--endpoint=${endpoint}`] : [],
              excludeSwitches: ["enable-logging"],
            },
          }
        : {
            browserName: "wry",
            "webkitgtk:browserOptions": {
              args: endpoint ? [`--endpoint=${endpoint}`] : [],
              binary: resolve(distribution?.shell ?? shellBinary),
            },
          },
    },
  });

  const session = {
    closed: false,
    applicationRoot,
    downloads,
    profilesFile,
    id: created.sessionId,
    lastBody: "",
    process: webdriver,
  };
  let queue = Promise.resolve();
  session.request = (method, path, body) => {
    const queued = queue.then(() =>
      measured("webdriver", `${method} ${path}`, () => request(method, `/session/${session.id}${path}`, body)),
    );
    queue = queued.catch(() => undefined);
    return queued;
  };
  session.execute = (script, args = []) => session.request("POST", "/execute/sync", { args, script });
  session.executeAsync = (script, args = []) => session.request("POST", "/execute/async", { args, script });
  session.domRevision = () =>
    session.execute(`
    if (!globalThis.__dokimeDom) {
      const state = {revision: 0};
      new MutationObserver(() => state.revision++).observe(document, {attributes: true, childList: true, characterData: true, subtree: true});
      globalThis.__dokimeDom = state;
    }
    return globalThis.__dokimeDom.revision;
  `);
  session.waitForDom = (description, script, args = []) =>
    measured("dom-wait", description, async () => {
      const result = await session.executeAsync(
        `
      const args = arguments[0];
      const done = arguments[arguments.length - 1];
      const ready = () => {${script}};
      const observer = new MutationObserver(check);
      function check() {
        try {
          const value = ready();
          if (!value) return;
          observer.disconnect();
          done({value});
        } catch (error) {
          observer.disconnect();
          done({failure: String(error)});
        }
      }
      observer.observe(document, {attributes: true, childList: true, characterData: true, subtree: true});
      check();
    `,
        [args],
      );
      assert.equal(result.failure, undefined, result.failure);
      return result.value;
    });
  session.url = () => session.request("GET", "/url");
  session.location = async () => new URL(await session.url());
  session.text = async () => {
    session.lastBody = redact(await session.execute("return document.body.innerText"));
    return session.lastBody;
  };
  session.find = (using, value) => session.request("POST", "/element", { using, value });
  session.findAll = (using, value) => session.request("POST", "/elements", { using, value });
  session.textOf = (element) => session.execute("return arguments[0].innerText || arguments[0].textContent || '';", [element]);
  /**
   * Click as a pointer would, and only fall back when something covers the
   * control. A transient toast overlays the bottom of the page for a few
   * seconds, which is not a reason for the gate to fail the row beneath it.
   */
  session.click = async (element) => {
    const key = element["element-6066-11e4-a52e-4f735466cecf"];
    const settled = await session.executeAsync(
      `const element = arguments[0];
       const done = arguments[arguments.length - 1];
       const initial = element.getBoundingClientRect();
       if (initial.top < 0 || initial.left < 0 || initial.bottom > innerHeight || initial.right > innerWidth) {
         element.scrollIntoView({block: 'center', inline: 'center', behavior: 'instant'});
       }
       let previous = '';
       function frame() {
         if (!element.isConnected) return done(false);
         const rect = element.getBoundingClientRect();
         const position = [rect.x, rect.y, rect.width, rect.height].join(',');
         if (position === previous && rect.width > 0 && rect.height > 0) return done(true);
         previous = position;
         requestAnimationFrame(frame);
       }
       requestAnimationFrame(frame);`,
      [element],
    );
    assert.ok(settled, "The pointer target must remain mounted while scrolling settles");
    try {
      return await session.request("POST", `/element/${key}/click`, {});
    } catch (failure) {
      if (!/intercepted|not interactable|obscured/i.test(failure.message)) throw failure;
      return session.execute(
        `const element = arguments[0];
         if (element.disabled) throw new Error('Control is disabled');
         const modal = [...document.querySelectorAll('[role=dialog][data-state=open], [role=alertdialog][data-state=open]')].at(-1);
         if (modal && !modal.contains(element)) throw new Error('An open modal prevents this control from being clicked');
         element.scrollIntoView({block: 'center', inline: 'center'});
         element.focus();
         element.click();`,
        [element],
      );
    }
  };
  /** A control by its accessible name: its own text, its `aria-label`, its `title`. */
  session.named = (role, name, { enabled = true } = {}) => {
    const literal = xpathLiteral(name);
    const tag = role === "button" ? "button" : `*[@role=${xpathLiteral(role)}]`;
    const text = `(normalize-space(.) = ${literal} or .//span[normalize-space(.) = ${literal}] or @aria-label = ${literal} or @title = ${literal})`;
    return session.find("xpath", `//${tag}[${text}${enabled ? " and not(@disabled) and not(@aria-disabled='true')" : ""}]`);
  };
  session.clickNamed = async (role, name, options) => {
    const result = await until(`${role} ${name}`, async () => {
      const element = await session.named(role, name, options);
      try {
        return { value: await session.click(element) };
      } catch (failure) {
        if (/stale element reference|pointer target must remain mounted/i.test(failure.message)) return null;
        return { failure };
      }
    });
    if (result.failure) throw result.failure;
    return result.value;
  };
  session.openPicker = (element) =>
    session.execute(
      `
    const button = arguments[0];
    if (button.disabled) throw new Error('Native picker control is disabled');
    button.focus();
    button.click();
  `,
      [element],
    );
  session.clickCss = async (selector) => {
    const element = await until(selector, () => session.find("css selector", selector));
    await until(`stable pointer target ${selector}`, () =>
      session.executeAsync(
        `const element = arguments[0];
         const done = arguments[arguments.length - 1];
         element.scrollIntoView({block: 'center', inline: 'center', behavior: 'instant'});
         requestAnimationFrame(() => {
           const before = element.getBoundingClientRect();
           requestAnimationFrame(() => {
             const after = element.getBoundingClientRect();
             const target = document.elementFromPoint(after.left + after.width / 2, after.top + after.height / 2);
             done(element.isConnected && after.width > 0 && after.height > 0
               && Math.abs(before.top - after.top) < 1 && Math.abs(before.left - after.left) < 1
               && element.contains(target));
           });
         });`,
        [element],
      ),
    );
    await session.click(element);
  };
  session.exists = async (role, name) => {
    try {
      await session.named(role, name, { enabled: false });
      return true;
    } catch {
      return false;
    }
  };
  session.type = async (element, value) => {
    const key = element["element-6066-11e4-a52e-4f735466cecf"];
    await session.click(element);
    await session.request("POST", `/element/${key}/clear`, {});
    if (value.length) await session.request("POST", `/element/${key}/value`, { text: value, value: [...value] });
  };

  /**
   * Press a key the way a person does, through the WebDriver Actions API.
   *
   * A synthetic `KeyboardEvent` reaches a listener but is not an input event,
   * and a modal layer that closes on one does not always run the cleanup it
   * runs for a real press: Radix left `pointer-events: none` on the body after
   * a synthetic Escape, and the next dropdown in the journey never opened.
   * `DOKIME_E2E_SYNTHETIC_KEYS=1` restores the old path for a WebDriver that
   * cannot honour the Actions API.
   */
  session.key = async (key) => {
    const code = WEBDRIVER_KEYS[key] ?? key;
    if (!syntheticKeys) {
      try {
        await session.request("POST", "/actions", {
          actions: [
            {
              type: "key",
              id: "keyboard",
              actions: [
                { type: "keyDown", value: code },
                { type: "keyUp", value: code },
              ],
            },
          ],
        });
        return session.request("DELETE", "/actions");
      } catch (failure) {
        throw new Error(`WebDriver could not press ${key}: ${failure.message}`, {
          cause: failure,
        });
      }
    }
    return session.execute(
      `const target = document.activeElement ?? document.body;
       for (const type of ['keydown', 'keyup']) {
         target.dispatchEvent(new KeyboardEvent(type, {bubbles: true, key: arguments[0]}));
       }`,
      [key],
    );
  };
  session.resize = (width, height) => session.request("POST", "/window/rect", { height, width, x: 0, y: 0 });
  session.viewport = () => session.execute("return [window.innerWidth, window.innerHeight]");
  session.screenshot = async (label) => {
    await session.execute(`
      const style = document.createElement('style');
      style.id = 'dokime-secret-capture';
      style.textContent = '[aria-label="Generated Access Key"] { visibility: hidden !important; }';
      document.head.append(style);
    `);
    try {
      const bytes = await session.request("GET", "/screenshot");
      const target = join(evidence, `${label}.png`);
      await writeFile(target, Buffer.from(bytes, "base64"));
      return target;
    } finally {
      await session.execute("document.getElementById('dokime-secret-capture')?.remove();");
    }
  };
  /**
   * Structured failure capture, beside the screenshots.
   *
   * A step that times out on a DOM condition says only that the condition
   * stayed false. What the DOM actually held at that moment is the diagnosis,
   * so the step writes it here and the file outlives the run.
   */
  session.dump = async (label, value) => {
    const target = join(evidence, `${label}.json`);
    await writeFile(target, `${redact(JSON.stringify(value, null, 2))}\n`);
    return target;
  };
  /** Every layer that could hold the page, as the browser reports it. */
  session.modalLayers = () =>
    session.execute(`
      return {
        pathname: location.pathname,
        search: location.search,
        bodyPointerEvents: document.body.style.pointerEvents || '',
        bodyAriaHidden: document.body.getAttribute('aria-hidden'),
        dialogs: [...document.querySelectorAll('[role=dialog]')].map((node) => ({
          state: node.getAttribute('data-state'),
          label: node.getAttribute('aria-label') || node.getAttribute('aria-labelledby') || '',
          slot: node.getAttribute('data-slot') || '',
          className: node.className,
          text: (node.innerText || '').slice(0, 160),
          visible: Boolean(node.offsetParent) || getComputedStyle(node).display !== 'none',
        })),
        menus: document.querySelectorAll('[role=menu]').length,
        listboxes: document.querySelectorAll('[role=listbox]').length,
      };
    `);
  /** The connected shell, which a full reload has to reach again. */
  session.connected = () =>
    until("connected shell", () => session.execute("return Boolean(document.querySelector('nav[aria-label=Navigation]'));"), 180000);
  session.load = async (url) => {
    const previousOrigin = await session.execute("return performance.timeOrigin;");
    await session.request("POST", "/url", { url });
    await until(
      `load ${url}`,
      () =>
        session.execute(
          `return performance.timeOrigin !== arguments[0]
            && document.readyState === "complete"
            && (document.body?.innerText.length ?? 0) > 20;`,
          [previousOrigin],
        ),
      120000,
    );
  };
  /** Navigate Through Application Controls; Reload Only For Explicit Persistence Checks. */
  session.goto = async (path, { reload = false } = {}) => {
    const url = path.startsWith("file:") ? path : session.origin + path;
    const target = new URL(url);
    const destinations = { "/cycles": "Cycles", "/library": "Library", "/history": "History", "/notifications": "Notifications" };
    const sections = {
      backends: "Backends",
      appearance: "Appearance",
      organization: "Organization",
      plugins: "Plugins",
      secrets: "Secret Storage",
      remote: "Remote Access",
      diagnostics: "Diagnostics",
      about: "About",
    };
    const section = sections[/^\/settings\/([^/]+)$/.exec(target.pathname)?.[1]];
    const button = (navigation, label) =>
      session.find(
        "xpath",
        `//nav[@aria-label=${xpathLiteral(navigation)}]//button[normalize-space(.)=${xpathLiteral(label)} or @aria-label=${xpathLiteral(label)}]`,
      );
    let navigated = false;
    if (
      !reload &&
      (await session.execute("return Boolean(document.querySelector('nav[aria-label=Navigation]'));")) &&
      (!target.search || (target.search === "?view=cycle" && (await session.location()).pathname === target.pathname)) &&
      (await session.execute("return !document.querySelector('[role=dialog][data-state=open], [role=alertdialog][data-state=open]');"))
    ) {
      if (target.search === "?view=cycle") {
        const overview = await until("cycle overview navigation", () =>
          session.find("css selector", 'nav[aria-label="Navigation"] li[data-level="overview"] button[data-tree-select]'),
        );
        await session.click(overview);
        navigated = true;
      } else if (section) {
        if (!(await session.execute("return Boolean(document.querySelector('nav[aria-label=\"Settings Sections\"]'));"))) {
          await session.click(await button("Navigation", "Settings"));
          await until("settings navigation", () => session.find("css selector", 'nav[aria-label="Settings Sections"]'));
        }
        await session.click(await button("Settings Sections", section));
        navigated = true;
      } else if (destinations[target.pathname]) {
        await session.click(await button("Navigation", destinations[target.pathname]));
        navigated = true;
      }
    }
    if (!navigated) {
      await measured("navigation", "deep link", () => session.load(url));
      if (path.startsWith("file:")) return;
      if (!endpoint && startLocal) await session.clickNamed("button", "Start Local Backend");
    }
    await session.connected();
    await until(`route ${path}`, async () => (await session.location()).pathname === target.pathname, 120000);
    await installSaveTap(session);
  };
  session.close = async () => {
    if (session.closed) return;
    const linuxProcesses = async (known = []) => {
      if (windows) return null;
      const pending = [...new Set([webdriver.pid, ...known])];
      const processes = [];
      for (let index = 0; index < pending.length && index < 64; index += 1) {
        const pid = pending[index];
        const status = await readFile(`/proc/${pid}/stat`, "utf8").catch(() => null);
        processes.push({ pid, status: status?.trim() ?? null });
        const children = await readFile(`/proc/${pid}/task/${pid}/children`, "utf8").catch(() => "");
        for (const child of children.trim().split(/\s+/).filter(Boolean).map(Number)) {
          if (!pending.includes(child)) pending.push(child);
        }
      }
      return processes;
    };
    const processesBefore = await linuxProcesses().catch((error) => ({ error: String(error) }));
    const ownedShells = Array.isArray(processesBefore)
      ? processesBefore.filter(({ pid, status }) => status?.startsWith(`${pid} (${basename(shellBinary)}) `))
      : [];
    try {
      let closeReplyLost = false;
      try {
        await session.execute(`
          globalThis.__TAURI_INTERNALS__.invoke('plugin:window|close', {label: 'main'});
        `);
      } catch (error) {
        const closedBeforeReply = windows
          ? /target frame detached|invalid session id|disconnected/i.test(error.message)
          : ownedShells.length && /Session terminated without a reply/i.test(error.message);
        if (!closedBeforeReply) throw error;
        closeReplyLost = true;
      }
      await until(
        "native window completes orderly close",
        async () => {
          let closed;
          try {
            closed = (await session.request("GET", "/window/handles")).length === 0;
          } catch (error) {
            if (!/invalid session id|disconnected|connection refused/i.test(error.message)) throw error;
            closed = true;
          }
          if (!closed || !closeReplyLost) return closed;
          for (const { pid } of ownedShells) {
            try {
              await readFile(`/proc/${pid}/stat`, "utf8");
              return false;
            } catch (error) {
              if (error.code !== "ENOENT") throw error;
            }
          }
          return true;
        },
        40000,
      );
    } catch (failure) {
      const label = `failure-native-close-${Date.now()}`;
      const processesAfter = await linuxProcesses(Array.isArray(processesBefore) ? processesBefore.map(({ pid }) => pid) : []).catch(
        (error) => ({ error: String(error) }),
      );
      const linuxDriver = windows
        ? null
        : {
            pid: webdriver.pid,
            exitCode: webdriver.exitCode,
            signalCode: webdriver.signalCode,
            stderr: driverStderr.toString("utf8"),
            processesBefore,
            processesAfter,
          };
      const captures = await Promise.allSettled([
        session.screenshot(label),
        session.text().then((body) => writeFile(join(evidence, `${label}.txt`), redact(`${failure.stack ?? failure.message}\n\n${body}`))),
        session
          .executeAsync(`
          const done = arguments[arguments.length - 1];
          globalThis.__TAURI_INTERNALS__.invoke('desktop_session_status')
            .then(done, (error) => done({error: String(error)}));
        `)
          .then(async (status) => ({
            status,
            listening: status.connection?.endpoint ? await listening(status.connection.endpoint) : null,
          })),
        drainInvokeLedger(session),
        windows
          ? executeFile(
              "powershell.exe",
              [
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                `
              $ErrorActionPreference = 'Stop'
              Add-Type -AssemblyName UIAutomationClient
              Add-Type -AssemblyName UIAutomationTypes
              $providers = Add-Type -AssemblyName UIAutomationClientsideProviders -PassThru
              [System.Windows.Automation.ClientSettings]::RegisterClientSideProviderAssembly($providers[0].Assembly.GetName())
              $processes = @(Get-CimInstance Win32_Process)
              $owned = [System.Collections.Generic.HashSet[uint32]]::new()
              [void]$owned.Add([uint32]$env:DOKIME_CLOSE_DRIVER)
              do {
                $added = $false
                foreach ($candidate in $processes) {
                  if ($owned.Contains([uint32]$candidate.ParentProcessId)) {
                    $added = $owned.Add([uint32]$candidate.ProcessId) -or $added
                  }
                }
              } while ($added)
              $root = [System.Windows.Automation.AutomationElement]::RootElement
              $windows = @(foreach ($candidate in $processes) {
                if (!$owned.Contains([uint32]$candidate.ProcessId) -or $candidate.ExecutablePath -ne $env:DOKIME_CLOSE_SHELL) { continue }
                $condition = [System.Windows.Automation.PropertyCondition]::new([System.Windows.Automation.AutomationElement]::ProcessIdProperty, [int]$candidate.ProcessId)
                foreach ($window in $root.FindAll([System.Windows.Automation.TreeScope]::Children, $condition)) {
                  $text = [System.Windows.Automation.PropertyCondition]::new([System.Windows.Automation.AutomationElement]::ControlTypeProperty, [System.Windows.Automation.ControlType]::Text)
                  [ordered]@{
                    process = $candidate.ProcessId
                    handle = $window.Current.NativeWindowHandle
                    title = $window.Current.Name
                    text = @($window.FindAll([System.Windows.Automation.TreeScope]::Descendants, $text) | ForEach-Object { $_.Current.Name })
                  }
                }
              })
              ConvertTo-Json -InputObject $windows -Depth 4 -Compress
            `,
              ],
              {
                windowsHide: true,
                timeout: 120000,
                env: { ...process.env, DOKIME_CLOSE_DRIVER: String(webdriver.pid), DOKIME_CLOSE_SHELL: resolve(shellBinary) },
              },
            ).then((result) => JSON.parse(result.stdout))
          : Promise.resolve(null),
      ]);
      const names = ["screenshot", "text", "session", "invocations", "nativeWindows"];
      const diagnostics = Object.fromEntries(
        captures.map((capture, index) => [
          names[index],
          capture.status === "fulfilled" ? capture.value : { error: String(capture.reason) },
        ]),
      );
      await writeFile(
        join(evidence, `${label}.json`),
        redact(JSON.stringify({ error: failure.message, ...diagnostics, linuxDriver }, null, 2)),
      ).catch((error) => process.stderr.write(`Native close evidence failed: ${error.message}\n`));
      throw failure;
    } finally {
      session.closed = true;
    }
  };

  const initial = await until("main native window", async () => {
    const handles = await session.request("GET", "/window/handles");
    if (!handles.length) return false;
    await session.request("POST", "/window", { handle: handles[0] });
    const location = await session.location();
    return location.href === "about:blank" ? null : location;
  });
  const embedded =
    (initial.protocol === "tauri:" && initial.hostname === "localhost") ||
    (["http:", "https:"].includes(initial.protocol) && initial.hostname === "tauri.localhost");
  assert.ok(embedded, `the packaged shell must render embedded assets, received ${initial.href}`);
  session.origin = `${initial.protocol}//${initial.host}`;
  if (!endpoint && startLocal) await session.clickNamed("button", "Start Local Backend");
  return session;
}

/** Size the native window until the WebView reports the exact target viewport. */
async function sizeViewport(session, target) {
  await session.connected();
  const restored = await session.executeAsync(`
    const done = arguments[arguments.length - 1];
    const invoke = globalThis.__TAURI_INTERNALS__.invoke;
    invoke('plugin:window|is_maximized', {label: 'main'}).then(async (maximized) => {
      if (maximized) await invoke('plugin:window|internal_toggle_maximize', {label: 'main'});
      done({maximized});
    }).catch((error) => done({error: String(error)}));
  `);
  assert.equal(restored.error, undefined, "The native titlebar action must restore the maximized window before sizing");
  await until("the native window restored before sizing", () =>
    session.executeAsync(`
      const done = arguments[arguments.length - 1];
      globalThis.__TAURI_INTERNALS__.invoke('plugin:window|is_maximized', {label: 'main'})
        .then((value) => done(!value), () => done(false));
    `),
  );
  let frame = { height: target.height, width: target.width };
  const measurements = [];
  for (let attempt = 0; attempt < 8; attempt += 1) {
    assert.ok(Number.isSafeInteger(frame.width) && frame.width > 0, `Invalid frame width after ${JSON.stringify(measurements)}`);
    assert.ok(Number.isSafeInteger(frame.height) && frame.height > 0, `Invalid frame height after ${JSON.stringify(measurements)}`);
    const resized = await session.resize(frame.width, frame.height);
    await session.executeAsync("const done = arguments[arguments.length - 1]; requestAnimationFrame(() => requestAnimationFrame(done));");
    const [width, height] = await session.viewport();
    measurements.push({ requested: frame, resized, viewport: { width, height } });
    await writeFile(
      join(evidence, `viewport-${target.width}x${target.height}.json`),
      JSON.stringify({ target, restored, measurements }, null, 2),
    );
    if (width === target.width && height === target.height) return;
    frame = {
      height: resized.height + (target.height - height),
      width: resized.width + (target.width - width),
    };
  }
  throw new Error(`The native window did not reach its required viewport: ${JSON.stringify({ target, measurements })}`);
}

/* The Save Picker Substitution */

/**
 * Tap the save command so the gate plays the operating system's picker.
 *
 * `desktop_save_document` blocks on a native dialog WebDriver cannot see. The
 * tap records inline documents the application produced. Report saves use
 * the real native verified pull with its open action instead of the picker;
 * the harness copies that verified cache file into its Downloads directory.
 * Every domain RPC still goes to the real backend.
 *
 * The tap is on `fetch`, not on `invoke`, because `invoke` cannot be reached.
 * Tauri's bootstrap defines both `window.__TAURI_INTERNALS__` and its `invoke`
 * as `writable: false, configurable: false`, so assignment fails silently and
 * `Object.defineProperty` throws on both; the previous shim never installed
 * and every save this gate ever made reached a real GTK dialog that never
 * returns, which is what hung the report save in every run. One layer down,
 * `ipc-protocol.js` sends each command as `POST` to
 * `convertFileSrc(cmd, 'ipc')` and reads the reply's `Tauri-Response` header,
 * and `window.fetch` is an ordinary writable property. So the tap answers that
 * one request itself, exactly as the shell would have.
 *
 * `desktop_save_diagnostics_bundle` is deliberately not tapped: it opens no
 * dialog, writes into the operating system's Downloads folder itself and
 * answers with the path, so the gate lets it run and reads the folder.
 */
async function installSaveTap(session) {
  await until("Tauri IPC bridge", () => session.execute("return typeof globalThis.__TAURI_INTERNALS__?.invoke === 'function'"), 120000);
  const state = await session.execute(`
    globalThis.__dokimeSavedDocuments = globalThis.__dokimeSavedDocuments ?? [];
    globalThis.__dokimeInvokes = globalThis.__dokimeInvokes ?? [];
    globalThis.__dokimeStreams = globalThis.__dokimeStreams ?? [];
    globalThis.__dokimeStreamReads = globalThis.__dokimeStreamReads ?? 0;
    globalThis.__dokimeToasts = globalThis.__dokimeToasts ?? [];
    if (!globalThis.__dokimeToastObserver) {
      let previous = '';
      const observe = () => {
        const text = [...document.querySelectorAll('[data-sonner-toast]')].map(toast => toast.innerText).join(' | ').slice(0, 1000);
        if (text && text !== previous) {
          const timeline = globalThis.__dokimeToasts;
          if (timeline.length >= 32) timeline.shift();
          timeline.push({at: Date.now(), text});
        }
        previous = text;
      };
      globalThis.__dokimeToastObserver = new MutationObserver(observe);
      globalThis.__dokimeToastObserver.observe(document.body, {childList: true, subtree: true, characterData: true});
      observe();
    }
    if (!globalThis.fetch.__dokimeTap) {
      const original = globalThis.fetch.bind(globalThis);
      const saved = globalThis.__dokimeSavedDocuments;
      /* One bounded line per command the application asked the shell for, with
         how long it took and whether it ever answered. A call that never
         answers stays 'pending' here, which is the difference between a slow
         backend and a stalled one. */
      const ledger = globalThis.__dokimeInvokes;
      const shim = function (input, init) {
        const url = String(typeof input === 'string' ? input : (input && input.url) || '');
        const command = decodeURIComponent(url.split('/').pop() || '');
        /* The WebView reaches the backend only through the IPC bridge; any
           other request is a regression this gate names rather than serves. */
        if (!url.includes('ipc')) {
          if (/^(https?|wss?):/i.test(url)) globalThis.__dokimeNetwork = (globalThis.__dokimeNetwork ?? 0) + 1;
          return original(input, init);
        }
        /* Each bridge stream delivers on its own Tauri channel. Wrapping that
           channel's registered callback counts what the application actually
           received, frame by frame, before its transport decodes it. */
        if (command === 'rpc_stream_open') {
          const headers = (init && init.headers) || {};
          const header = (name) => (typeof headers.get === 'function' ? headers.get(name) : headers[name]);
          const channel = Number(String(header('dokime-rpc-channel') || '').split(':')[1]);
          const callbacks = globalThis.__TAURI_INTERNALS__.callbacks;
          const deliver = callbacks && callbacks.get(channel);
          if (deliver) {
            const streams = globalThis.__dokimeStreams;
            if (streams.length >= 128) streams.shift();
            const stream = { path: header('dokime-rpc-path'), reads: 0, bytes: 0, done: false };
            streams.push(stream);
            const open = streams.filter((entry) => !entry.done).length;
            globalThis.__dokimeStreamPeak = Math.max(globalThis.__dokimeStreamPeak ?? 0, open);
            callbacks.set(channel, (data) => {
              const frame = data && data.message instanceof ArrayBuffer ? new Uint8Array(data.message) : null;
              if (frame && frame.length > 8) {
                if (frame[8] === 0) {
                  stream.reads += 1;
                  stream.bytes += frame.length - 9;
                  globalThis.__dokimeStreamReads += 1;
                } else stream.done = true;
              }
              if (data && data.end) stream.done = true;
              return deliver(data);
            });
          }
        }
        if (ledger.length >= 2000) ledger.shift();
        const entry = { command: command, at: Date.now(), state: 'pending' };
        if (command === 'rpc' || command === 'rpc_stream_open') {
          const headers = (init && init.headers) || {};
          entry.path = typeof headers.get === 'function' ? headers.get('dokime-rpc-path') : headers['dokime-rpc-path'];
        }
        if (command === 'desktop_plugin_install_file') {
          const payload = String(init && init.body);
          entry.keys = Object.keys(JSON.parse(payload));
          entry.size = payload.length;
        }
        ledger.push(entry);
        if (command === 'desktop_save_document') {
          let document = null;
          try {
            document = JSON.parse(String(init && init.body)).document;
          } catch (failure) {
            entry.state = 'rejected';
            entry.error = 'unreadable save payload: ' + String(failure);
            return original(input, init);
          }
          saved.push({
            bytesBase64: document.bytesBase64,
            filename: document.filename,
            kind: document.kind,
          });
          entry.state = 'tapped';
          entry.ms = 0;
          /* The shell answers with the path the operator picked; the gate names
             one that was never written, and the open that follows is tapped too. */
          return Promise.resolve(
            new Response(JSON.stringify('/dokime-gate/saved/' + document.filename), {
              headers: { 'content-type': 'application/json', 'Tauri-Response': 'ok' },
              status: 200,
            }),
          );
        }
        if (command === 'desktop_open_path') {
          globalThis.__dokimeOpenedPaths = globalThis.__dokimeOpenedPaths ?? [];
          try {
            globalThis.__dokimeOpenedPaths.push(JSON.parse(String(init && init.body)).path);
          } catch (failure) {
            globalThis.__dokimeOpenedPaths.push('unreadable open payload: ' + String(failure));
          }
          entry.state = 'tapped';
          entry.ms = 0;
          return Promise.resolve(
            new Response('null', {
              headers: { 'content-type': 'application/json', 'Tauri-Response': 'ok' },
              status: 200,
            }),
          );
        }
        return original(input, init).then(
          function (value) {
            entry.nativeResponse = value.headers.get('Tauri-Response');
            entry.httpStatus = value.status;
            entry.state = entry.nativeResponse === 'error' ? 'rejected' : 'settled';
            entry.ms = Date.now() - entry.at;
            if ((entry.path?.endsWith('/StopExecution') || command === 'desktop_plugin_install_file') && entry.nativeResponse === 'error') {
              return value.clone().text().then(
                error => { entry.error = error.slice(0, 1024); return value; },
                error => { entry.error = 'Native error capture failed: ' + String(error); return value; },
              );
            }
            if (command === 'desktop_save_diagnostics_bundle' && value.headers.get('Tauri-Response') === 'ok') {
              return value.clone().json().then((path) => {
                globalThis.__dokimeDiagnosticsPath = path;
                return value;
              });
            }
            return value;
          },
          function (error) {
            entry.state = 'rejected';
            entry.ms = Date.now() - entry.at;
            entry.error = String(error);
            throw error;
          },
        );
      };
      shim.__dokimeTap = true;
      globalThis.fetch = shim;
    }
    return {
      installed: globalThis.fetch.__dokimeTap === true,
      reason: 'window.fetch did not keep the shim',
    };
  `);
  assert.ok(state?.installed, `The save tap did not take on window.fetch (${JSON.stringify(state)})`);
}

/**
 * The invoke ledger, summarized in the page rather than shipped out of it.
 *
 * One row per command with its call count, its slowest answer and how many
 * calls never answered at all. A journey that waits on something that never
 * resolves leaves a `pending` row here, which is the difference between a slow
 * backend and a stalled one. The summary is computed in the WebView because a
 * thousand rows across the WebDriver boundary is its own failure mode.
 */
async function drainInvokeLedger(session) {
  return session
    .execute(`
      const now = Date.now();
      const entries = globalThis.__dokimeInvokes ?? [];
      const rows = new Map();
      for (const entry of entries) {
        const key = JSON.stringify([entry.command, entry.path ?? null]);
        const row = rows.get(key) ?? {
          command: entry.command,
          path: entry.path ?? null,
          calls: 0,
          pending: 0,
          rejected: 0,
          slowestMs: 0,
        };
        row.calls += 1;
        if (entry.state === 'pending') {
          row.pending += 1;
          row.waitingMs = Math.max(row.waitingMs ?? 0, now - entry.at);
        }
        if (entry.state === 'rejected') {
          row.rejected += 1;
          if (entry.error) row.lastError = String(entry.error).slice(0, 300);
        }
        if (entry.nativeResponse) row.lastNativeResponse = entry.nativeResponse;
        row.slowestMs = Math.max(row.slowestMs, entry.ms ?? 0);
        rows.set(key, row);
      }
      return {
        recorded: entries.length,
        stops: entries.filter(entry => entry.path?.endsWith('/StopExecution')).slice(-8).map(entry => ({
          path: entry.path, at: entry.at, state: entry.state, ms: entry.ms,
          nativeResponse: entry.nativeResponse, httpStatus: entry.httpStatus, error: entry.error,
        })),
        toasts: globalThis.__dokimeToasts ?? [],
        streams: globalThis.__dokimeStreams ?? [],
        peakStreams: globalThis.__dokimeStreamPeak ?? 0,
        network: globalThis.__dokimeNetwork ?? 0,
        commands: [...rows.values()].sort((left, right) => right.calls - left.calls),
      };
    `)
    .then((summary) => JSON.parse(redact(JSON.stringify(summary))))
    .catch((failure) => ({ error: redact(String(failure)) }));
}

/* Shared Interactions */

async function startupPromptOpen(session) {
  return session.execute(
    `return [...document.querySelectorAll('ol[aria-label="Pending Prompts"] li[data-prompt-state="open"] #active-prompt-title')]
      .some((title) => title.textContent.trim() === arguments[0]);`,
    [STARTUP_PROMPT],
  );
}

/** Set the density through Appearance, which is the only honest switch. */
async function setDensity(session, variant) {
  await session.goto("/settings/appearance");
  await until("appearance section", async () => hasText(await session.text(), "Density"), 120000);
  await selectDensity(session, variant.density);
  await until(
    `density ${variant.density}`,
    async () => (await session.execute("return document.documentElement.dataset.density")) === variant.density,
    120000,
  );
  await sizeViewport(session, variant);
}

async function selectDensity(session, density) {
  const control = await until("local density preference", () => session.find("css selector", 'select[id="local-appearance.density"]'));
  await session.execute(
    `
    const select = arguments[0], value = arguments[1];
    if (![...select.options].some(option => option.value === value)) throw new Error('Density option is absent');
    select.value = value;
    select.dispatchEvent(new Event('change', {bubbles: true}));
  `,
    [control, density],
  );
}

/** Dismiss Toasts Through Their Controls Before The Next Pointer Target. */
async function settle(session) {
  await session.execute(`
    for (const close of document.querySelectorAll('[data-sonner-toast] [data-close-button]')) close.click();
  `);
  await session.waitForDom(
    "toast dismissal",
    `
    return [...document.querySelectorAll('[data-sonner-toast]')].every(toast =>
      toast.dataset.visible === 'false' || getComputedStyle(toast).pointerEvents === 'none' || !toast.getClientRects().length);
  `,
  );
}

/**
 * Answer every prompt the open cycle is waiting on, where it lives.
 *
 * A prompt has one home, the execution that asked it: the cycle page only
 * names the executions waiting on input and links to them, so a link found
 * there is followed first. The queue on the execution page holds the oldest
 * prompt open under the Control card; a decision prompt answers with Yes.
 */
async function answerPendingPrompts(session) {
  const selection = await session.execute(`
    if (document.querySelector('ol[aria-label="Pending Prompts"]')) return null;
    const attention = document.querySelector('[data-testid="navigation-prompt-count"]');
    const executionId = new URL(location.href).searchParams.get('executionId');
    return Number(attention?.textContent) > 0 && executionId && /^\\/cycles\\/[^/]+$/.test(location.pathname)
      ? {cycle: location.pathname, executionId} : null;
  `);
  if (selection) {
    await session.clickCss('nav[aria-label="Navigation"] li[data-level="overview"] button[data-tree-select]');
    await until("the selected cycle's prompt overview", async () => {
      const location = await session.location();
      return location.pathname === selection.cycle && location.searchParams.get("view") === "cycle";
    });
  }
  try {
    /* The cycle page answers nothing an execution asked: it names where. */
    const cycleSighting = await session.execute(`
    const needed = document.querySelector('[data-input-needed]');
    if (!needed) return null;
    return {
      text: needed.innerText,
      respond: [...document.querySelectorAll('button')].some((button) => button.innerText.trim() === 'Respond'),
      responsePanels: document.querySelectorAll('[aria-labelledby="active-prompt-title"]').length,
    };
  `);
    if (cycleSighting) {
      cycleInputSightings.push(cycleSighting);
      await session.execute(`
      const link = [...document.querySelectorAll('[data-input-needed] button')].find((button) => /^Answer On #/.test(button.innerText.trim()));
      link?.click();
    `);
      await session.waitForDom(
        "execution prompt queue",
        `
        return Boolean(document.querySelector('ol[aria-label="Pending Prompts"] li[data-prompt-state="open"]'));
      `,
      );
    }
    /* The queue is gone the moment its last prompt is answered, so what it
     showed is read here, before answering, and asserted once the run is over. */
    const observe = () =>
      session.execute(`
    const queue = document.querySelector('ol[aria-label="Pending Prompts"]');
    if (!queue) return null;
    const cards = [...queue.querySelectorAll('li[data-prompt-level]')];
    const control = document.querySelector('[data-execution-section="Control"]');
    const bar = control?.querySelector('[role=progressbar]');
    const row = bar?.parentElement;
    const tracks = row ? [...row.children].slice(0, 4).map(node => {
      const bounds = node.getBoundingClientRect();
      return {left: bounds.left, right: bounds.right, center: (bounds.top + bounds.bottom) / 2};
    }) : [];
    return {
      progress: bar ? {tracks, count: bar.nextElementSibling?.textContent,
        radius: parseFloat(getComputedStyle(bar).borderRadius), height: bar.getBoundingClientRect().height} : null,
      executionId: new URL(location.href).searchParams.get('executionId'),
      cards: cards.map((card) => ({
        level: card.dataset.promptLevel,
        state: card.dataset.promptState,
        text: card.innerText,
        top: card.getBoundingClientRect().top,
      })),
      queueTop: queue.closest('section')?.getBoundingClientRect().top ?? queue.getBoundingClientRect().top,
      controlBottom: control?.getBoundingClientRect().bottom ?? null,
      header: queue.closest('section')?.querySelector('header')?.innerText ?? null,
      navigation:
        document.querySelector('[data-testid="navigation-prompt-count"]')?.innerText ?? null,
    };
  `);
    let sighting = await observe();
    if (sighting?.cards.some((card) => hasText(card.text, STARTUP_PROMPT)) && !sighting.progress) {
      sighting = await until("the startup prompt's live progress", async () => {
        const current = await observe();
        return current?.progress ? current : null;
      });
    }
    if (sighting) promptSightings.push(sighting);
    const pending = sighting?.cards.length ?? 0;
    if (!pending) return 0;
    await session.execute(
      `const note = arguments[0];
     const panel = document.querySelector('li[data-prompt-state="open"] [aria-labelledby="active-prompt-title"]') ?? document.body;
     const setter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value').set;
     for (const input of panel.querySelectorAll('input[type="text"], input[type="number"]')) {
       if (input.value) continue;
       setter.call(input, input.type === 'number' ? '1' : note);
       input.dispatchEvent(new Event('input', {bubbles: true}));
       input.dispatchEvent(new Event('change', {bubbles: true}));
     }
     for (const box of panel.querySelectorAll('[role="checkbox"], [role="switch"]')) {
       if (box.getAttribute('aria-checked') !== 'true') box.click();
     }`,
      [OPERATOR_NOTE],
    );
    await session.waitForDom(
      "prompt response ready",
      `
      const panel = document.querySelector('li[data-prompt-state="open"] [aria-labelledby="active-prompt-title"]');
      return panel && [...panel.querySelectorAll('button')].some(button =>
        /^(Yes|Submit|Confirm|Accept|Continue)$/.test(button.innerText.trim()) && !button.disabled);
    `,
    );
    const submitted = await session.execute(`
    const panel = document.querySelector('li[data-prompt-state="open"] [aria-labelledby="active-prompt-title"]');
    if (!panel) return false;
    const submit = [...panel.querySelectorAll('button')].find((button) =>
      /^(Yes|Submit|Confirm|Accept|Continue)$/.test(button.innerText.trim()),
    );
    if (!submit) return false;
    const id = panel.closest("[data-prompt-id]").dataset.promptId;
    submit.click();
    return id;
  `);
    if (submitted)
      await session.waitForDom(
        "prompt response acknowledged",
        `
      return ![...document.querySelectorAll('[data-prompt-id]')].some(card => card.dataset.promptId === args[0]);
    `,
        [submitted],
      );
    return pending;
  } finally {
    if (selection) {
      await session.execute(
        `
        const row = [...document.querySelectorAll('nav[aria-label="Navigation"] li[data-execution-id]')]
          .find((candidate) => candidate.dataset.executionId === arguments[0]);
        if (!row) throw new Error('The measured execution disappeared while answering another prompt');
        row.querySelector('button[data-tree-select]').click();
      `,
        [selection.executionId],
      );
      await until("the restored measured execution", async () => {
        const location = await session.location();
        return location.pathname === selection.cycle && location.searchParams.get("executionId") === selection.executionId;
      });
    }
  }
}

/** Wait for a condition while answering whatever prompts appear meanwhile. */
async function runUntil(...args) {
  return measured("execution-wait", args[1], () => runUntilMeasured(...args));
}

async function runUntilMeasured(session, description, check, timeout) {
  const deadline = Date.now() + timeout;
  /* A run that never starts looks exactly like a run that is slow, and the
     difference is in the toast: the workspace reports every command's own
     outcome there, including the backend's refusal text. Carrying the last
     announcement into the timeout is what turns five silent minutes into the
     reason. */
  let announcement = "";
  while (Date.now() < deadline) {
    const revision = await session.domRevision();
    if (await check()) return;
    announcement = (await toastText(session).catch(() => "")) || announcement;
    await answerPendingPrompts(session);
    await session.waitForDom(description, "return globalThis.__dokimeDom.revision !== args[0];", [revision]);
  }
  throw new Error(`Timed out: ${description}. The application said: ${announcement || "nothing at all"}`);
}

/* Journeys */

/**
 * Record a product defect without abandoning the rest of the journey.
 *
 * A blocker in one control should still leave the surfaces behind it covered,
 * so the gate states the defect, takes the route an operator would have to take
 * instead, and still exits non-zero.
 */
function defect(name, reason) {
  failures.push({ kind: "defect", name, reason });
  process.stdout.write(`   DEFECT ${name}: ${reason}\n`);
}

/**
 * Start one case from its own row menu.
 *
 * Returns whether the row moved: a case the backend does not consider queued
 * stays where it was, and the caller needs to know that rather than wait out a
 * run that never began.
 */
async function startCaseFromItsRow(session, fragment) {
  const target = await until(
    `the actionable ${fragment} execution`,
    () =>
      session.execute(
        `const fragment = arguments[0];
     const row = [...document.querySelectorAll('nav[aria-label=Navigation] li[data-level=case]')].find(
       (candidate) => candidate.innerText.includes(fragment) && candidate.querySelector('button[aria-label^="Actions For "]'),
     );
     if (!row?.dataset.executionId) return null;
     return {
       before: row.innerText,
       executionId: row.dataset.executionId,
       trigger: row.querySelector('button[aria-label^="Actions For "]').getAttribute('aria-label'),
     };`,
        [fragment],
      ),
    120000,
  );
  /* A row menu is a modal layer: the next row is unreachable until it is gone. */
  await closeAnyMenu(session);
  const started = async () => {
    const row = await session.execute(
      `return [...document.querySelectorAll('nav[aria-label=Navigation] li[data-level=case]')]
          .find((candidate) => candidate.dataset.executionId === arguments[0])?.innerText ?? '';`,
      [target.executionId],
    );
    return Boolean(row) && (row !== target.before || /Running|Waiting|Finalizing|Passed|Failed/.test(row));
  };
  await until(
    `the ${fragment} row start command`,
    async () => {
      // A Native Click Can Outlive Its Popup. Reconcile The Execution Before Opening It Again.
      if (await started()) return true;
      const item = await session.named("menuitem", "Start Execution").catch(() => null);
      if (!item) {
        const trigger = await session.named("button", target.trigger);
        const expanded = await session.execute("return arguments[0].getAttribute('aria-expanded') === 'true'", [trigger]);
        if (!expanded) await session.click(trigger);
        return false;
      }
      try {
        await session.click(item);
        return true;
      } catch (failure) {
        if (/stale element reference|pointer target must remain mounted/i.test(failure.message)) return false;
        throw failure;
      }
    },
    120000,
  );
  await closeAnyMenu(session);
  return until(`the ${fragment} case to start`, started, 120000).catch(() => false);
}

/** Press a menu trigger until its menu is up, and answer with the menu. */
async function openMenu(session, name, attempts = 2) {
  let last;
  for (let attempt = 0; attempt < attempts; attempt += 1) {
    await session.clickNamed("button", name);
    const menu = await until(`the ${name} menu`, () => session.find("css selector", "[role=menu]"), 120000).catch((failure) => {
      last = failure;
      return null;
    });
    if (menu) return menu;
    await closeAnyMenu(session);
  }
  throw new Error(`${name} did not open its menu: ${last?.message ?? "no menu appeared"}`);
}

/** Leave no open menu behind: Radix keeps the page inert while one is up. */
async function closeAnyMenu(session) {
  await session.execute(`
    for (const menu of document.querySelectorAll('[role=menu]')) {
      for (const type of ['keydown', 'keyup']) menu.dispatchEvent(new KeyboardEvent(type, {bubbles: true, key: 'Escape'}));
    }
  `);
  await session.waitForDom(
    "menus closed",
    `
    return !document.querySelector('[role=menu]') && document.body.style.pointerEvents !== 'none';
  `,
  );
}

/** One case row's own text, which carries its status and attempt count. */
async function caseRowText(session, fragment) {
  return session.execute(
    `const fragment = arguments[0];
     const row = [...document.querySelectorAll('nav[aria-label=Navigation] li[data-level=case]')].find(
       (candidate) => candidate.innerText.includes(fragment),
     );
     return row ? row.innerText : '';`,
    [fragment],
  );
}

/** Whatever a transient notification is saying right now. */
async function toastText(session) {
  return session.execute("return [...document.querySelectorAll('[data-sonner-toast]')].map((toast) => toast.innerText).join(' | ');");
}

/** The sidebar's case rows as one string, which is where status lives. */
function sidebarText(session) {
  return session.execute("const nav = document.querySelector('nav[aria-label=Navigation]'); return nav ? nav.innerText : '';");
}

/**
 * Startup: the pre-connection column, then Cycles with three empty groups.
 *
 * The backend the shell attaches to is already serving, so the startup phase
 * list can be gone before the first sample. The screen itself is recorded; the
 * phases are reported as unobserved when the attach wins the race.
 */
async function startupJourney(session, notes) {
  const screen = await session
    .execute(`
      const phases = document.querySelector('ol[aria-label="Startup Phases"]');
      return { path: location.pathname, phases: phases ? phases.innerText : null };
    `)
    .catch(() => null);
  if (screen?.phases) {
    notes.push(`startup: phases observed -- ${normalize(screen.phases)}`);
  } else {
    notes.push("startup: the attach completed before the phase list could be sampled");
  }

  await until("cycles home", async () => (await session.location()).pathname === "/cycles", 120000);
  const body = await until(
    "loaded cycle groups",
    async () => {
      const text = await session.text();
      return hasText(text, "Recent") && !hasText(text, "Loading cycles") ? text : null;
    },
    120000,
  );
  for (const heading of ["Cycles", "Needs Attention", "Active", "Recent"]) {
    assert.ok(hasText(body, heading), `Cycles home must render "${heading}"`);
  }
  for (const empty of ["Nothing Is Waiting On You", "No Active Cycles", "No Cycles Loaded Yet"]) {
    assert.ok(hasText(body, empty), `An empty group must state "${empty}"`);
  }
  assert.ok(await session.exists("button", "New Cycle"), "Cycles home must offer New Cycle");
  assert.ok(await session.find("css selector", "nav[aria-label=Navigation]"), "The shell must render the sidebar");
  if (!windows) await verifyLinuxRenderer();
}

async function verifyLinuxRenderer() {
  const renderer = await until("owned WebKit renderer environment", async () => {
    const pending = [driverProcess.pid];
    for (let index = 0; index < pending.length && index < 64; index += 1) {
      const pid = pending[index];
      const executable = await realpath(`/proc/${pid}/exe`).catch(() => "");
      if (basename(executable) === "WebKitWebProcess") {
        const environment = (await readFile(`/proc/${pid}/environ`, "utf8")).split("\0");
        const value = environment.find((entry) => entry.startsWith("WEBKIT_DISABLE_DMABUF_RENDERER="))?.split("=")[1];
        return { value: value ?? null };
      }
      const children = await readFile(`/proc/${pid}/task/${pid}/children`, "utf8").catch(() => "");
      pending.push(...children.trim().split(/\s+/).filter(Boolean).map(Number));
    }
    return null;
  });
  const expected = process.env.WEBKIT_DISABLE_DMABUF_RENDERER ?? "1";
  assert.equal(renderer.value, expected, "The real renderer receives the default workaround or the explicit operator override");
  await writeFile(join(evidence, "linux-renderer.json"), JSON.stringify({ expected, ...renderer }, null, 2));
}

/**
 * The active cycle: shell state, changed only through its picker.
 *
 * The status bar names the active cycle and opens the picker. New Cycle there
 * creates a cycle and makes it active; choosing a row makes that cycle active;
 * the template dialog then names it with Change, and the case joins it.
 */
async function activeCycleJourney(session, otherTitle) {
  const element = () =>
    session.execute("return document.querySelector('[data-testid=status-active-cycle]')?.innerText.replace(/\\s+/g, ' ').trim() ?? null;");
  const openPicker = async () => {
    await session.clickCss("[data-testid=status-active-cycle]");
    await until("active cycle picker", () => session.find("css selector", "[role=dialog] [data-testid=open-cycles]"), 120000);
  };
  const clickInDialog = (label) =>
    until(`${label} in the open dialog`, () =>
      session.execute(
        `const dialog = [...document.querySelectorAll('[role=dialog][data-state=open]')].at(-1);
         const button = dialog && [...dialog.querySelectorAll('button')].find((candidate) => candidate.innerText.includes(arguments[0]));
         if (!button || button.disabled) return null;
         button.click();
         return true;`,
        [label],
      ),
    );
  const dialogsClosed = () =>
    until("dialogs closed", async () => (await session.execute("return document.querySelectorAll('[role=dialog]').length")) === 0, 120000);

  await session.goto("/library");
  await until("suite rows", async () => hasText(await session.text(), STARTUP_SUITE), 120000);
  assert.equal(
    await session.execute("return document.querySelector('select[aria-label=\"Destination Cycle\"]');"),
    null,
    "The Library carries no destination control",
  );

  /* New Cycle in the picker creates a cycle and makes it active at once. */
  await openPicker();
  await clickInDialog("New Cycle");
  const titleField = await until("new cycle dialog", () => session.find("css selector", "#new-cycle-title"), 120000);
  /* A title of its own, so no row or toast can match the library journey's cycle. */
  const title = `Active Cycle Journey ${Date.now()}`;
  await session.type(titleField, title);
  await session.clickNamed("button", "Create Cycle");
  await dialogsClosed();
  await until("the created cycle active", async () => hasText((await element()) ?? "", title), 120000);

  /* Choosing a row makes that cycle active; choosing the new one again restores it. */
  if (otherTitle) {
    await openPicker();
    await clickInDialog(otherTitle);
    await dialogsClosed();
    await until("the chosen cycle active", async () => hasText((await element()) ?? "", otherTitle), 120000);
  }
  await openPicker();
  await clickInDialog(title);
  await dialogsClosed();
  await until("the new cycle active again", async () => hasText((await element()) ?? "", title), 120000);
  assert.match((await element()) ?? "", /Active · Idle/, "The element states the cycle's own state");

  /* The template dialog names the active cycle with Change, and the case joins it. */
  await until("startup template", async () => hasText(await session.text(), STARTUP_TEMPLATE), 120000);
  await session.clickNamed("button", STARTUP_TEMPLATE);
  const inline = () =>
    session.execute("return document.querySelector('[role=dialog] [data-testid=template-active-cycle]')?.innerText ?? null;");
  const named = (expected) => until(`the dialog naming ${expected}`, async () => hasText((await inline()) ?? "", expected), 120000);
  await named(title);
  assert.ok(hasText((await inline()) ?? "", "Change"), "The template dialog must offer Change");

  /* Change opens the same picker over the dialog, and the choice applies in place. */
  for (const chosen of otherTitle ? [otherTitle, title] : [title]) {
    await clickInDialog("Change");
    await until("picker over the dialog", () => session.find("css selector", "[role=dialog] [data-testid=open-cycles]"), 120000);
    await clickInDialog(chosen);
    await until(
      "picker closed",
      async () => (await session.execute("return document.querySelectorAll('[data-testid=open-cycles]').length")) === 0,
      120000,
    );
    await named(chosen);
  }
  await clickInDialog("Next");
  await dialogsClosed();
  const added = await until("add announced", async () => {
    const text = await toastText(session);
    return /Added 1 case to /i.test(text) ? text : null;
  });
  assert.ok(hasText(added, title), `The case must join the active cycle (got ${normalize(added)})`);
  await session.clickNamed("button", "Open Cycle");
  return { executionIds: await cycleExecutionIds(session) };
}

/**
 * Library: the only route by which cases enter a cycle.
 *
 * The cycle comes first, through New Cycle, and becomes the active cycle that
 * `appendTo` in the URL names. Then one suite expanded, one template
 * configured, the editor exercised on a number and on a list, the summary
 * read, and the cases materialized twice into that same cycle.
 */
/** Materialize Only The Cases A Non-Library Scene Needs Through The Real Bridge. */
async function prepareCycle(session, title, selections) {
  await session.goto("/cycles");
  const cycle = await call(session, "Execution", "CreateCycle", { metadata: { label: title }, continueAutomatically: false });
  const catalog = await call(session, "Catalog", "ListSuites");
  const executionIds = [];
  const result = { id: cycle.cycleId, title, executionIds };
  for (const [suiteLabel, templateLabel] of selections) {
    const suite = catalog.suites.find((candidate) => candidate.metadata.label === suiteLabel);
    assert.ok(suite, `The prepared fixture registers ${suiteLabel}`);
    const definition = await call(session, "Catalog", "GetSuite", { suiteId: suite.suiteId });
    const template = definition.templates.find((candidate) => candidate.metadata.label === templateLabel);
    assert.ok(template, `The prepared fixture registers ${templateLabel}`);
    const plan = await call(session, "Planning", "MaterializeTemplate", {
      selection: { suiteId: suite.suiteId, templateId: template.templateId, parameters: [] },
    });
    const testCase = plan.cases[0];
    assert.ok(testCase, "A fixture selection materializes its case");
    const execution = { executionId: randomUUID(), caseId: testCase.caseId };
    await call(session, "Execution", "AppendExecutions", { cycleId: cycle.cycleId, executions: [execution] });
    executionIds.push(execution.executionId);
    if (templateLabel === STARTUP_TEMPLATE) result.startupExecutionId = execution.executionId;
    if (templateLabel === PROTECTION_TEMPLATE) result.measuringExecutionId = execution.executionId;
  }
  const opener = await until("prepared cycle in native navigation", () =>
    session.find("xpath", `//nav[@aria-label='Navigation']//button[normalize-space(.)=${xpathLiteral(title)}]`),
  );
  await session.click(opener);
  await until("prepared cycle selected", async () => (await session.location()).pathname === `/cycles/${cycle.cycleId}`);
  return result;
}

async function cycleExecutions(session) {
  const cycleId = /^\/cycles\/([^/]+)$/.exec((await session.location()).pathname)?.[1];
  assert.ok(cycleId, "Execution inventory belongs to the selected cycle");
  const executions = [];
  let afterExecutionId;
  let cursor;
  do {
    const page = await call(session, "Execution", "ListExecutions", {
      cycleId,
      limit: 32,
      ...(afterExecutionId ? { afterExecutionId } : {}),
      ...(cursor ? { cursor } : {}),
    });
    executions.push(...page.executions);
    afterExecutionId = page.nextAfterExecutionId;
    cursor = page.cursor;
  } while (afterExecutionId);
  return executions;
}

async function cycleExecutionIds(session) {
  const executions = await cycleExecutions(session);
  return executions.map((execution) => execution.executionId);
}

async function libraryJourney(session) {
  await session.goto("/library");
  await until("suite rows", async () => hasText(await session.text(), PROTECTION_SUITE), 120000);
  assert.equal(
    await session.execute("return document.querySelector('select[aria-label=\"Destination Cycle\"]');"),
    null,
    "The active cycle is shell state, not a Library control",
  );

  /* New Cycle opens a dialog titled by the moment it opened. */
  await session.clickNamed("button", "New Cycle");
  await until("new cycle dialog", () => session.find("css selector", "#new-cycle-title"), 120000);
  const title = await session.execute("return document.querySelector('#new-cycle-title').value");
  assert.ok(title.trim().length > 0, "New Cycle must prefill its title");
  assert.ok(hasText(await session.text(), "Continue Automatically"), "New Cycle must state its continuation policy");
  await session.clickNamed("button", "Create Cycle");
  const appendTo = await until(
    "appendTo in the URL",
    async () => {
      const location = await session.location();
      const id = location.searchParams.get("appendTo");
      return location.pathname === "/library" && id ? id : null;
    },
    120000,
  );
  await until(
    "new cycle dialog closed",
    async () => (await session.execute("return document.querySelectorAll('[role=dialog]').length")) === 0,
    120000,
  );
  await until("the created cycle active in the shell", async () =>
    hasText((await session.execute("return document.querySelector('[data-testid=status-active-cycle]')?.innerText ?? '';")) ?? "", title),
  );
  assert.ok(await session.exists("button", "Open Cycle"), "With an active cycle the page bar action must become Open Cycle");

  /* The Library lists every template beside its suite; a template's row opens its dialog. */
  assert.ok(hasText(await session.text(), PROTECTION_TEMPLATE), "The Library must list the suite's templates");
  await session.clickNamed("button", PROTECTION_TEMPLATE);
  await until("template dialog", () => session.find("css selector", "[role=dialog]"), 120000);
  const step1 = await session.text();
  assert.ok(hasText(step1, "Parameters"), "Step 1 must be Parameters");
  assert.ok(hasText(step1, "Summary"), "The step indicator must name Summary");
  assert.ok(hasText(step1, "Values you change are remembered for this template."), "Step 1 must state that edits persist");
  await until("parameter grid", () => session.find("css selector", "[data-testid=parameter-editor-grid]"));
  assert.ok(await session.exists("button", "Reset All"), "The parameter step must carry Reset All");
  const resetAll = await session.named("button", "Reset All", { enabled: false });
  if (await session.execute("return !arguments[0].disabled;", [resetAll])) await session.click(resetAll);
  await session.waitForDom(
    "fixture parameters reset",
    `
    return !document.querySelector('[data-testid="parameter-editor-grid"] [data-edited="true"]');
  `,
  );

  /* A number edit reveals that row's Reset affordance and its gilt rule. */
  const number = await session.find("css selector", `[data-testid="parameter-input-${NUMBER_PARAMETER}"]`);
  await session.type(number, "60");
  const edited = await until("edited row reset", () =>
    session.execute(
      `const row = document.querySelector('[data-testid="parameter-row-' + arguments[0] + '"]');
       if (!row || row.dataset.edited !== 'true') return null;
       const reset = row.querySelector('button[aria-label^="Reset To "]');
       const label = reset ? reset.getAttribute('aria-label') : null;
       return label && label.startsWith('Reset To') ? label : null;`,
      [NUMBER_PARAMETER],
    ),
  );
  assert.match(edited, /^Reset To /, "An edited row must offer a named reset");

  /* A list parameter: one chip per value, an add affordance, and an expression
     that expands into chips of its own. */
  const chips = () =>
    session.execute(
      `const cell = document.querySelector('[data-testid="parameter-chips-' + arguments[0] + '"]');
       return cell ? [...cell.querySelectorAll('input')].map((input) => input.value) : null;`,
      [LIST_PARAMETER],
    );
  const before = await until("list chips", chips);
  assert.ok(before.length >= 1, "A list parameter must render its values as chips");
  await session.clickCss(`[data-testid="parameter-chips-${LIST_PARAMETER}"] button[aria-label^="Add "]`);
  await until("added chip", async () => ((await chips()) ?? []).length === before.length + 1, 120000);
  await session.execute(
    `const cell = document.querySelector('[data-testid="parameter-chips-' + arguments[0] + '"]');
     const input = [...cell.querySelectorAll('input')].at(-1);
     const setter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value').set;
     setter.call(input, '1..3');
     input.dispatchEvent(new Event('input', {bubbles: true}));
     input.dispatchEvent(new KeyboardEvent('keydown', {bubbles: true, key: 'Enter'}));
     input.blur();`,
    [LIST_PARAMETER],
  );
  const expandedChips = await until(
    "expression expanded to chips",
    async () => {
      const values = (await chips()) ?? [];
      return values.length >= before.length + 2 ? values : null;
    },
    120000,
  );
  assert.ok(
    expandedChips.join(",").includes("1,2,3") || expandedChips.length >= before.length + 3,
    `The expression 1..3 must expand into chips (got ${expandedChips.join("|")})`,
  );

  /* The expression reference is a popover, reachable beside the value. */
  await session.clickNamed("button", "List Expression Help");
  const reference = await until("expression reference", () => session.find("css selector", "[data-testid=expression-reference]"));
  const referenceText = await session.textOf(reference);
  for (const syntax of ["a, b, c", "1..5", "linspace(start, stop, n)"]) {
    assert.ok(hasText(referenceText, syntax), `The reference must document ${syntax}`);
  }
  await session.key("Escape");
  await session.waitForDom("expression reference closed", "return !document.querySelector('[data-testid=expression-reference]');");

  /* Step 2 reads the preview back: the deltas, the count, and the target. */
  await session.clickNamed("button", "Next");
  await until("summary step", async () => hasText(await session.text(), "Finish And Add"), 120000);
  const summary = await session.text();
  assert.ok(hasText(summary, "Target Cycle"), "The summary must name the active cycle");
  assert.ok(hasText(summary, title.trim()) || hasText(summary, appendTo), "The summary target must be the cycle New Cycle created");
  assert.ok(/\d+ Cases?/.test(normalize(summary)), "The summary must state the case count");
  const deltas = await session.execute(`
    const list = document.querySelector('[data-testid="summary-cases"]');
    return list ? list.innerText : '';
  `);
  assert.ok(
    hasText(deltas, "Supply Frequency") || hasText(deltas, "Load Steps"),
    `The summary rows must state the parameter deltas (got ${normalize(deltas).slice(0, 200)})`,
  );

  await session.clickNamed("button", "Finish And Add");
  await until(
    "first add finished",
    async () => (await session.execute("return document.querySelectorAll('[role=dialog]').length")) === 0,
    120000,
  );
  assert.equal((await session.location()).searchParams.get("appendTo"), appendTo, "Adding cases must keep the active cycle");

  /* A second template joins the same cycle, so the run has a prompt in it. */
  await settle(session);
  await until("startup template", async () => hasText(await session.text(), STARTUP_TEMPLATE), 120000);
  await session.clickNamed("button", STARTUP_TEMPLATE);
  await until("second dialog", () => session.find("css selector", "[role=dialog]"), 120000);
  /* One case at its registered values has nothing to review: Next adds it at
     once and closes the dialog, without a summary step. */
  await session.clickNamed("button", "Next");
  await until(
    "second add finished",
    async () => (await session.execute("return document.querySelectorAll('[role=dialog]').length")) === 0,
    120000,
  );
  assert.equal((await session.location()).searchParams.get("appendTo"), appendTo, "Adding cases must keep the active cycle");

  await session.clickNamed("button", "Open Cycle");
  await until("cycle workspace", async () => (await session.location()).pathname === `/cycles/${appendTo}`, 120000);
  const startupExecutionId = await until("the appended startup execution", () =>
    session.execute(
      `return [...document.querySelectorAll('nav[aria-label=Navigation] li[data-level=case]')]
        .find(row => row.innerText.includes(arguments[0]))?.dataset.executionId ?? null;`,
      [STARTUP_TEMPLATE],
    ),
  );
  const measuringExecutionId = await until(
    "the materialized measuring execution identity",
    () =>
      session.execute(
        `return [...document.querySelectorAll('nav[aria-label=Navigation] li[data-level=case]')]
      .find((row) => row.innerText.includes(arguments[0]))?.dataset.executionId ?? null;`,
        [PROTECTION_TEMPLATE],
      ),
    120000,
  );
  return { id: appendTo, title: title.trim(), measuringExecutionId, startupExecutionId, executionIds: await cycleExecutionIds(session) };
}

/**
 * The cycle workspace: run it, answer it, then read it.
 *
 * One layout throughout: the sidebar owns selection, the header owns every
 * whole-cycle command, and the evidence pane shows what the URL selects.
 */
async function cycleJourney(session, cycleId) {
  await session.goto(`/cycles/${cycleId}?view=cycle`);
  const beforeReads = await session.execute("return globalThis.__dokimeStreamReads ?? 0");
  let listened = false;
  const sidebar = await until(
    "the cycle's case rows in the sidebar",
    async () => {
      const text = await session.execute(
        "const nav = document.querySelector('nav[aria-label=Navigation]'); return nav ? nav.innerText : '';",
      );
      return hasText(text, PROTECTION_TEMPLATE) && hasText(text, STARTUP_TEMPLATE) ? text : null;
    },
    120000,
  );
  assert.ok(hasText(sidebar, "Cycles"), "The sidebar must keep its destinations");
  /* `CaseSummary.status` is the row's word now, and for a cycle nobody has
     run it is owed work: PLANNED where no attempt exists, QUEUED where
     CreateCycle already materialized one and nothing dispatched it. Both read
     as work; "Skipped" or an outcome would not. */
  if (!hasText(sidebar, "Planned") && !hasText(sidebar, "Queued")) {
    defect(
      "cycle/case-status",
      `No case of a cycle that has never run reads as planned or queued work. The rows read ${JSON.stringify([
        ...new Set(normalize(sidebar).match(/Planned|Queued|Not Run|Skipped|Passed|Failed|Running/g) ?? []),
      ])}.`,
    );
  }

  // Mount The Selected Execution Listener After Installing The Reader Observation.
  await session.execute(
    `
    const row = [...document.querySelectorAll('nav[aria-label=Navigation] li[data-level=case]')]
      .find((candidate) => candidate.innerText.includes(arguments[0]));
    if (!row) throw new Error('The measuring case is absent');
    row.querySelector('button').click();
  `,
    [PROTECTION_TEMPLATE],
  );
  /* The cycle page carries the cycle's progress, status and run controls:
     select the cycle node, then come back to the measuring case. */
  await session.execute("document.querySelector('nav[aria-label=Navigation] li[data-level=overview] button[data-tree-select]').click();");
  await until("the cycle page", async () => (await session.location()).searchParams.get("view") === "cycle", 120000);
  const header = await session.text();
  assert.ok(/\d+ of \d+ cases finished/.test(normalize(header)), "The header must state progress");
  assert.ok(hasText(header, "Active · Idle"), "A materialized cycle that runs nothing must render its Active · Idle state");
  assert.ok(await session.exists("button", "Cycle Actions"), "The header must carry the overflow menu");
  await assertNodeDeliveries(session, "Cycle Actions");
  assert.ok(!(await session.exists("button", "Start Cycle")), "Containers have no Start control");
  assert.ok(!(await session.exists("button", "Stop Cycle")), "Containers have one Seal action");
  /* Observe The Measuring Execution Before Admitting Its Work. */
  await session.execute(
    `
    const row = [...document.querySelectorAll('nav[aria-label=Navigation] li[data-level=case]')]
      .find((candidate) => candidate.innerText.includes(arguments[0]));
    if (!row) throw new Error('The measuring case is absent');
    row.querySelector('button').click();
  `,
    [PROTECTION_TEMPLATE],
  );
  const measuringExecutionId = await until("the selected measuring execution", async () =>
    (await session.location()).searchParams.get("executionId"),
  );
  assert.ok(await startCaseFromItsRow(session, PROTECTION_TEMPLATE), "The measuring execution Start must be admitted");
  listened = await until("live bridge reads from the selected measuring execution", () =>
    session.execute(
      `return globalThis.__dokimeStreamReads > arguments[0]
        && globalThis.__dokimeStreams.some((stream) => !stream.done && stream.reads >= 2);`,
      [beforeReads],
    ),
  );
  assert.ok(await startCaseFromItsRow(session, STARTUP_TEMPLATE), "The prompting execution Start must be admitted");

  /* The Overview Links To Each Execution That Needs Input. */
  await session.execute("document.querySelector('nav[aria-label=Navigation] li[data-level=overview] button[data-tree-select]').click();");
  await until(
    "the cycle selection for operator prompts",
    async () => (await session.location()).searchParams.get("view") === "cycle",
    120000,
  );

  /* Open Report while the cycle is in flight: no seal yet, so the runtime
     renders one from the run so far and the shell hands it to the opener
     (`xdg-open` is a no-op here). It is never disabled and never refused. */
  const inFlight = await until(
    "the cycle header to show an in-flight lifecycle",
    async () => /\b(Running|Paused|Stopping)\b/.test(normalize(await session.text())),
    120000,
  ).catch(() => false);
  if (inFlight) {
    await openMenu(session, "Cycle Actions");
    const items = await session.execute(`
      return [...document.querySelectorAll('[role=menu] [role=menuitem]')].map((item) => ({
        disabled: item.getAttribute('aria-disabled') === 'true' || item.hasAttribute('data-disabled'),
        name: item.innerText.trim(),
      }));
    `);
    const open = items.find((item) => item.name === "Open Report");
    assert.ok(open, "The overflow menu must offer Open Report while the cycle runs");
    assert.ok(!open.disabled, "Open Report must stay enabled while the cycle is in flight");
    await session.clickNamed("menuitem", "Open Report");
    const opened = await until(
      "the in-flight report to open",
      async () => /Report Opened/.test(await toastText(session).catch(() => "")),
      120000,
    ).catch(() => false);
    if (!opened) {
      defect("cycle/open-report", `Open Report did not open a report while the cycle ran: ${await toastText(session).catch(() => "")}`);
    }
    await closeAnyMenu(session);
    assert.equal(
      await session.execute("return document.body.style.pointerEvents || '';"),
      "",
      "A closed overflow menu must leave the page interactive",
    );
  } else {
    defect(
      "cycle/open-report",
      "The admitted cycle never showed Running, Paused or Stopping, so Open Report could not be checked while it was in flight.",
    );
  }

  await runUntil(
    session,
    "cycle completion",
    async () => {
      const rows = await sidebarText(session);
      if (/\b(Running|Waiting|Finalizing)\b/.test(rows)) {
        listened ||= await session.execute(
          `
          return globalThis.__dokimeStreamReads > arguments[0]
            && globalThis.__dokimeStreams.some((stream) => !stream.done && stream.reads >= 2);
        `,
          [beforeReads],
        );
        return false;
      }
      return /\b(Passed|Failed|Blocked)\b/.test(await caseRowText(session, STARTUP_TEMPLATE));
    },
    300000,
  );

  /* The Startup Prompt Was Queued On Its Execution, Under Control, At Warning. */
  const startupSighting = promptSightings.find((seen) => seen.cards.some((card) => hasText(card.text, STARTUP_PROMPT)));
  assert.ok(startupSighting, `The prompt queue must show "${STARTUP_PROMPT}" before it is answered`);
  assert.ok(startupSighting.executionId, "A response panel must belong to a selected execution");
  const startupCard = startupSighting.cards.find((card) => hasText(card.text, STARTUP_PROMPT));
  assert.equal(startupCard.level, "warning", "The OBC plugin's startup prompt must render at its declared Warning level");
  assert.ok(
    startupCard.state === "open" && /Run Paused/i.test(startupCard.text) && !/Respond/.test(startupCard.text),
    `The oldest prompt must be open in its execution's queue, with no Respond (got ${normalize(startupCard.text)})`,
  );
  assert.ok(
    startupSighting.controlBottom !== null && startupSighting.queueTop >= startupSighting.controlBottom,
    `The prompt queue must sit directly under the Control card (queue ${startupSighting.queueTop}px, control ${startupSighting.controlBottom}px)`,
  );
  assert.match(startupSighting.header ?? "", /\d+ prompts? waiting/, "The prompt queue must count the open prompts");
  const progress = startupSighting.progress;
  assert.ok(progress && progress.tracks.length === 4, "Control must mount state, progress, count and actions in one row");
  assert.match(progress.count, /^\d+\/\d+$/);
  assert.ok(progress.radius >= progress.height / 2, "The progress bar must have rounded ends");
  const centers = progress.tracks.map((track) => track.center);
  assert.ok(Math.max(...centers) - Math.min(...centers) < 3, "Progress row tracks must share a vertical center");
  assert.ok(
    progress.tracks.every((track, index) => index === 0 || track.left >= progress.tracks[index - 1].right),
    "Progress row tracks must not overlap",
  );
  /* The Cycle Page Names Where Input Is Needed And Answers Nothing Itself. */
  for (const seen of cycleInputSightings) {
    assert.match(seen.text, /Input needed on #\d+/, "The cycle page must name the execution waiting on input");
    assert.ok(!seen.respond, "The cycle page must not offer Respond for an execution's prompt");
    assert.equal(seen.responsePanels, 0, "The cycle page must link to execution prompts without mounting response controls");
  }
  /* The navigation badge follows its own query, so any sighting of the open
     prompt may carry it; one must. */
  assert.ok(
    promptSightings.some((seen) => seen.cards.some((card) => hasText(card.text, STARTUP_PROMPT)) && /^\d+$/.test(seen.navigation ?? "")),
    `The Cycles navigation entry must count the open prompts (saw ${JSON.stringify(promptSightings.map((seen) => seen.navigation))})`,
  );

  assert.ok(listened, "the native application consumes live bridge stream updates while the execution is active");
  assert.equal(
    await session.execute("return globalThis.__dokimeNetwork ?? 0"),
    0,
    "the WebView reaches its backend only through the IPC bridge",
  );
  await session.dump(`listening-${cycleId}`, await session.execute("return globalThis.__dokimeStreams"));

  /* Each execution row selects its own evidence directly. The prompting case
     is the one re-run below: its re-run holds at the startup prompt, so it is
     still running, and its plots still accept settings, however slow the machine. */
  const executionRowsBefore = await until(
    "the prompting case's execution to finish",
    () =>
      session.execute(
        `
        const rows = [...document.querySelectorAll('nav[aria-label=Navigation] li[data-level=case]')];
        const terminal = rows.find(row => row.innerText.includes(arguments[0]) && /Passed|Failed|Blocked/.test(row.innerText));
        if (!terminal) return null;
        terminal.querySelector('button').click();
        return { count: rows.length, selectedIndex: rows.indexOf(terminal) };
      `,
        [STARTUP_TEMPLATE],
      ),
    120000,
  );
  const selection = await until(
    "execution selected in the URL",
    async () => {
      const location = await session.location();
      const executionId = location.searchParams.get("executionId");
      return executionId ? { executionId, caseId: location.searchParams.get("caseId") } : null;
    },
    120000,
  );
  assert.equal(selection.caseId, null, "Execution rows must select solely by executionId");
  const evidence = await until(
    "execution evidence",
    async () => {
      const text = await session.text();
      return /Execution \d+/i.test(normalize(text)) ? text : null;
    },
    120000,
  );
  assert.match(normalize(evidence), /Execution \d+/i);
  await checkParameterGeometry(session);
  await assertNodeDeliveries(session, "Execution Actions");
  assert.ok(!(await session.exists("button", "Choose Attempt")), "Execution selection belongs to the list");

  /* A Re-Run Appends A New Row And Preserves The Earlier Evidence. */
  await session.clickNamed("button", "Re-Run");
  const rerunId = await until(
    "new re-run execution selected",
    async () => {
      const location = await session.location();
      const id = location.searchParams.get("executionId");
      const count = await session.execute("return document.querySelectorAll('nav[aria-label=Navigation] li[data-level=case]').length;");
      return id && id !== selection.executionId && count === executionRowsBefore.count + 1 ? id : null;
    },
    120000,
  );
  assert.notEqual(rerunId, selection.executionId, "Re-Run must create a distinct execution");
  assert.match(await sidebarText(session), /Re-Run Of #\d+/, "The new row must name its previous execution");
  await until("Start on the fresh attempt", () => session.exists("button", "Start"), 120000);
  await session.clickNamed("button", "Start");
  await until("the new attempt admission", async () => /Execution Started/.test(await toastText(session)), 120000);
  /* Plot Settings Persist In The Native Frozen Document. */
  const plotId = await until(
    "the plot identity to configure",
    () =>
      session.execute(`
    return document.querySelector('button[aria-label^="Plot Settings For "]')?.closest('[data-plot-id]')?.dataset.plotId ?? null;
  `),
    120000,
  );
  const settingsSelector = `[data-plot-id=${JSON.stringify(plotId)}] button[aria-label^='Plot Settings For ']`;
  await session.clickCss(settingsSelector);
  const settingsText = await until(
    "plot settings dialog",
    () =>
      session.execute(`
        const dialog = document.querySelector('[role=dialog][data-state=open]');
        return dialog?.innerText;
      `),
    120000,
  );
  assert.match(settingsText, /Display Points|Reduction/);
  const pointsInput = await session.find("css selector", "[role=dialog] input[type=number]");
  await session.execute(
    `
    window.__plotInputValues = [];
    arguments[0].addEventListener('input', event => window.__plotInputValues.push(event.target.value));
  `,
    [pointsInput],
  );
  await session.type(pointsInput, "2500");
  const typedPoints = await session.execute("return {value: arguments[0].value, events: window.__plotInputValues}", [pointsInput]);
  assert.equal(typedPoints.value, "2500", `The settings input must receive the requested budget: ${JSON.stringify(typedPoints)}`);
  await session.clickNamed("button", "Apply");
  /* A dialog still open names why the backend has not accepted the settings. */
  let openDialog = "";
  try {
    await until(
      "plot settings accepted by backend",
      async () => {
        openDialog = (await session.execute("return document.querySelector('[role=dialog][data-state=open]')?.innerText ?? ''")) ?? "";
        return openDialog === "";
      },
      120000,
    );
  } catch (error) {
    throw new Error(`${error.message}; the open dialog reads: ${normalize(openDialog)}`);
  }
  await session.goto("/cycles");
  await session.goto(`/cycles/${cycleId}?executionId=${rerunId}`, { reload: true });
  await until(
    "the same plot frame after remounting",
    () =>
      session.execute(
        `
    const frame = [...document.querySelectorAll('[data-live-mounted="true"][data-plot-id]')]
      .find((node) => node.dataset.plotId === arguments[0]);
    return frame && !frame.innerText.includes('Loading Plot') && Boolean(frame.querySelector('button[aria-label^="Plot Settings For "]'));
  `,
        [plotId],
      ),
    120000,
  );
  // Reacquire Native Focus If A Running Prompt Remounts The Plot Control.
  const persistedPoints = await until(
    "persisted display point budget",
    async () => {
      const dialog = await session.execute(`
        const dialog = document.querySelector('[role=dialog][data-state=open]');
        const input = dialog?.querySelector('input[type=number]');
        return dialog ? {open: true, input: input ? {value: input.value} : null} : null;
      `);
      if (dialog) return dialog.input;
      const reopenedSettings = await session.find("css selector", settingsSelector);
      const focused = await session.execute(
        `
        arguments[0].focus();
        return document.activeElement === arguments[0] && !arguments[0].disabled;
      `,
        [reopenedSettings],
      );
      if (focused) await session.key("Enter");
      return null;
    },
    120000,
  );
  assert.equal(persistedPoints.value, "2500", "The backend must retain the plot budget after remounting");
  const cancelPlotSettings = await session.find("xpath", "//*[@role='dialog' and @data-state='open']//button[normalize-space(.)='Cancel']");
  await session.click(cancelPlotSettings);

  await runUntil(
    session,
    "the re-run to settle",
    async () => {
      const text = await sidebarText(session);
      return !/\b(Running|Waiting|Finalizing|Queued)\b/.test(text);
    },
    300000,
  );
  const rerunOutcome = await session.execute(
    `return [...document.querySelectorAll('nav[aria-label=Navigation] li[data-level=case]')]
      .find((row) => row.dataset.executionId === arguments[0])?.innerText ?? '';`,
    [rerunId],
  );
  assert.match(
    rerunOutcome,
    /\bPassed\b/,
    `Closing plot settings must leave the prompting re-run able to pass: ${normalize(rerunOutcome)}`,
  );
  const plotWindow = await until("plot window control", () => session.find("css selector", '[role="slider"][aria-label$="Window"]'));
  await session.execute("arguments[0].focus();", [plotWindow]);
  await session.key("End");
  await until("complete plot history selected", () =>
    session.execute("return arguments[0].getAttribute('aria-valuenow') === arguments[0].getAttribute('aria-valuemax');", [plotWindow]),
  );
  await until("prompt marker hover labels", async () => {
    const promptHovers = await session.executeAsync(`(${promptHoverLabels.toString()})().then(arguments[arguments.length - 1]);`);
    assertPromptHovers(promptHovers);
    return true;
  });
  assertHistogram(await until("live voltage histogram", () => session.execute(`return (${histogramPresentation.toString()})();`)));
  const statisticsHeaders = await until(
    "the native trace statistics headers",
    () =>
      session.execute(`
        const tables = [...document.querySelectorAll('table[aria-label="Trace Statistics"]')];
        if (!tables.length) return null;
        return tables.flatMap(table => [...table.querySelectorAll('th')].map(header => {
          const leaves = [...header.querySelectorAll('span')].filter(node => !node.children.length && node.textContent.trim());
          return {
            text: header.innerText,
            clipped: leaves.some(node => {
              const range = document.createRange();
              range.selectNodeContents(node);
              const text = range.getBoundingClientRect();
              let visibleLeft = text.left;
              let visibleRight = text.right;
              let parent = node;
              while (parent && parent !== table.parentElement) {
                const style = getComputedStyle(parent);
                const bounds = parent.getBoundingClientRect();
                if (['hidden', 'clip'].includes(style.overflowX) && (visibleLeft < bounds.left - 1 || visibleRight > bounds.right + 1)) {
                  const fullLabel = node.textContent.replace(/\\s+/g, ' ').trim();
                  const tooltip = node.closest('[title]')?.getAttribute('title')?.replace(/\\s+/g, ' ').trim();
                  if (style.textOverflow !== 'ellipsis' || tooltip !== fullLabel) return true;
                  visibleLeft = Math.max(visibleLeft, bounds.left);
                  visibleRight = Math.min(visibleRight, bounds.right);
                }
                if (['hidden', 'clip'].includes(style.overflowY) && (text.top < bounds.top - 1 || text.bottom > bounds.bottom + 1)) return true;
                parent = parent.parentElement;
              }
              return false;
            }),
          };
        }));
      `),
    120000,
  );
  assert.ok(
    statisticsHeaders.some((header) => /Y Cursor/.test(header.text)),
    "Native statistics include cursor headings",
  );
  assert.ok(
    statisticsHeaders.some((header) => /\[[^\]]+\]/.test(header.text)),
    "Native statistics include physical units",
  );
  assert.deepEqual(
    statisticsHeaders.filter((header) => header.clipped),
    [],
    "Statistics titles and units need full-label tooltips when truncated and must not clip vertically",
  );
  await session.execute(
    `
    const rows = [...document.querySelectorAll('nav[aria-label=Navigation] li[data-level=case]')];
    rows[arguments[0]].querySelector('button').click();
  `,
    [executionRowsBefore.selectedIndex],
  );
  await until(
    "original execution remains selectable",
    async () => (await session.location()).searchParams.get("executionId") === selection.executionId,
    120000,
  );

  /* Focus is a URL flag, so it survives a reload and can be linked. The router
     serializes the flag as `focus=true`; `focusSearchSchema` also accepts the
     `focus=1` a person would type, so both spellings are the flag. */
  await session.clickNamed("button", "Focus");
  await until("focus flag set", async () => ["1", "true"].includes((await session.location()).searchParams.get("focus")), 120000);
  assert.ok(await session.exists("button", "Leave Focus"), "Focus view must say how to leave it");
  await session.clickNamed("button", "Leave Focus");
  await until("focus flag cleared", async () => (await session.location()).searchParams.get("focus") !== "1", 120000);

  /* The console: scopes follow the route, Escape closes the drawer. */
  await session.clickNamed("button", "Open Console");
  const drawer = await until("console drawer", () => session.find("css selector", "section[aria-label=Console]"));
  const consoleText = await session.textOf(drawer);
  for (const scope of ["Instance", "Cycle", "Execution"]) {
    assert.ok(hasText(consoleText, scope), `The console must offer the ${scope} scope`);
  }
  await session.execute(`
    const bar = document.querySelector('section[aria-label=Console]');
    for (const name of ['Cycle', 'Execution']) {
      const item = [...bar.querySelectorAll('[role=radio], button')].find(
        (candidate) => candidate.innerText.trim() === name && !candidate.disabled,
      );
      item?.click();
    }
  `);
  const consoleLayout = await until("scrollable console records", () =>
    session.execute(`
    const viewport = [...document.querySelectorAll('section[aria-label=Console] div')]
      .find(node => getComputedStyle(node).overflowY === 'scroll' && node.scrollHeight > node.clientHeight);
    if (!viewport) return null;
    const rows = [...viewport.querySelectorAll('button')];
    if (!rows.length) return null;
    const first = rows[0].firstElementChild.getBoundingClientRect();
    const last = rows.at(-1).getBoundingClientRect();
    const bounds = viewport.getBoundingClientRect();
    return {inset: first.left - bounds.left, bottomPadding: bounds.bottom - last.bottom,
      tailDistance: viewport.scrollHeight - viewport.scrollTop - viewport.clientHeight};
  `),
  );
  assert.ok(consoleLayout.inset >= 8, "Console rows must have horizontal padding");
  assert.ok(consoleLayout.bottomPadding >= 4 && consoleLayout.tailDistance <= 1, "Console must initially follow its padded newest row");
  await session.execute(`
    const viewport = [...document.querySelectorAll('section[aria-label=Console] div')]
      .find(node => getComputedStyle(node).overflowY === 'scroll' && node.scrollHeight > node.clientHeight);
    viewport.scrollTop = 0;
    viewport.dispatchEvent(new Event('scroll', {bubbles: true}));
  `);
  await session.clickNamed("button", "Jump To Latest");
  await until("console follows newest row again", () =>
    session.execute(`
    const viewport = [...document.querySelectorAll('section[aria-label=Console] div')]
      .find(node => getComputedStyle(node).overflowY === 'scroll' && node.scrollHeight > node.clientHeight);
    return viewport && viewport.scrollHeight - viewport.scrollTop - viewport.clientHeight <= 1;
  `),
  );
  /* A real pointer, not `element.click()`. The level selector is a modal
     layer: opened by a script click it never sees the pointer sequence its own
     cleanup keys on, and it left `pointer-events: none` on the body after it
     closed, which made every later control in this journey unclickable. */
  await session.clickCss("section[aria-label=Console] [data-slot=select-trigger]");
  await session.waitForDom("console level options", `return document.querySelectorAll('[role=option]').length > 0;`);
  const levels = await session.execute("return [...document.querySelectorAll('[role=option]')].map((item) => item.innerText.trim());");
  for (const level of ["Trace", "Info", "Error"]) {
    assert.ok(levels.includes(level), `The level selector must offer ${level} (got ${levels})`);
  }
  await session.key("Escape");
  await session.waitForDom("console level menu closed", `return !document.querySelector('[role=listbox]');`);
  await until("console level focus restored", () =>
    session.execute(`
      const trigger = document.querySelector('section[aria-label=Console] [data-slot=select-trigger]');
      return trigger && document.activeElement === trigger && getComputedStyle(document.body).pointerEvents !== 'none';
    `),
  );
  await session.executeAsync("const done = arguments[arguments.length - 1]; requestAnimationFrame(() => requestAnimationFrame(done));");
  await session.key("Escape");
  await until(
    "console closed by Escape",
    async () => (await session.execute("return document.querySelectorAll('section[aria-label=Console]').length")) === 0,
    120000,
  );
  /* A modal layer that closed must give the page back. Stating it here names
     the cause; without it the next control simply never responds. */
  assert.equal(
    await session.execute("return document.body.style.pointerEvents || '';"),
    "",
    "A closed console must leave the page interactive",
  );

  /* The overflow menu names every whole-cycle command; Archive replaces
     clearing and changes nothing but the archive mark. The trigger is pressed again if the first press produced no menu: a real
     WebView occasionally drops the pointer sequence a dropdown opens on, and
     one silent miss is not a product defect. A second miss is, and fails. */
  await session.execute("document.querySelector('nav[aria-label=Navigation] li[data-level=overview] button[data-tree-select]').click();");
  await until("the cycle page", () => session.exists("button", "Cycle Actions"), 120000);
  const menu = await openMenu(session, "Cycle Actions");
  const items = await session.textOf(menu);
  for (const item of ["Copy Cycle", "Add Cases", "Follow Active", "Open Report", "Settings", "Archive"]) {
    assert.ok(hasText(items, item), `The overflow menu must offer ${item}`);
  }
  for (const item of ["Download Bundle", "Write Report", "Open Folder", "Clear Cycle"]) {
    assert.ok(!hasText(items, item), `The overflow menu must no longer offer ${item}`);
  }
  await closeAnyMenu(session);
  assert.equal(
    await session.execute("return document.body.style.pointerEvents || '';"),
    "",
    "A closed overflow menu must leave the page interactive",
  );
  return { ...selection, executionIds: [rerunId], reports: [measuringExecutionId, selection.executionId, rerunId] };
}

/**
 * Stop One Execution And Preserve Its Unstarted Siblings.
 *
 * Stop Execution in the page bar is scoped to the selected execution: the
 * cycle remains Active and sibling attempts remain unstarted. Seal Cycle
 * is the only container finalization action. The copy keeps the measured cycle the later journeys read
 * untouched.
 */
async function executionStopJourney(session, cycleId) {
  // === Copy The Cycle And Run It === //

  await session.goto(`/cycles/${cycleId}?view=cycle`);
  await openMenu(session, "Cycle Actions");
  await session.clickNamed("menuitem", "Copy Cycle");
  const copyId = await until(
    "the copied cycle route and startup action",
    async () => {
      const match = /^\/cycles\/([^/]+)$/.exec((await session.location()).pathname);
      if (!match || match[1] === cycleId) return null;
      const ready = await session.execute(
        `return [...document.querySelectorAll('nav[aria-label=Navigation] li[data-level=case]')]
          .some((candidate) => candidate.innerText.includes(arguments[0]) && candidate.querySelector('button[aria-label^="Actions For "]'));`,
        [STARTUP_TEMPLATE],
      );
      return ready ? match[1] : null;
    },
    120000,
  );
  const executionIds = await cycleExecutionIds(session);
  assert.ok(await startCaseFromItsRow(session, STARTUP_TEMPLATE), "Explicit Start admits the selected execution");

  // === Select The Running Execution === //

  /* The startup case holds its run open on the operator's inspection prompt,
     so a stop always lands while it is still running. */
  let running = null;
  await runUntil(
    session,
    "the copy's startup execution running",
    async () => {
      running = await session.execute(
        `
        const row = [...document.querySelectorAll('nav[aria-label=Navigation] li[data-level=case]')]
          .find((candidate) => /\\b(Running|Waiting)\\b/.test(candidate.innerText) && candidate.innerText.includes(arguments[0]));
        if (!row) return null;
        const label = row.querySelector('button');
        label.click();
        return row.dataset.executionId ?? null;
      `,
        [STARTUP_TEMPLATE],
      );
      return running !== null;
    },
    300000,
  );

  // === An Explicit Selection Beats Follow Active === //

  /* Selecting a queued execution while another runs must stay there: Follow
     Active only moves a selection that is following. The queued attempt shows
     its planned parameters. Selecting the cycle node selects the cycle. */
  const unstarted = await session.execute(`
    const row = [...document.querySelectorAll('nav[aria-label=Navigation] li[data-level=case]')]
      .find((candidate) => /\\b(Queued|Planned)\\b/.test(candidate.innerText));
    if (!row) return null;
    row.querySelector('button').click();
    return row.dataset.executionId ?? null;
  `);
  assert.ok(unstarted, "The copied cycle must offer an unstarted sibling for explicit selection");
  await session.waitForDom(
    "the selected unstarted execution",
    `return new URL(location.href).searchParams.get("executionId") === args[0] &&
      [...document.querySelectorAll('button')].some((button) =>
        button.textContent.trim() === 'Cancel Execution' && !button.disabled);`,
    [unstarted],
  );
  assert.equal(
    (await session.location()).searchParams.get("executionId"),
    unstarted,
    "An unstarted execution the operator selected must stay selected while another runs",
  );
  await checkParameterGeometry(session);
  /* An attempt that has not started is cancelled, not stopped: Stop belongs
     to a running attempt alone. */
  await until("Cancel Execution on the unstarted attempt", () => session.exists("button", "Cancel Execution"), 120000);
  assert.ok(!(await session.exists("button", "Stop Execution")), "An unstarted execution must not offer Stop");
  await session.execute("document.querySelector('nav[aria-label=Navigation] li[data-level=overview] button[data-tree-select]').click();");
  await until("the cycle selection", async () => (await session.location()).searchParams.get("view") === "cycle", 120000);
  await session.execute(
    `
    const row = [...document.querySelectorAll('nav[aria-label=Navigation] li[data-level=case]')]
      .find((candidate) => candidate.dataset.executionId === arguments[0]);
    row?.querySelector('button').click();
  `,
    [running],
  );
  assert.ok(!(await session.exists("button", "Stop Cycle")), "The execution page carries the execution's controls, not the cycle's");
  assert.ok(!(await session.exists("button", "Stop")), "No unscoped Stop may sit beside a selected execution");
  await until("Stop Execution", () => session.exists("button", "Stop Execution"), 120000);

  // === Stop Only That Execution === //

  await session.clickNamed("button", "Stop Execution");
  await until("the stop acknowledgement", async () => /Execution Stopped And Evidence Saved/.test(await toastText(session)), 120000);
  await writeFile(join(evidence, "invokes-execution-stop.json"), JSON.stringify(await drainInvokeLedger(session), null, 2));
  const header = normalize(await session.text());
  assert.ok(!/Stopping Cycle|\bStopping\b/.test(header), "Stopping one execution must not put the cycle into Stopping");

  // === Stop Seals Only The Selected Execution And Preserves Unstarted Siblings === //

  const stoppedRow = await session.execute(
    `return [...document.querySelectorAll('nav[aria-label=Navigation] li[data-level=case]')]
      .find((row) => row.dataset.executionId === arguments[0])?.innerText ?? '';`,
    [running],
  );
  assert.ok(/\b(Stopped|Interrupted)\b/.test(stoppedRow), `The stopped execution must keep its truthful outcome: ${stoppedRow}`);
  const rows = await sidebarText(session);
  assert.ok(/\b(Queued|Planned)\b/.test(rows), "Stop must preserve unstarted sibling attempts");

  // === Cancel Skips One Unstarted Sibling And Leaves The Cycle Active === //

  const cancelled = await session.execute(`
    const row = [...document.querySelectorAll('nav[aria-label=Navigation] li[data-level=case]')]
      .find((candidate) => /\\b(Queued|Planned)\\b/.test(candidate.innerText));
    if (!row) return null;
    row.querySelector('button').click();
    return row.dataset.executionId ?? null;
  `);
  assert.ok(cancelled, "An unstarted sibling must remain to cancel");
  await until("Cancel Execution", () => session.exists("button", "Cancel Execution"), 120000);
  await session.clickNamed("button", "Cancel Execution");
  await until("the cancel acknowledgement", async () => /Execution Cancelled/.test(await toastText(session)), 120000);
  const cancelledRow = await until("the cancelled row", () =>
    session.execute(
      `const text = [...document.querySelectorAll('nav[aria-label=Navigation] li[data-level=case]')]
        .find((row) => row.dataset.executionId === arguments[0])?.innerText ?? '';
       return /\\bSkipped\\b/.test(text) ? text : null;`,
      [cancelled],
    ),
  );
  assert.ok(cancelledRow, "The cancelled execution must be recorded as skipped");
  await session.execute("document.querySelector('nav[aria-label=Navigation] li[data-level=overview] button[data-tree-select]').click();");
  await cyclePhase(session, "Active");
  assert.ok(
    !hasText(await session.text(), "Active · Stopped"),
    "Stopping or cancelling one execution must leave the cycle Active, not Stopped",
  );
  return { id: copyId, executionIds, unrun: [cancelled], reports: [running] };
}

/** Completed executions seal independently; the operator closes their container. */
async function containersJourney(session, cycleId) {
  await session.goto(`/cycles/${cycleId}?view=cycle`);
  await cyclePhase(session, "Active");
  const executions = await cycleExecutions(session);
  const reports = executions.filter((execution) => !execution.sealed).map((execution) => execution.executionId);
  await exportAvailability(session, false);
  const select = await until("a finished execution row", () =>
    session.find("css selector", "nav[aria-label=Navigation] li[data-level=case] button[data-tree-select]"),
  );
  await session.click(select);
  const executionId = await until("a finished execution selected", async () => (await session.location()).searchParams.get("executionId"));
  await exportAvailability(session, true, "execution");

  // A Second Container Does Not Replace The First.
  await session.goto("/cycles");
  await session.clickNamed("button", "New Cycle");
  const input = await until("new independent cycle", () => session.find("css selector", "#new-cycle-title"));
  await session.type(input, "Independent Remote Cycle");
  await session.clickNamed("button", "Create Cycle");
  const independentId = await until("the independent cycle route", async () => {
    const match = /^\/cycles\/([^/]+)$/.exec((await session.location()).pathname);
    return match && match[1] !== cycleId ? match[1] : null;
  });
  await cyclePhase(session, "Active");
  await exportAvailability(session, false);
  assert.ok(hasText(await session.text(), "0 of 0 cases finished"), "A newly selected empty cycle must not inherit another cycle's cases");

  await session.goto(`/cycles/${cycleId}?view=cycle`);
  await cyclePhase(session, "Active");
  await session.clickNamed("button", "Seal Cycle");
  const confirmation = await until("seal scope confirmation", () => session.find("css selector", "[role=dialog][data-state=open]"));
  const description = await session.textOf(confirmation);
  assert.ok(hasText(description, "closes this cycle permanently"), "Sealing must explain that admission closes permanently");
  assert.ok(hasText(description, "Other cycles keep running"), "Sealing must state its cycle-local scope");
  await session.key("Escape");
  await until(
    "seal cancelled without effect",
    async () => (await session.execute("return document.querySelectorAll('[role=dialog][data-state=open]').length")) === 0,
  );
  await cyclePhase(session, "Active");
  await exportAvailability(session, false);

  await sealContainer(session);
  await exportAvailability(session, true);
  assert.ok(!(await session.exists("button", "Start Cycle")), "A sealed cycle must not offer new run admission");
  assert.ok(!(await session.exists("button", "Seal Cycle")), "A sealed cycle must show its final state rather than another seal command");
  await session.goto(`/cycles/${cycleId}?executionId=${executionId}`);
  await exportAvailability(session, true, "execution");
  assert.equal(
    (await session.location()).searchParams.get("executionId"),
    executionId,
    "Sealing must preserve the explicit execution selection",
  );

  // The Other Active Cycle Still Exists And Owns Its Own Seal.
  await session.goto(`/cycles/${independentId}?view=cycle`);
  await cyclePhase(session, "Active");
  await exportAvailability(session, false);
  await sealContainer(session);
  await exportAvailability(session, true);
  return { reports };
}

/** The cycle's own state reads "Active · Idle" and the like while Active, then "Sealing" and "Sealed". */
async function cyclePhase(session, phase) {
  await until(`cycle phase ${phase}`, async () => hasText(await session.text(), phase === "Active" ? "Active ·" : phase));
}

async function exportAvailability(session, enabled, kind = "cycle") {
  await openMenu(session, kind === "execution" ? "Execution Actions" : "Cycle Actions");
  const state = await until(`evidence export ${enabled ? "enabled" : "disabled"}`, () =>
    session.execute(
      `
    const button = [...document.querySelectorAll('[role=menuitem]')].find((candidate) => candidate.innerText.trim() === 'Export Evidence');
    if (!button) return null;
    const disabled = button.getAttribute('aria-disabled') === 'true';
    if (disabled === arguments[0]) return null;
    return { disabled, title: button.title };
  `,
      [enabled],
    ),
  );
  assert.equal(state.disabled, !enabled);
  assert.equal(state.title, enabled ? "Download SQLite, Files And Saved Reports" : "Seal This Cycle Before Exporting");
  await session.key("Escape");
}

async function assertNodeDeliveries(session, trigger) {
  await openMenu(session, trigger);
  for (const name of ["Export Evidence", "Download Report"]) {
    assert.ok(
      await until(`${trigger} offers ${name}`, () => session.named("menuitem", name, { enabled: false })),
      `${trigger} must name ${name}`,
    );
  }
  await session.key("Escape");
}

async function sealContainer(session) {
  await session.clickNamed("button", "Seal Cycle");
  const confirm = await until("explicit seal button", () =>
    session.find("xpath", "//*[@role='dialog']//button[normalize-space(.)='Seal Cycle']"),
  );
  await session.click(confirm);
  await until(
    "seal confirmation closed",
    async () => (await session.execute("return document.querySelectorAll('[role=dialog]').length")) === 0,
  );
  await cyclePhase(session, "Sealed");
}

/** Cycles home after the run: the Recent row, the Archived view, and the filter. */
async function cyclesHomeJourney(session, cycleTitle) {
  await session.goto("/cycles");
  const body = await until(
    "a cycle row",
    async () => {
      const text = await session.text();
      /* The stopped copy is the loaded cycle; it lands in whichever group its
         status selects (a stopped execution leaves it needing attention). */
      return hasText(text, cycleTitle) && /Active · (Idle|Running|Waiting)|Sealing|Sealed/.test(text) ? text : null;
    },
    120000,
  );
  assert.ok(hasText(body, "Attention"), "The group must render its column headings");
  await checkStatusBounds(session);
  assert.ok(/\d+ \/ \d+/.test(normalize(body)), "A row must state executions over cases");
  const updated = await session.execute(
    `
    return [...document.querySelectorAll('tbody tr')]
      .filter(row => row.cells[0]?.innerText.trim() === arguments[0])
      .map(row => ({text: row.cells[4]?.innerText.trim(), detail: row.cells[4]?.querySelector('[title]')?.title}));
  `,
    [cycleTitle],
  );
  assert.ok(updated.length > 0, "The cycle must have a dated row");
  for (const row of updated) {
    assert.match(row.text, /\d{1,4}\D+\d{1,2}\D+\d{1,4}.*\d{1,2}:\d{2}/, "A row must state its localized absolute update time");
    assert.ok(
      row.detail?.includes(`Updated ${row.text}`) && /Created [^\n]+/.test(row.detail),
      "The time tooltip preserves update and creation instants",
    );
  }

  /* Nothing is archived yet, so the Archived view is one empty group, and
     leaving it brings the run's cycle back. */
  await session.clickNamed("button", "Archived");
  assert.ok(
    await until("archived view", async () => hasText(await session.text(), "No Archived Cycles")),
    "The Archived view must say when nothing is archived",
  );
  await session.clickNamed("button", "Archived");
  await until("default view", async () => !hasText(await session.text(), "No Archived Cycles"), 120000);

  const filter = await session.find("css selector", "input[aria-label='Filter Cycles']");
  await session.type(filter, "zzz-no-such-cycle");
  await until("filter excludes every row", async () => hasText(await session.text(), "No Cycles Match This Filter"), 120000);
  await session.type(filter, "");
  await until("filter cleared", async () => !hasText(await session.text(), "No Cycles Match This Filter"), 120000);
  assert.ok(
    await session.execute(
      `return [...document.querySelectorAll('button')].some((button) =>
         button.innerText.includes(arguments[0]),
       );`,
      [cycleTitle],
    ),
    `The run's cycle "${cycleTitle}" must be one of the rows`,
  );
}

/** History: the recorded executions behind the run, and the Back chevron. */
async function historyJourney(session, cycleId, cycleTitle, executionId) {
  await session.goto("/history");
  /* The empty table is one placeholder row too, so a row counts once it opens
     something: its primary cell is the control an operator clicks. */
  const opener = await until(
    "history rows",
    () =>
      session.execute(
        `
      const candidates = [...document.querySelectorAll('tbody tr')].filter(row =>
        /\\b[1-9]\\d* passed\\b/i.test(row.innerText)
        && [...row.querySelectorAll('button')].some(button => button.innerText.trim() === arguments[0]));
      if (candidates.length !== 1) return null;
      return [...candidates[0].querySelectorAll('button')].find(button => button.innerText.trim() === arguments[0]);
    `,
        [cycleTitle],
      ),
    120000,
  );
  assert.ok(hasText(await session.text(), "History"), "The page bar must name History");
  await checkStatusBounds(session);
  await session.click(opener);
  await until("history detail", async () => (await session.location()).pathname === `/history/${cycleId}`, 120000);
  const checkpoint = await until(
    "the executed measuring checkpoint",
    () =>
      session.execute(
        `
    const identity = [...document.querySelectorAll('table[aria-label="History Executions"] [data-execution-id]')]
      .find((node) => node.dataset.executionId === arguments[0]);
    const entry = identity?.closest('tr');
    if (!entry || !/\\b(passed|failed)\\b/i.test(entry.innerText)) return null;
    return entry.querySelector('button[aria-label="Inspect"]:not(:disabled)');
  `,
        [executionId],
      ),
    120000,
  );
  await session.click(checkpoint);
  await until("captured execution", async () => hasText(await session.text(), "Recorded Steps"), 120000);
  const captured = await session.text();
  assert.ok(hasText(captured, "Recorded status"), "The checkpoint must state its recorded status");
  for (const view of ["Records", "Telemetry", "Files", "Context"]) {
    assert.ok(await session.exists("button", view), `The checkpoint must offer the ${view} view`);
  }
  await session.clickNamed("button", "Records");
  await until(
    "captured records",
    async () => {
      const section = await session.find("css selector", "section[aria-label='Captured Records']").catch(() => null);
      if (!section) return false;
      return !hasText(await session.textOf(section), "No execution records at this checkpoint.");
    },
    120000,
  );
  await session.clickNamed("button", "Context");
  await until("captured context", async () => hasText(await session.text(), "Notes At Capture"));

  await session.clickNamed("button", "Back");
  await until("back to history", async () => (await session.location()).pathname === "/history", 120000);
}

/** Remote Access Uses Mounted Controls And Real TLS CLI Calls. */
async function remoteAccessJourney(recoveredReports) {
  const input = (label) =>
    until(label, () => session.find("xpath", `//input[@id=//label[normalize-space(.)=${xpathLiteral(label)}]/@for]`));
  /* Preferences Autosave Before The Listener Is Explicitly Restarted. */
  const settled = () =>
    until("remote settings settled", () =>
      session.execute(`
        const states = [...document.querySelectorAll('[data-state=saved], [data-state=saving], [data-state=error]')];
        const restart = [...document.querySelectorAll('button')].find(button => button.innerText.trim() === 'Restart Listener');
        return states.length > 0 && states.every(status => status.dataset.state === 'saved')
          && [...document.querySelectorAll('input[id], button[role=switch]')].every(control => !control.disabled)
          && restart && !restart.disabled;
      `),
    );
  const toggle = async (label, checked) => {
    const control = await until(label, () =>
      session.find("xpath", `//*[@role='switch' and @id=//label[normalize-space(.)=${xpathLiteral(label)}]/@for]`),
    );
    await settled();
    const current = await session.execute("return arguments[0].getAttribute('aria-checked') === 'true';", [control]);
    if (current !== checked) await session.click(control);
    await until(`${label} applied`, () =>
      session.execute("return (arguments[0].getAttribute('aria-checked') === 'true') === arguments[1] && !arguments[0].disabled;", [
        control,
        checked,
      ]),
    );
  };
  /* Wait For Each Persisted Field Before Editing The Next One. */
  const commitField = async (label, value) => {
    await settled();
    await session.type(await input(label), value);
    await session.key("Tab");
    await settled();
  };
  /* A notice still on screen from an earlier change cannot stand in for this one. */
  const notice = async (text, action) => {
    await settle(session);
    await until(`no earlier ${text}`, async () => !hasText(await toastText(session), text), 120000);
    await action();
    return until(text, async () => {
      const shown = await toastText(session);
      return hasText(shown, text) ? shown : null;
    });
  };
  const endpoint = () =>
    until("remote listening address", async () => {
      const address = await session.execute(`
        const section = [...document.querySelectorAll('section')].find(node => node.querySelector('h2')?.textContent === 'Effective Listener State');
        return section?.querySelector('[role=status]')?.textContent?.trim() ?? '';
      `);
      return /^127\.0\.0\.1:\d+$/.test(address) ? `https://${address}` : null;
    });
  const restart = async () => {
    await settled();
    await session.clickNamed("button", "Restart Listener");
    await settled();
  };
  /* Certificate controls are advanced settings; each settings page states the view it needs. */
  const showAdvanced = async () => {
    await openMenu(session, "Page Actions");
    await toggle("Show Advanced", true);
    await session.key("Escape");
    await until("settings actions closed", () => session.execute("return document.querySelectorAll('[role=menu]').length === 0;"));
  };
  const regenerate = async () => {
    await settled();
    await session.clickNamed("button", "Regenerate Certificate");
    await settled();
  };
  const profiles = join(work, "pairing-profiles.json");
  const env = { ...process.env, DOKIME_APPLICATION_ROOT: join(work, "cli") };
  delete env.DOKIME_ENDPOINT;
  const cli = (args, applicationRoot = env.DOKIME_APPLICATION_ROOT) =>
    executeFile(cliBinary, args, {
      windowsHide: true,
      cwd: work,
      env: { ...env, DOKIME_APPLICATION_ROOT: applicationRoot },
      timeout: 120000,
    });
  const connect = async (identifier, accepted, applicationRoot, profilesPath = profiles) => {
    let result;
    try {
      result = await cli(["catalog", "list", "--backend", identifier, "--profiles-file", profilesPath], applicationRoot);
    } catch (error) {
      assert.ok(!accepted, `The paired client establishes a real CLI connection: ${redact(error.stderr ?? error.message)}`);
      assert.equal(error.killed, false);
      assert.ok(Number.isInteger(error.code) && error.code !== 0);
      return;
    }
    assert.ok(accepted, "The revoked client is refused");
    assert.ok(result.stdout.trim().length > 0);
    assert.ok(![...secrets].some((secret) => result.stdout.includes(secret) || result.stderr.includes(secret)));
  };
  const createPairing = async (identifier, address) => {
    const path = join(await realpath(work), `${identifier}.dokime-remote`);
    const saved = await notice("Remote Access File Saved", async () => {
      await session.openPicker(await session.named("button", "Export Remote Access"));
      const picker = { window: null };
      await until("native pairing save", () => chooseNativeFile(picker, path, "Save"), 120000);
    });
    /* The notice names the file the product wrote; it must be the one the operator chose. */
    assert.ok(hasText(saved, path), `The pairing file is saved where the operator chose: ${normalize(saved)}`);
    const pairing = JSON.parse(await readFile(path, "utf8"));
    assert.equal(pairing.version, 1);
    assert.deepEqual(pairing.endpoints, [address]);
    if (pairing.access_key) {
      secrets.add(pairing.access_key);
      await session.clickNamed("button", pairing.label);
      await session.type(await session.find("css selector", '[aria-label="Client Label"]'), identifier);
      await session.clickNamed("button", "Rename");
    } else {
      assert.equal(await session.exists("button", pairing.label), false, "A keyless access file creates no paired-client verifier");
    }
    await cli(["backend", "add", "--remote", path, "--identifier", identifier, "--profiles-file", profiles]);
    return path;
  };

  // Enable A Stable TLS Endpoint Before Exporting Client Pairings.
  await session.goto("/settings/remote");
  await until("remote settings", async () => hasText(await session.text(), "Listener Enabled"));
  await showAdvanced();
  const reservation = net.createServer();
  reservation.listen(0, "127.0.0.1");
  await once(reservation, "listening");
  const listenAddress = `127.0.0.1:${reservation.address().port}`;
  await new Promise((done) => reservation.close(done));
  await commitField("Listener Address", listenAddress);
  const certificateInput = await input("Certificate Names");
  await session.type(certificateInput, "localhost:");
  await until("invalid certificate name rejected", () =>
    session.execute("return arguments[0].getAttribute('aria-invalid') === 'true';", [certificateInput]),
  );
  await session.type(certificateInput, "127.0.0.1, localhost");
  await session.key("Tab");
  await settled();
  await regenerate();
  await toggle("Listener TLS", true);
  await toggle("Listener Authentication", true);
  await toggle("Listener Enabled", true);
  await restart();
  const address = await endpoint();
  const first = await createPairing("desktop-first", address);
  await createPairing("desktop-second", address);
  await connect("desktop-first", true);
  await connect("desktop-second", true);

  // Import Through The Desktop's Native Picker Without Exposing Credentials To The WebView.
  await session.goto("/settings/backends");
  await session.clickNamed("button", "Add Backend");
  const identifier = await until("backend identifier", () => session.find("css selector", '[aria-label="Backend Identifier"]'));
  await session.type(identifier, "desktop-pairing");
  await session.openPicker(await session.find("xpath", "//*[@role='dialog']//button[normalize-space(.)='Import Remote']"));
  const importPicker = { window: null };
  await until("native pairing import", () => chooseNativeFile(importPicker, first), 120000);
  await until("imported backend", async () => hasText(await session.text(), "desktop-pairing"));

  // A Failed Bind Preserves The Existing Listener.
  await session.goto("/settings/remote", { reload: true });
  const occupied = net.createServer();
  occupied.listen(0, "127.0.0.1");
  await once(occupied, "listening");
  try {
    await commitField("Listener Address", `127.0.0.1:${occupied.address().port}`);
    await notice("Remote Access Change Failed", restart);
    assert.equal(await endpoint(), address);
    await connect("desktop-first", true);
    await commitField("Listener Address", listenAddress);
    await restart();
  } finally {
    await new Promise((done) => occupied.close(done));
  }

  // Restart Preserves Both Independent Client Verifiers.
  const nativeEndpoint = backend.endpoint;
  await stopBackend();
  backend = await startBackend(work, evidence, nativeEndpoint, "backend-remote-restart", false);
  await session.goto("/settings/remote", { reload: true });
  assert.equal(await endpoint(), address);
  await connect("desktop-first", true);
  await connect("desktop-second", true);
  const browserStorage = await session.execute("return JSON.stringify([Object.entries(localStorage), Object.entries(sessionStorage)]);");
  assert.ok(![...secrets].some((secret) => browserStorage.includes(secret)));

  // Revocation Preserves An Admitted Execution Waiting For Its Operator.
  const remoteCycle = await prepareCycle(session, "Revocation Campaign", [[STARTUP_SUITE, STARTUP_TEMPLATE]]);
  const activeCycle = remoteCycle.id;
  assert.ok(await startCaseFromItsRow(session, STARTUP_TEMPLATE));
  await until("operator prompt before revocation", () => startupPromptOpen(session), 120000);
  await session.goto("/settings/remote");

  // Revoking One Client Leaves The Other Client And Local Desktop Connected.
  const revoke = await until("first paired client", () =>
    session.find(
      "xpath",
      "//table[@aria-label='Paired Clients']//tr[td[normalize-space(.)='desktop-first']]//button[@aria-label='Revoke']",
    ),
  );
  await session.click(revoke);
  await until(
    "first client removed",
    async () =>
      (await session.findAll("xpath", "//table[@aria-label='Paired Clients']//tr[td[normalize-space(.)='desktop-first']]")).length === 0,
  );
  await connect("desktop-first", false);
  await connect("desktop-second", true);
  await session.goto(`/cycles/${activeCycle}?view=cycle`);
  await cyclePhase(session, "Active");
  /* The cycle page names the execution waiting on input; the prompt itself is answered there. */
  await until("the cycle names the execution its surviving prompt waits on", async () =>
    hasText(await session.text(), "Input needed on #"),
  );
  const running = await caseRowText(session, STARTUP_TEMPLATE);
  assert.match(running, /\b(Running|Waiting)\b/, "revocation leaves the admitted execution running");
  await session.execute(
    `const row = [...document.querySelectorAll('nav[aria-label=Navigation] li[data-level=case]')]
      .find((candidate) => candidate.innerText.includes(arguments[0]));
     row.querySelector('button[data-tree-select]').click();`,
    [STARTUP_TEMPLATE],
  );
  await until("the surviving prompt on its execution", () => startupPromptOpen(session), 120000);
  const decision = await session.execute(`
    const panel = document.querySelector('[aria-labelledby="active-prompt-title"]');
    return {actions: [...panel.querySelectorAll('button')].map(button => button.innerText.trim()),
      checkboxes: panel.querySelectorAll('[role=checkbox], input[type=checkbox]').length};
  `);
  assert.ok(decision.actions.includes("No") && decision.actions.includes("Yes"), "The inspection decision must offer No and Yes");
  assert.equal(decision.checkboxes, 0, "The decision belongs in the footer without a duplicate checkbox");
  await session.clickNamed("button", "Stop Execution");
  await until("revocation execution stopped", async () => hasText(await toastText(session), "Execution Stopped And Evidence Saved"));
  // Reissuing The Leaf Preserves The Existing CA Trust Anchor.
  await session.goto("/settings/remote");
  await showAdvanced();
  await regenerate();
  await connect("desktop-second", true);
  await createPairing("desktop-regenerated", address);
  await connect("desktop-regenerated", true);

  // Authentication And Encryption Remain Independent Lab Choices.
  await session.goto("/settings/remote");
  await toggle("Listener Authentication", false);
  await restart();
  await createPairing("desktop-lab-anonymous", address);
  await connect("desktop-lab-anonymous", true);
  await toggle("Listener Authentication", true);
  await restart();
  await connect("desktop-lab-anonymous", false);
  await toggle("Listener TLS", false);
  await restart();
  await createPairing("desktop-lab-http", address.replace("https:", "http:"));
  await connect("desktop-lab-http", true);
  await toggle("Listener TLS", true);
  await restart();
  await connect("desktop-regenerated", true);
  await writeFile(join(evidence, "invokes-remote-access.json"), JSON.stringify(await drainInvokeLedger(session), null, 2));
  const clients = await Promise.allSettled(
    Array.from({ length: 21 }, async (_, index) => {
      const root = join(work, "session-clients", String(index));
      await mkdir(root, { recursive: true });
      const clientProfiles = join(root, "profiles.json");
      await copyFile(profiles, clientProfiles);
      await connect("desktop-regenerated", true, root, clientProfiles);
    }),
  );
  for (const client of clients) if (client.status === "rejected") throw client.reason;
  await session.goto("/settings/remote", { reload: true });
  const pageSize = await until("remote session table page size", () =>
    session.find("xpath", "//table[@aria-label='Recent Remote Sessions']/parent::div/parent::div//select[@aria-label='Rows Per Page']"),
  );
  await session.execute(
    `const select = arguments[0];
     if (![...select.options].some(option => option.value === '25')) throw new Error('The remote session table must offer twenty-five rows');
     select.value = '25';
     select.dispatchEvent(new Event('change', {bubbles: true}));`,
    [pageSize],
  );
  const sessionRows = () =>
    session.execute(`
    const table = document.querySelector('table[aria-label="Recent Remote Sessions"]');
    if (!table) return [];
    table.scrollIntoView({block: 'center'});
    return [...table.querySelectorAll('tbody tr')].filter(row => row.cells.length > 1).map(row => row.cells[0].innerText);
  `);
  const firstPage = await until("twenty remote sessions", async () => {
    const rows = await sessionRows();
    return rows.length === 20 ? rows : null;
  });
  await session.clickNamed("button", "Older");
  await until("second remote sessions page", async () => hasText(await session.text(), "Page 2"));
  const secondPage = await until("older remote sessions", async () => {
    const rows = await sessionRows();
    return rows.length > 0 && rows.every((row) => !firstPage.includes(row)) ? rows : null;
  });
  assert.ok(secondPage.length <= 20, "Every remote session page is bounded to twenty rows");
  await session.clickNamed("button", "Newer");
  await until("newest remote sessions restored", async () => JSON.stringify(await sessionRows()) === JSON.stringify(firstPage));
  await session.screenshot("remote-access");
  for (const identifier of ["desktop-first", "desktop-second", "desktop-regenerated", "desktop-lab-anonymous", "desktop-lab-http"]) {
    await cli(["backend", "remove", "--identifier", identifier, "--profiles-file", profiles]);
  }
  await session.goto("/settings/backends");
  await openMenu(session, "Actions For desktop-pairing");
  await session.clickNamed("menuitem", "Delete");
  await session.clickNamed("button", "Confirm Delete");
  await until("paired backend removed", async () => !(await session.exists("button", "Actions For desktop-pairing")));
  return { executionIds: remoteCycle.executionIds, reports: [...recoveredReports, remoteCycle.startupExecutionId] };
}

/** Settings: every section, and the rows the redesign promises in each. */
async function settingsJourney(session, notes) {
  const sections = [
    ["backends", "Backends"],
    ["appearance", "Appearance"],
    ["organization", "Organization"],
    ["plugins", "Plugins"],
    ["secrets", "Secrets"],
    ["remote", "Remote Access"],
    ["diagnostics", "Diagnostics"],
    ["about", "About"],
  ];
  for (const [id, heading] of sections) {
    await session.goto(`/settings/${id}`);
    const body = await until(
      `settings ${id}`,
      async () => {
        const text = await session.text();
        return hasText(text, heading) ? text : null;
      },
      120000,
    );
    assert.ok(hasText(body, "Settings"), `The ${id} page bar must name Settings`);
    const rows = await session.execute("return document.querySelectorAll('main section, main dl > div').length");
    assert.ok(rows > 0, `The ${id} section must render rows`);
  }

  /* Appearance: the local density preference autosaves, with no Save button anywhere. */
  await session.goto("/settings/appearance");
  await until("appearance", async () => hasText(await session.text(), "Density"), 120000);
  assert.ok(!(await session.exists("button", "Save")), "Settings autosave; no section may carry a Save button");
  await selectDensity(session, "touch");
  await until(
    "touch density applied",
    async () => (await session.execute("return document.documentElement.dataset.density")) === "touch",
    120000,
  );
  await selectDensity(session, "comfortable");
  await until(
    "comfortable density applied",
    async () => (await session.execute("return document.documentElement.dataset.density")) === "comfortable",
    120000,
  );

  /* Storage sources: one row per configured source. */
  await session.goto("/settings/diagnostics");
  const storage = await until(
    "storage sources",
    async () => {
      const state = await session.execute(`
        const table = document.querySelector('table[aria-label="Storage Source Priority"]');
        const rows = table ? [...table.querySelectorAll('tbody tr')].filter(row => !row.querySelector('td[colspan]')) : [];
        return {rows: rows.length, text: document.body.innerText};
      `);
      return state.rows >= 1 || hasText(state.text, "No storage sources are configured") ? state : null;
    },
    120000,
  );
  assert.ok(
    storage.rows >= 1 || hasText(storage.text, "No storage sources are configured"),
    "Storage Sources must render its rows or say there are none",
  );

  /* Plugins: rows with a Settings button, and the plugin page behind it. */
  await session.goto("/settings/plugins");
  const plugins = await until(
    "the plugins section",
    async () => {
      const state = await session.execute(`
        const buttons = [...document.querySelectorAll('table[aria-label="Plugins"] tbody tr button')]
          .filter(button => button.getAttribute('aria-label') === 'Settings');
        const configurable = buttons.filter(button => !button.disabled);
        return {rows: buttons.length, configurable: configurable.length, text: document.body.innerText};
      `);
      return state.rows || hasText(state.text, "Loaded from the launch configuration; managed outside the installation store.")
        ? state
        : null;
    },
    120000,
  );
  if (!plugins.rows) {
    /* Nothing is in the installation store: this backend was launched with
       `--plugin`, and the section has to say where its plugins came from
       rather than leaving the list unexplained. */
    assert.ok(
      hasText(plugins.text, "Loaded from the launch configuration; managed outside the installation store."),
      "With no installed plugin the Plugins section must still name the loaded ones",
    );
    notes.push(
      "settings: the gate launches the backend with --plugin, so the installation store is empty and the plugin row's Settings button is not on this path",
    );
  }
  if (plugins.configurable) {
    await session.click(await session.find("xpath", "//table[@aria-label='Plugins']//button[@aria-label='Settings' and not(@disabled)]"));
    /* Every row, the host's included, opens its plugin's own page. */
    await until("plugin page", async () => (await session.location()).pathname.startsWith("/settings/plugins/"), 120000);
    assert.ok(hasText(await session.text(), "Settings"), "The plugin page must carry its settings breadcrumb");
    await session.goto("/settings/plugins");
    await until("back to plugins", async () => (await session.location()).pathname === "/settings/plugins", 120000);
  } else {
    notes.push("settings: no installed plugin declares settings, so the plugin page stayed shut");
  }

  /* Shared Secrets: Add And Remove Two Masked Keyring Entries. */
  await session.goto("/settings/secrets");
  await until("secret storage ready", () =>
    session.execute(
      "return [...document.querySelectorAll('button')].some(button => button.textContent.trim() === 'Add Secret' && !button.disabled);",
    ),
  );
  for (const [name, value] of [
    ["e2e.settings.first", "native-first-47f93d"],
    ["e2e.settings.second", "native-second-94d2ac"],
  ]) {
    secrets.add(value);
    await session.clickNamed("button", "Add Secret");
    for (const [label, text] of [
      ["Identifier", name],
      ["Label", name],
      ["Value", value],
    ]) {
      const input = await until(`secret field ${label}`, () =>
        session.execute(
          "return [...document.querySelectorAll('label')].find(label => label.textContent.trim() === arguments[0])?.control ?? null;",
          [label],
        ),
      );
      await session.type(input, text);
    }
    await session.clickNamed("button", "Save Secret");
    await until("shared keyring entry saved", () => session.exists("button", `Actions For ${name}`));
    assert.ok(
      await session.find("xpath", `//tr[td[normalize-space(.)='${name}']]//button[@aria-label='Edit' and not(@disabled)]`),
      "The saved secret remains editable",
    );
    assert.ok(!(await session.text()).includes(value), "stored secret values stay masked");
  }
  for (const name of ["e2e.settings.first", "e2e.settings.second"]) {
    await openMenu(session, `Actions For ${name}`);
    await session.clickNamed("menuitem", "Delete");
    await session.clickNamed("button", "Confirm Delete");
    await until("shared keyring entry removed", async () => !(await session.exists("button", `Actions For ${name}`)));
  }

  /* Backend Dialog Validation, Cancelled Deletion And Selection Recovery. */
  await session.goto("/settings/backends");
  await until("backend registry", () => session.exists("button", "Add Backend"));
  const original = await session.execute("return window.__TAURI_INTERNALS__.invoke('desktop_session_status')");
  const endpoint = original.connection.endpoint;
  assert.ok(endpoint);
  const field = (label) => session.find("css selector", `[aria-label="${label}"]`);
  const rowAction = (name, action) =>
    session.find("xpath", `//tr[td[normalize-space(.)='${name}']]//button[@aria-label='${action === "Select" ? "Connect" : action}']`);
  const add = async (name, identity = "") => {
    await session.clickNamed("button", "Add Backend");
    await session.type(await field("Backend Identifier"), name);
    if (identity) {
      const pairing = join(work, `${name}.dokime-remote`);
      await writeFile(
        pairing,
        JSON.stringify({
          version: 1,
          endpoints: [new URL(endpoint).origin],
          backend_identifier: identity,
          label: name,
          ca_certificate_pem: null,
          access_key: null,
        }),
      );
      await session.openPicker(await session.find("xpath", "//*[@role='dialog']//button[normalize-space(.)='Import Remote']"));
      const picker = { window: null };
      await until("native identity pairing import", () => chooseNativeFile(picker, pairing), 120000);
      await until("native identity import completes", () =>
        session.execute(`
        return globalThis.__dokimeInvokes.some(entry => entry.command === 'desktop_backend_import_pairing' && entry.state !== 'pending');
      `),
      );
      await until("identity mismatch refuses the imported connection", async () => {
        const status = await session.execute("return window.__TAURI_INTERNALS__.invoke('desktop_session_status')");
        return status.connection.phase === "failed";
      });
      await session.goto("/settings/backends");
    } else {
      await session.type(await session.find("xpath", "//*[@role='dialog']//input[@id=//label[normalize-space(.)='Label']/@for]"), name);
      await session.type(await field("Backend Endpoint"), endpoint);
      await session.click(await session.find("xpath", "//*[@role='dialog']//button[normalize-space(.)='Add Backend']"));
    }
    await until(`saved backend ${name}`, () => rowAction(name, "Select"));
  };
  await session.clickNamed("button", "Add Backend");
  await session.type(await field("Backend Identifier"), "native-invalid");
  await session.type(
    await session.find("xpath", "//*[@role='dialog']//input[@id=//label[normalize-space(.)='Label']/@for]"),
    "native-invalid",
  );
  await session.type(await field("Backend Endpoint"), "http://operator:private@127.0.0.1/");
  await session.click(await session.find("xpath", "//*[@role='dialog']//button[normalize-space(.)='Add Backend']"));
  await until("invalid origin refused", () =>
    session.execute(`
    const field = document.querySelector('[role="dialog"] input[aria-label="Backend Endpoint"]');
    const explanation = document.getElementById(field?.getAttribute('aria-describedby'));
    return field?.getAttribute('aria-invalid') === 'true' && Boolean(explanation?.textContent.trim());
  `),
  );
  const unchanged = await session.execute("return window.__TAURI_INTERNALS__.invoke('desktop_backends_list')");
  assert.ok(!unchanged.profiles.some((profile) => profile.identifier === "native-invalid"));
  await session.clickNamed("button", "Cancel");

  await add("native-recovery");
  await add("native-stale", "deliberately-wrong-native-identity");
  await openMenu(session, "Actions For native-stale");
  await session.clickNamed("menuitem", "Delete");
  await session.clickNamed("button", "Cancel");
  assert.ok(await rowAction("native-stale", "Select"));
  await session.click(await rowAction("native-recovery", "Select"));
  await until("saved selection connected", async () => {
    const status = await session.execute("return window.__TAURI_INTERNALS__.invoke('desktop_session_status')");
    return status.connection.phase === "connected";
  });
  await session.goto("/settings/backends");
  await until("registry after selection", () => rowAction("native-stale", "Select"));
  await session.click(await rowAction("native-stale", "Select"));
  await until("failed selection displays the registry", async () => {
    const status = await session.execute("return window.__TAURI_INTERNALS__.invoke('desktop_session_status')");
    if (status.connection.phase !== "failed") return false;
    return hasText(await toastText(session), "Backend Connection Failed") && (await rowAction("native-recovery", "Select"));
  });
  const afterFailure = await session.execute("return window.__TAURI_INTERNALS__.invoke('desktop_backends_list')");
  assert.equal(afterFailure.default, "native-recovery");
  await session.click(await rowAction("native-recovery", "Select"));
  await until("explicit profile recovery", async () => {
    const status = await session.execute("return window.__TAURI_INTERNALS__.invoke('desktop_session_status')");
    return status.connection.phase === "connected";
  });
  await session.goto("/settings/backends");
  await openMenu(session, "Actions For native-stale");
  await session.clickNamed("menuitem", "Delete");
  await session.clickNamed("button", "Confirm Delete");
  await until("stale profile removed", async () => {
    const document = await session.execute("return window.__TAURI_INTERNALS__.invoke('desktop_backends_list')");
    return !document.profiles.some((profile) => profile.identifier === "native-stale");
  });

  /* Diagnostics: read-only paths with Copy and Open, then the bundle itself. */
  await session.goto("/settings/diagnostics");
  const diagnostics = await until(
    "diagnostics",
    async () => {
      const text = await session.text();
      return hasText(text, "Data Directory") ? text : null;
    },
    120000,
  );
  assert.ok(hasText(diagnostics, "Diagnostics Bundle"), "Diagnostics must offer the bundle row");
  assert.ok(await session.exists("button", "Copy Path"), "A path row must offer Copy");
  if (!(await session.exists("button", "Open Path"))) {
    notes.push("settings: Open Path is absent, which only a remote backend would explain");
  }

  await session.clickNamed("button", "Download Bundle");
  let announcement = "";
  /* The archive is written by the shell itself into the operating system's
     Downloads folder, so the evidence is the file, not a tapped payload. */
  const bundle = await until(
    "diagnostics bundle saved",
    async () => {
      announcement = (await toastText(session)) || announcement;
      const path = await session.execute("return globalThis.__dokimeDiagnosticsPath ?? null");
      return path && existsSync(path) ? path : null;
    },
    120000,
  ).catch((failure) => {
    throw new Error(`${failure.message}. The application said: ${announcement || "nothing at all"}`);
  });
  assert.match(basename(bundle), /^dokime-diagnostics-.*\.zip$/);
  const retained = join(session.downloads, basename(bundle));
  if (resolve(bundle) !== resolve(retained)) await copyFile(bundle, retained);
}

/** Shared Journey Finalizer: Native Sealed Pull, Saved File Reopen And Artifact Delivery. */
async function finalizeStoryReports(name, executionIds) {
  const exemptions = JSON.parse(await readFile(join(workspace, "crates/dokime-runtime/tests/support/system_reports.json"), "utf8"));
  if (Object.hasOwn(exemptions.desktop, name)) {
    assert.deepEqual(executionIds, [], "A report exemption must not hide owned executions");
    process.stdout.write(`   execution report exempt: ${exemptions.desktop[name]}\n`);
    return;
  }
  assert.ok(executionIds.length > 0, "Report-producing scene must declare its executions");
  assert.equal(new Set(executionIds).size, executionIds.length, "A scene owns each execution once");
  const reopenedSession = !session || session.closed;
  if (reopenedSession) {
    session = await startSession(work, backend?.endpoint, evidence);
    await session.connected();
  }
  let saved = 0;
  const inventory = [...executionIds];
  const deliver = async (request, destination) =>
    measured("report-export", request.source.kind, async () => {
      let settled = false;
      const pending = session.executeAsync(
        `
      const request = arguments[0];
      const done = arguments[arguments.length - 1];
      const invoke = globalThis.__TAURI_INTERNALS__.invoke;
      invoke('desktop_session_status').then(status =>
        invoke('desktop_artifact_deliver', {generation: status.connection.generation, request})
      ).then(value => done({value}), error => done({failure: String(error)}));
    `,
        [request],
      );
      const picker = { window: null };
      const [delivery, selection] = await Promise.allSettled([
        pending.finally(() => {
          settled = true;
        }),
        until("native artifact save", () => settled || chooseNativeFile(picker, destination, "Save"), 120000, 25),
      ]);
      if (delivery.status === "rejected") throw delivery.reason;
      if (selection.status === "rejected") throw selection.reason;
      const response = delivery.value;
      assert.equal(response.failure, undefined, response.failure);
      assert.ok(response.value?.path);
      return response.value;
    });
  try {
    for (const id of executionIds) {
      assert.ok(!finalizedReports.has(id), "Execution reports have one scene owner");
      const path = join(work, "story-reports", name, id, "report.html");
      await mkdir(dirname(path), { recursive: true });
      const delivered = await deliver(
        {
          source: { kind: "execution_report", executionId: id },
          kind: "html_report",
          filename: "report.html",
          action: "save",
        },
        path,
      );
      assert.equal(delivered.origin, "sealed");
      assert.equal(await realpath(delivered.path), await realpath(path));
      const html = await readFile(path, "utf8");
      const marker = '<script id="hub-report-data" type="application/json">';
      const report = JSON.parse(html.split(marker)[1].split("</script>")[0]);
      assert.deepEqual(
        report.test_executions.map((execution) => execution.test_execution_id),
        [id],
      );
      const artifacts = report.snapshot.artifacts;
      assert.ok(Array.isArray(artifacts));
      assert.equal(new Set(artifacts.map((artifact) => artifact.artifact_id)).size, artifacts.length);
      for (const artifact of artifacts) {
        assert.equal(artifact.href, `outputs/${artifact.path}`);
        assert.equal(artifact.downloadHref, artifact.href);
        assert.ok(artifact.path.split("/").every((part) => part && part !== "." && part !== ".." && !/[\\:]/.test(part)));
        const destination = join(dirname(path), artifact.href);
        await mkdir(dirname(destination), { recursive: true });
        const content = await deliver(
          {
            source: { kind: "artifact", artifactId: artifact.artifact_id },
            kind: "artifact",
            filename: artifact.path.split("/").at(-1),
            action: "save",
          },
          destination,
        );
        assert.equal(await realpath(content.path), await realpath(destination));
        const file = await open(destination, "r");
        try {
          assert.equal((await file.stat()).size, artifact.size_bytes);
        } finally {
          await file.close();
        }
      }
      // Open The Saved Copy After Its Relative Artifact Links Have Been Published.
      const reopened = await measured("report-reopen", id, () =>
        fetch(reportBrowser.endpoint, {
          method: "POST",
          headers: { "content-type": "application/json" },
          body: JSON.stringify({ path }),
        }),
      );
      assert.ok(reopened.ok);
      await cp(dirname(path), join(session.downloads, name, id), { recursive: true });
      saved += 1;
      finalizedReports.add(id);
      pendingReports.delete(id);
    }
    await session.dump(`report-inventory-${name}`, inventory);
    assert.ok(saved > 0, "Report-producing story needs a sealed execution; exempt no-report stories centrally");
    process.stdout.write(`   reopened ${saved} sealed execution reports with their artifacts\n`);
  } finally {
    if (reopenedSession) await closeSession();
  }
}

/**
 * Touch density: the same surfaces, driven through the real Appearance switch.
 *
 * At touch the sidebar is a rail, so the cycle's cases are only reachable
 * through the header's Executions sheet, and the template dialog becomes a
 * full-width sheet. Those three are what this pass is for.
 */
async function touchJourney(session, cycleId) {
  await setDensity(session, densities.touch);

  await session.goto("/cycles");
  const home = await until(
    "touch cycles home",
    async () => {
      const text = await session.text();
      return hasText(text, "Recent") ? text : null;
    },
    120000,
  );
  assert.ok(hasText(home, "Needs Attention"), "The groups keep their order at touch");

  await session.goto(`/cycles/${cycleId}?view=cycle`);
  await until("touch workspace", () => session.exists("button", "Cycle Actions"), 120000);
  /* The rail: collapsed at touch, with the children behind it. */
  if (await session.exists("button", "Collapse Sidebar")) {
    await session.clickNamed("button", "Collapse Sidebar");
  }
  await until("collapsed rail", () => session.exists("button", "Expand Sidebar"), 120000);
  const railChildren = await session.execute("return document.querySelectorAll('nav[aria-label=Navigation] li[data-level=case]').length;");
  assert.equal(railChildren, 0, "A collapsed rail must hide its children");

  /* The sidebar tree is the one execution switcher: expanding the rail
     brings the rows back, at the touch row height. */
  await session.clickNamed("button", "Expand Sidebar");
  const touchRows = await until("expanded tree", () =>
    session.execute(`
      const rows = [...document.querySelectorAll('nav[aria-label=Navigation] li[data-level=case] > div')];
      return rows.length ? rows.map((row) => Math.round(row.getBoundingClientRect().height)) : null;
    `),
  );
  assert.ok(
    touchRows.every((height) => height >= 44),
    `Touch tree rows must be at least 44px tall: ${JSON.stringify(touchRows)}`,
  );

  await session.goto("/library");
  await until("touch library", async () => hasText(await session.text(), PROTECTION_SUITE), 120000);
  await session.clickNamed("button", PROTECTION_TEMPLATE);
  const share = await until("touch dialog", () =>
    session.execute(`
      const panel = document.querySelector('[data-slot=sheet-content], [role=dialog]');
      return panel
        ? Math.round((panel.getBoundingClientRect().width / window.innerWidth) * 100)
        : null;
    `),
  );
  assert.ok(share >= 80, `At touch the dialog must be a full-width sheet (measured ${share}% of the viewport)`);
  await session.key("Escape");
  await session.waitForDom("the touch dialog closes", `return !document.querySelector('[data-slot=sheet-content], [role=dialog]');`);

  await setDensity(session, densities.desktop);
}

/**
 * Run one journey, unless the filter excludes it.
 *
 * A skipped journey is neither a pass nor a failure: it produced no evidence,
 * so it states that and nothing else. `startup` and `library` ignore the
 * filter because everything after them addresses what they establish.
 */
async function journey(name, run, reports = () => []) {
  timingScene = name;
  const started = Date.now();
  if (
    (workflow && !workflow.journeys.has(name)) ||
    (selectedJourneys.size && !selectedJourneys.has(name) && !REQUIRED_JOURNEYS.has(name))
  ) {
    process.stdout.write(`\n== ${name}\n   excluded from the selected journeys or workflow\n`);
    results.push({ name, seconds: 0, skipped: "Excluded from the selected journeys or workflow" });
    return { ok: true, skipped: true };
  }
  process.stdout.write(`\n== ${name}\n`);
  if (webviewGone) {
    const reason = `skipped: the WebView process exited during the ${webviewGone} journey`;
    failures.push({ kind: "failure", name, reason });
    results.push({ name, seconds: 0, failure: reason });
    process.stdout.write(`   ${reason}\n`);
    return { ok: false };
  }
  try {
    const value = await traceSpan(name, "scene", async () => {
      const result = await measured("scene-behavior", name, run);
      for (const id of result?.executionIds ?? []) pendingReports.add(id);
      for (const id of result?.unrun ?? []) pendingReports.delete(id);
      await measured("reports", name, () => finalizeStoryReports(name, reports(result)));
      return result;
    });
    passedJourneys.push(name);
    results.push({ name, seconds: (Date.now() - started) / 1000 });
    process.stdout.write("   passed\n");
    return { ok: true, value };
  } catch (failure) {
    /* A dead session is one crash, not one failure per journey after it: the
       journeys behind it never ran, and saying they failed would bury the
       crash under its own consequences. */
    if (/invalid session id|Session terminated without a reply/.test(failure.message)) {
      webviewGone = name;
    }
    failures.push({ kind: "failure", name, reason: redact(failure.message) });
    results.push({ name, seconds: (Date.now() - started) / 1000, failure: redact(failure.message) });
    process.stdout.write(`   FAILED: ${redact(failure.message)}\n`);
    if (session && !session.closed) {
      try {
        const shot = await session.screenshot(`failure-${name}`);
        const failureBody = await session.text().catch(() => session.lastBody ?? "");
        await writeFile(join(evidence, `failure-${name}.txt`), redact(`${failure.stack ?? failure.message}\n\n${failureBody}`));
        /* What the application asked the shell to do, and how long each call
           took: a journey that waits on something that never answers says so
           here and nowhere else. */
        await writeFile(join(evidence, `invokes-${name}.json`), JSON.stringify(await drainInvokeLedger(session), null, 2));
        process.stdout.write(`   evidence: ${shot}\n`);
      } catch {
        // A dead WebView has no DOM left to photograph.
      }
    }
    return { ok: false };
  } finally {
    await reportBrowser?.closeTabs();
    const seconds = (Date.now() - started) / 1000;
    process.stdout.write(`   timing: ${name} ${seconds.toFixed(3)} s\n`);
    if (evidence) await writeFile(join(evidence, "timings.json"), JSON.stringify(timings, null, 2));
  }
}

async function terminate(child, signal = "SIGTERM") {
  if (!child?.pid || child.exitCode !== null || child.signalCode !== null) return;
  if (windows) {
    try {
      await executeFile("taskkill.exe", ["/PID", String(child.pid), "/T", "/F"], { windowsHide: true });
    } catch (error) {
      if (child.exitCode !== null || child.signalCode !== null) return;
      try {
        process.kill(child.pid, 0);
      } catch (probe) {
        if (probe.code === "ESRCH") return;
      }
      throw error;
    }
    return;
  }
  try {
    process.kill(-child.pid, signal);
  } catch (error) {
    if (error.code !== "ESRCH") throw error;
  }
}

async function sha256(path) {
  const hash = createHash("sha256");
  for await (const chunk of createReadStream(path)) hash.update(chunk);
  return hash.digest("hex");
}

async function installWindowsCandidate() {
  if (process.env.DOKIME_E2E_DISPOSABLE_WINDOWS !== "1") {
    throw new Error("Installer acceptance requires a disposable interactive Windows account: set DOKIME_E2E_DISPOSABLE_WINDOWS=1 there.");
  }
  if (process.env.DOKIME_E2E_DEV_URL || process.env.DOKIME_E2E_SHELL) {
    throw new Error(
      "Windows acceptance drives the installed shell and backend with bundled frontend assets; development overrides are invalid.",
    );
  }
  const installer = process.env.DOKIME_E2E_INSTALLER;
  if (!installer || !existsSync(installer)) throw new Error("DOKIME_E2E_INSTALLER must name the candidate NSIS installer.");
  const checks = [
    ["installer", installer, process.env.DOKIME_E2E_INSTALLER_SHA256],
    ["shell", join(work, "installation", "dokime-gui.exe"), process.env.DOKIME_E2E_SHELL_SHA256],
    ["cli", join(work, "installation", "dokime.exe"), process.env.DOKIME_E2E_CLI_SHA256],
  ];
  for (const [label, , expected] of checks) {
    if (!/^[a-f0-9]{64}$/.test(expected ?? "")) throw new Error(`Missing candidate SHA-256 for ${label}.`);
  }
  if (!process.env.DOKIME_E2E_PLUGIN || !/^[a-f0-9]{64}$/.test(process.env.DOKIME_E2E_PLUGIN_SHA256 ?? "")) {
    throw new Error("Windows acceptance requires the candidate OBC fixture path and DOKIME_E2E_PLUGIN_SHA256.");
  }
  assert.equal(await sha256(pluginBundle), process.env.DOKIME_E2E_PLUGIN_SHA256, "the fixture matches the selected candidate");
  assert.equal(await sha256(installer), checks[0][2], "installer matches the selected candidate");
  installation = join(work, "installation");
  installerAttempted = true;
  await executeFile(resolve(installer), ["/S", `/D=${installation}`], {
    windowsHide: true,
    windowsVerbatimArguments: true,
    timeout: 120000,
  });
  const hashes = {};
  for (const [label, path, expected] of checks.slice(1)) {
    hashes[label] = await sha256(path);
    assert.equal(hashes[label], expected, `installed ${label} matches the selected candidate`);
  }
  shellBinary = checks[1][1];
  cliBinary = checks[2][1];
  backendBinary = cliBinary;
  await writeFile(
    join(evidence, "installation.json"),
    JSON.stringify(
      {
        installer: resolve(installer),
        sha256: checks[0][2],
        hashes,
        plugin: { path: resolve(pluginBundle), sha256: process.env.DOKIME_E2E_PLUGIN_SHA256 },
      },
      null,
      2,
    ),
  );
}

async function closeSession(...args) {
  return measured("gui-shutdown", "closeSession", () => closeSessionMeasured(...args));
}

async function closeSessionMeasured() {
  if (!session) {
    await terminate(driverProcess);
    driverProcess = undefined;
    return;
  }
  const owned = session;
  await owned.close();
  await owned.request("DELETE", "").catch((error) => {
    if (!/invalid session id/.test(error.message)) throw error;
  });
  await terminate(owned.process);
  driverProcess = undefined;
  session = undefined;
}

async function stopBackend(...args) {
  return measured("backend-shutdown", "stopBackend", () => stopBackendMeasured(...args));
}

async function stopBackendMeasured() {
  if (!backend) return;
  const owned = backend;
  if (owned.child.startupError) throw owned.child.startupError;
  owned.child.stdin.end();
  const exited =
    owned.child.exitCode === null ? once(owned.child, "exit") : Promise.resolve([owned.child.exitCode, owned.child.signalCode]);
  const timer = new AbortController();
  try {
    const [code, signal] = await Promise.race([
      exited,
      delay(120000, undefined, { signal: timer.signal }).then(() => {
        throw new Error("Native backend did not drain within its 120 second deadline");
      }),
    ]);
    assert.equal(signal, null, "native backend drains without a forced signal");
    assert.equal(code, 0, "native backend drains successfully");
  } finally {
    timer.abort();
  }
  await owned.log.close();
  backend = undefined;
}

/** Create An Empty Container Through The Native Form */
async function createEmptyContainer(session, title) {
  await session.goto("/cycles");
  await session.clickNamed("button", "New Cycle");
  const input = await until("the new cycle form", () => session.find("css selector", "#new-cycle-title"));
  await session.type(input, title);
  assert.equal(await session.execute("return arguments[0].value", [input]), title, "Native typing replaces the cycle title");
  await session.clickNamed("button", "Create Cycle");
  return until("the created cycle route", async () => /^\/cycles\/([^/]+)$/.exec((await session.location()).pathname)?.[1] ?? null);
}

async function ownedStartupJourney() {
  session = await startSession(work, undefined, evidence);
  await session.connected();
  const status = await session.executeAsync(`
    const done = arguments[arguments.length - 1];
    globalThis.__TAURI_INTERNALS__.invoke('desktop_session_status').then(done, (error) => done({error: String(error)}));
  `);
  assert.equal(status.ownership, "owned_local", "the native shell owns its embedded backend");
  assert.ok(status.connection?.endpoint, "the embedded session reports its negotiated endpoint");
  assert.ok(await listening(status.connection.endpoint), "the embedded server is listening");
  const cycleId = await createEmptyContainer(session, "Managed Shutdown Evidence");
  await cyclePhase(session, "Active");
  await session.screenshot("owned-startup");
  await installSaveTap(session);
  await installPluginThroughPicker(session, hostBundle);
  await installPluginThroughPicker(session);
  const closing = await session.executeAsync(`
    const done = arguments[arguments.length - 1];
    globalThis.__TAURI_INTERNALS__.invoke('desktop_session_status').then(done, (error) => done({error: String(error)}));
  `);
  assert.equal(closing.ownership, "owned_local", "the cycle's current backend remains owned by this shell");
  assert.ok(closing.connection?.endpoint, "the current embedded session has an endpoint before window close");
  assert.ok(await listening(closing.connection.endpoint), "the current owned listener is live immediately before window close");
  const idleStarted = performance.now();
  await closeSession();
  await until("owned backend drains on window close", async () => !(await listening(closing.connection.endpoint)), 120000);
  const idleMs = performance.now() - idleStarted;
  session = await startSession(work, undefined, evidence);
  await session.connected();
  await emptyCycleHistory(session, cycleId, "Managed Shutdown Evidence", "owned_local");
  const runningCycle = await prepareCycle(session, "Owned Shutdown Campaign", [[STARTUP_SUITE, STARTUP_TEMPLATE]]);
  assert.ok(await startCaseFromItsRow(session, STARTUP_TEMPLATE), "The owned backend starts its installed plugin");
  await until("owned execution awaits its operator", () => startupPromptOpen(session), 120000);
  const running = await session.executeAsync(`
    const done = arguments[arguments.length - 1];
    globalThis.__TAURI_INTERNALS__.invoke('desktop_session_status').then(done, (error) => done({error: String(error)}));
  `);
  assert.equal(running.ownership, "owned_local");
  const runningStarted = performance.now();
  await closeSession();
  await until("running owned backend drains on close", async () => !(await listening(running.connection.endpoint)), 120000);
  const runningMs = performance.now() - runningStarted;
  session = await startSession(work, undefined, evidence);
  await session.connected();
  await session.goto(`/history/${runningCycle.id}`);
  /* The saved cycle lists its executions in one table; the stopped one is cancelled, never interrupted. */
  await until("cancelled owned execution survives restart", async () =>
    session.execute(
      `
    const table = document.querySelector('[aria-label="History Executions"]');
    const row = table && [...table.querySelectorAll('tbody tr')].find((node) => node.innerText.includes(arguments[0]));
    const text = row?.innerText.toLowerCase() ?? '';
    return text.includes('cancelled') && !text.includes('interrupted');
  `,
      [STARTUP_TEMPLATE],
    ),
  );
  assert.ok(await session.exists("button", "Open Report"), "Orderly close preserves the sealed cycle report");
  await session.screenshot("owned-running-recovered");
  await writeFile(join(evidence, "owned-startup.json"), JSON.stringify({ initial: status, closed: closing, idleMs, runningMs }, null, 2));
  return { executionIds: runningCycle.executionIds, reports: runningCycle.executionIds };
}

async function idleJourney() {
  await session.goto("/library");
  await installSaveTap(session);
  await until("loaded Library catalog", async () => hasText(await session.text(), STARTUP_SUITE));
  const library = await idleInvokes("Library");
  const cycleId = await createEmptyContainer(session, "Idle Refresh Evidence");
  await cyclePhase(session, "Active");
  await until("idle cycle admission ready", () => session.named("button", "Seal Cycle"));
  const cycle = await idleInvokes("Active Cycle");
  await writeFile(join(evidence, "idle-invokes.json"), JSON.stringify({ library, cycleId, cycle }, null, 2));
  assert.deepEqual(library.queries, [], "An idle mounted Library does not poll backend queries");
  assert.deepEqual(cycle.queries, [], "An idle mounted Active Cycle does not poll backend queries");
}

async function idleInvokes(surface) {
  await until(`idle ${surface} requests settle`, () =>
    session.execute(`
    const calls = globalThis.__dokimeInvokes;
    const pending = calls.some((entry) => entry.state === 'pending');
    return !pending;
  `),
  );
  const started = await session.execute("return Date.now();");
  await measured("fixed-wait", "UI settling (10000 ms)", () => delay(10000));
  const observation = await session.execute(
    `
    const calls = globalThis.__dokimeInvokes.filter((entry) => entry.at >= arguments[0]);
    const counts = {};
    for (const entry of calls) {
      const key = entry.path || entry.command;
      counts[key] = (counts[key] || 0) + 1;
    }
    return {elapsedMs: Date.now() - arguments[0], counts,
      queries: calls.filter((entry) => entry.command === 'rpc').map((entry) => entry.path)};
  `,
    [started],
  );
  return observation;
}

async function emptyCycleHistory(session, cycleId, title, ownership) {
  const previous = await session.executeAsync(`
    const done = arguments[arguments.length - 1];
    globalThis.__TAURI_INTERNALS__.invoke('desktop_session_status').then(done, (error) => done({error: String(error)}));
  `);
  // Discover The Saved Container Through The Mounted History Library
  await session.goto("/history");
  const opener = await until("the empty sealed cycle in History", () =>
    session.find("xpath", `//tbody//button[normalize-space(.)=${xpathLiteral(title)}]`),
  );
  const row = await session.execute("return arguments[0].closest('tr').innerText;", [opener]);
  assert.match(row, /sealed/i, "the persisted header retains its sealed phase");
  await session.click(opener);
  await until("the matching saved cycle route", async () => (await session.location()).pathname === `/history/${cycleId}`);
  await until("an empty saved container without invented executions", async () =>
    hasText(await session.text(), "This sealed cycle contains no executions."),
  );

  // Open The Actual Saved Report Through Native Delivery
  const openedBefore = await session.execute(
    "return (globalThis.__dokimeInvokes ?? []).filter((entry) => entry.command === 'desktop_artifact_deliver').length;",
  );
  await session.clickNamed("button", "Open Report");
  await until("the saved empty-cycle report opened", () =>
    session.execute(
      `
    const opened = (globalThis.__dokimeInvokes ?? []).filter((entry) => entry.command === 'desktop_artifact_deliver');
    const button = [...document.querySelectorAll('button')].find((candidate) => candidate.innerText.trim() === 'Open Report');
    return opened.length > arguments[0] && opened.at(-1).state === 'settled' && button && !button.disabled;
  `,
      [openedBefore],
    ),
  );
  assert.equal((await session.findAll("css selector", "[role=alert]")).length, 0, "saved report delivery has no failure");

  // Verify The Same Sealed Evidence Is Exportable After Restart
  // A Full Route Load Can Replace The Owned Companion And Its Listener.
  const current = await session.executeAsync(`
    const done = arguments[arguments.length - 1];
    globalThis.__TAURI_INTERNALS__.invoke('desktop_session_status').then(done, (error) => done({error: String(error)}));
  `);
  assert.equal(current.ownership, ownership, "history retains its selected backend ownership");
  assert.ok(current.connection?.backend?.epoch, "the current native session reports a connected backend epoch");
  const endpoint = current.connection.endpoint;
  assert.ok(await listening(endpoint), "the current native backend accepts connections before export");
  await writeFile(
    join(evidence, `empty-cycle-${cycleId}-connection.json`),
    JSON.stringify(
      {
        ownership: current.ownership,
        before: { endpoint: previous.connection?.endpoint, epoch: previous.connection?.backend?.epoch },
        after: { endpoint, epoch: current.connection.backend.epoch },
      },
      null,
      2,
    ),
  );
  const archive = join(evidence, `empty-cycle-${cycleId}.zip`);
  const result = await executeFile(cliBinary, ["cycle", "export", cycleId, "--endpoint", endpoint, "--output", archive, "--json"], {
    windowsHide: true,
    cwd: work,
    timeout: 120000,
  });
  const receipt = JSON.parse(result.stdout);
  const bytes = await readFile(archive);
  assert.equal(receipt.size_bytes, bytes.length);
  assert.equal(receipt.sha256, createHash("sha256").update(bytes).digest("hex"));
  assert.equal(bytes.readUInt32LE(0), 0x04034b50, "saved evidence is a ZIP archive");
}

async function restartJourney(recoveredReports) {
  const activeCycleId = await createEmptyContainer(session, "Server Restart Evidence");
  await cyclePhase(session, "Active");
  const previous = await session.executeAsync(`
    const done = arguments[arguments.length - 1];
    globalThis.__TAURI_INTERNALS__.invoke('desktop_session_status').then(done, (error) => done({error: String(error)}));
  `);
  assert.ok(previous.connection?.backend?.epoch, "the first native connection reports its backend epoch");
  const endpoint = backend.endpoint;
  await closeSession();
  assert.ok(await listening(endpoint), "closing the attached shell leaves its backend alive");
  await stopBackend();
  backend = await startBackend(work, evidence, endpoint, "backend-restart", false);
  session = await startSession(work, endpoint, evidence);
  await session.connected();
  const current = await session.executeAsync(`
    const done = arguments[arguments.length - 1];
    globalThis.__TAURI_INTERNALS__.invoke('desktop_session_status').then(done, (error) => done({error: String(error)}));
  `);
  assert.ok(current.connection?.backend?.epoch, "the restarted native connection reports its backend epoch");
  assert.notEqual(
    current.connection.backend.epoch,
    previous.connection.backend.epoch,
    "the relaunched shell establishes the restarted backend epoch",
  );
  assert.equal(current.ownership, "attached");
  await emptyCycleHistory(session, activeCycleId, "Server Restart Evidence", "attached");
  await session.goto("/history");
  await until("saved evidence survives native restart", async () => (await session.findAll("css selector", "tbody tr")).length > 0, 120000);
  await setDensity(session, densities.desktop);
  const cycle = await prepareCycle(session, "Restart Campaign", [
    [PROTECTION_SUITE, PROTECTION_TEMPLATE],
    [STARTUP_SUITE, STARTUP_TEMPLATE],
  ]);
  const recovered = await cycleJourney(session, cycle.id);
  await session.screenshot("native-restart");
  return {
    ...recovered,
    executionIds: [...cycle.executionIds, ...recovered.executionIds],
    reports: [...recoveredReports, ...recovered.reports],
  };
}

async function pluginInstallJourney() {
  await installSaveTap(session);
  await installPluginThroughPicker(session);
  const endpoint = backend.endpoint;
  await stopBackend();
  backend = await startBackend(work, evidence, endpoint, "plugin-activation", false);
  await session.goto("/settings/plugins", { reload: true });
  await until(
    "installed bundle active after restart",
    () =>
      session.execute(`
    const row = [...document.querySelectorAll('table[aria-label="Plugins"] tbody tr')]
      .find(row => row.innerText.includes('plugin-obc'));
    return Boolean(row && [...row.querySelectorAll('[data-slot="badge"]')].some(badge => badge.innerText === 'Active'));
  `),
    120000,
  );
  await session.goto("/library");
  await until("installed bundle contributes its catalog", async () => hasText(await session.text(), PROTECTION_SUITE), 120000);
  await session.screenshot("plugin-activation");
}

async function installPluginThroughPicker(session, bundle = pluginBundle) {
  await session.goto("/settings/plugins");
  await session.clickNamed("button", "Install Plugin");
  await session.execute("globalThis.__dokimeInvokes.length = 0;");
  await session.openPicker(await session.named("button", "Choose File"));
  const picker = { window: null };
  await until("native plugin file picker", () => chooseNativeFile(picker, bundle), 120000);
  const installed = await until(
    "native plugin installation completes",
    () =>
      session.execute(`
    const entry = globalThis.__dokimeInvokes.find(entry => entry.command === 'desktop_plugin_install_file');
    return entry && entry.state !== 'pending' ? {entry} : null;
  `),
    300000,
  );
  assert.equal(
    installed.entry.state,
    "settled",
    `Native plugin installation failed: ${installed.entry.error ?? installed.entry.nativeResponse}`,
  );
  await until("saved plugin installation", () => session.exists("button", "Later"), 300000);
  const invokes = await session.execute(
    "return globalThis.__dokimeInvokes.filter(entry => entry.command === 'desktop_plugin_install_file');",
  );
  assert.equal(invokes.length, 1, "A bundle uses one native installation invocation");
  assert.deepEqual(
    invokes[0].keys.sort(),
    ["drop", "generation", "progress"],
    "Installation passes control arguments without bundle bytes",
  );
  assert.ok(invokes[0].size < 1024, "The installation invocation stays bounded independently of bundle size");
  await session.clickNamed("button", "Later");
  await until("plugin restart requirement", async () => hasText(await session.text(), "Restart Required"), 120000);
  await session.screenshot("plugin-install");
}

/** One Owned Automation Host Reuses Its CLR And Native Providers Across Pickers. */
async function windowsPicker(script, environment) {
  if (!nativePickerWorker) {
    const boundary = script.indexOf("      $filenameId =");
    const initialization = script.slice(0, boundary);
    const selection = script.slice(boundary).replaceAll("exit 0", "return");
    const server = `
      [Console]::InputEncoding = New-Object System.Text.UTF8Encoding($false)
      [Console]::OutputEncoding = New-Object System.Text.UTF8Encoding($false)
      ${initialization}
      $owners = @{}
      while ($line = [Console]::ReadLine()) {
        try {
          $request = $line | ConvertFrom-Json
          foreach ($property in $request.PSObject.Properties) {
            [Environment]::SetEnvironmentVariable($property.Name, [string]$property.Value)
          }
          $outcome = & {${selection}}
          [Console]::Out.WriteLine((@{value=($outcome -join "")} | ConvertTo-Json -Compress))
        } catch {
          [Console]::Out.WriteLine((@{failure=$_.Exception.Message} | ConvertTo-Json -Compress))
        }
      }`;
    const child = spawn("powershell.exe", ["-NoProfile", "-NonInteractive", "-Command", server], {
      windowsHide: true,
      stdio: ["pipe", "pipe", "pipe"],
    });
    const lines = createInterface({ input: child.stdout, crlfDelay: Infinity });
    const worker = { child, lines, responses: lines[Symbol.asyncIterator](), diagnostics: "", busy: false };
    worker.closed = once(child, "exit").then(([code]) => {
      throw new Error(`Native picker host exited (${code})`);
    });
    worker.closed.catch(() => {});
    child.stdin.on("error", () => {});
    child.stderr.on("data", (chunk) => {
      worker.diagnostics = (worker.diagnostics + chunk).slice(-65536);
    });
    nativePickerWorker = worker;
  }
  const worker = nativePickerWorker;
  assert.ok(!worker.busy, "Native picker requests remain serial");
  worker.busy = true;
  const timer = new AbortController();
  try {
    worker.child.stdin.write(`${JSON.stringify(environment)}\n`);
    const line = await Promise.race([
      worker.responses.next(),
      worker.closed,
      delay(120000, undefined, { signal: timer.signal }).then(() => {
        throw new Error("Native picker host did not answer");
      }),
    ]);
    assert.ok(!line.done, "Native picker host returns its selection");
    const response = JSON.parse(line.value);
    assert.equal(response.failure, undefined, response.failure);
    const diagnostics = worker.diagnostics;
    worker.diagnostics = "";
    return { stdout: response.value, stderr: diagnostics };
  } catch (error) {
    const failure = new Error(`${error.message}${worker.diagnostics ? `\n${worker.diagnostics}` : ""}`, { cause: error });
    await stopNativePicker();
    throw failure;
  } finally {
    timer.abort();
    worker.busy = false;
  }
}

async function stopNativePicker() {
  const worker = nativePickerWorker;
  if (!worker) return;
  nativePickerWorker = undefined;
  worker.child.stdin.end();
  await terminate(worker.child);
  worker.lines.close();
}

async function chooseNativeFile(...args) {
  return measured("native-picker", "chooseNativeFile", () => chooseNativeFileMeasured(...args));
}

async function chooseNativeFileMeasured(picker, file, operation = "Open", owner = driverProcess) {
  if (!windows) {
    if (picker.failure) throw picker.failure;
    const children = await readFile(`/proc/${owner.pid}/task/${owner.pid}/children`, "utf8");
    const executable = await realpath(shellBinary);
    for (const pid of children.trim().split(/\s+/)) {
      if (!pid || (await realpath(`/proc/${pid}/exe`)) !== executable) continue;
      const run = (args) => executeFile("xdotool", args, { windowsHide: true, timeout: 120000 });
      const found = await run([
        "search",
        "--all",
        "--onlyvisible",
        "--pid",
        pid,
        "--name",
        operation === "Save" ? "^Save( File| As)?$" : "^Open File$",
      ]).catch((error) => {
        if (error.code === 1) return null;
        throw error;
      });
      if (!found) return picker.submitted === true;
      const ids = found.stdout.trim().split(/\s+/);
      assert.equal(ids.length, 1, "The owned desktop exposes exactly one native file picker");
      if (picker.window !== null) assert.equal(ids[0], picker.window, "Selection remains with its original native dialog");
      if (picker.submitted) return false;

      const action = picker.window === null ? "ready" : picker.entered ? "open" : "fill";
      const script = `
import sys
import gi
gi.require_version("Atspi", "2.0")
from gi.repository import Atspi

Atspi.init()
pid, action, path, operation = int(sys.argv[1]), sys.argv[2], sys.argv[3], sys.argv[4]
desktop = Atspi.get_desktop(0)

def descendants(node):
    yield node
    for index in range(node.get_child_count()):
        child = node.get_child_at_index(index)
        if child is not None:
            yield from descendants(child)

def ready(node):
    states = node.get_state_set()
    bounds = Atspi.Component.get_extents(node, Atspi.CoordType.SCREEN)
    return all(states.contains(state) for state in (Atspi.StateType.SHOWING, Atspi.StateType.SENSITIVE)) and bounds.width > 0 and bounds.height > 0

for index in range(desktop.get_child_count()):
    app = desktop.get_child_at_index(index)
    if app.get_process_id() != pid:
        continue
    dialogs = [app.get_child_at_index(child) for child in range(app.get_child_count())]
    dialogs = [node for node in dialogs if node.get_role() == Atspi.Role.FILE_CHOOSER and node.get_name() in (("Save", "Save File", "Save As") if operation == "Save" else ("Open File",))]
    assert len(dialogs) <= 1, "The owned process exposes one accessible file picker"
    if not dialogs:
        break
    controls = list(descendants(dialogs[0]))
    if action == "fill":
        fields = [node for node in controls if node.get_role() == Atspi.Role.TEXT and node.get_editable_text_iface() is not None and ready(node)]
        if not fields:
            break
        assert len(fields) == 1, "The native chooser has one visible editable location entry"
        field = fields[0]
        assert field.get_editable_text_iface() is not None, "The native location entry is editable"
        assert Atspi.EditableText.set_text_contents(field, path), "The native location entry accepts the path"
        assert Atspi.Text.get_text(field, 0, -1) == path, "The native location entry retains the selected path"
    else:
        name = "Cancel" if action == "ready" else operation
        buttons = [node for node in controls if node.get_role() == Atspi.Role.PUSH_BUTTON and node.get_name() == name and ready(node)]
        if not buttons:
            break
        assert len(buttons) == 1, "The native picker exposes one ready action"
        if action == "ready" and operation == "Open":
            locations = [(node, index) for node in controls if node.get_action_iface() is not None for index in range(Atspi.Action.get_n_actions(node)) if Atspi.Action.get_action_name(node, index) == "show_location"]
            assert len(locations) == 1, "The owned chooser exposes its location action"
            assert Atspi.Action.do_action(*locations[0]), "The native location entry is shown"
        if action == "open":
            assert Atspi.Action.do_action(buttons[0], 0), "The Native File Action Is Activated Once"
    print(action)
    break
`;
      try {
        const result = await executeFile("/usr/bin/python3", ["-c", script, pid, action, resolve(file), operation], {
          windowsHide: true,
          timeout: 120000,
        });
        if (result.stdout.trim() !== action) return false;
        await appendFile(join(evidence, "native-picker.log"), `picker window=${ids[0]} action=${action} file=${resolve(file)}\n`);
        if (action === "ready") {
          picker.window = ids[0];
        } else if (action === "fill") {
          picker.entered = true;
        } else {
          picker.submitted = true;
        }
      } catch (failure) {
        picker.failure = failure;
        throw failure;
      }
      return false;
    }
    return false;
  }
  const script = `
      $ErrorActionPreference = 'Stop'
      Add-Type -AssemblyName UIAutomationClient
      Add-Type -AssemblyName UIAutomationTypes
      $root = [System.Windows.Automation.AutomationElement]::RootElement
      $providers = Add-Type -AssemblyName UIAutomationClientsideProviders -PassThru
      [System.Windows.Automation.ClientSettings]::RegisterClientSideProviderAssembly($providers[0].Assembly.GetName())
      $scope = [System.Windows.Automation.TreeScope]::Descendants
      $condition = [System.Windows.Automation.Condition]::TrueCondition
      Add-Type -Namespace DokimePicker -Name Window -MemberDefinition '[DllImport("user32.dll")] public static extern bool IsWindow(System.IntPtr handle); [DllImport("user32.dll", SetLastError = true)] public static extern bool PostMessage(System.IntPtr handle, uint message, System.IntPtr wide, System.IntPtr wide2);'
      $filenameId = if ($env:DOKIME_PICKER_OPERATION -eq 'Save') { '1001' } else { '1148' }
      [Console]::Error.WriteLine(('picker shell={0} file={1} window={2}' -f $env:DOKIME_PICKER_SHELL, $env:DOKIME_PICKER_FILE, $env:DOKIME_PICKER_WINDOW))
      $driver = Get-Process -Id ([int]$env:DOKIME_PICKER_DRIVER)
      $ownerKey = $driver.Id.ToString() + '/' + $driver.StartTime.ToFileTimeUtc().ToString()
      if (!$owners.ContainsKey($ownerKey)) {
        $children = @(Get-CimInstance Win32_Process -Filter ('ParentProcessId=' + $driver.Id) | ForEach-Object { $_.ProcessId })
        $found = @(Get-Process -Id $children -ErrorAction SilentlyContinue | Where-Object { $_.Path -eq $env:DOKIME_PICKER_SHELL } | ForEach-Object {
          @{Id=$_.Id; Started=$_.StartTime.ToFileTimeUtc()}
        })
        if ($found.Count) { $owners[$ownerKey] = $found }
      }
      foreach ($entry in $owners[$ownerKey]) {
        $process = Get-Process -Id $entry.Id -ErrorAction SilentlyContinue
        if (!$process -or $process.Path -ne $env:DOKIME_PICKER_SHELL -or $process.StartTime.ToFileTimeUtc() -ne $entry.Started) { continue }
        [Console]::Error.WriteLine(('picker process={0}' -f $process.Id))
        $pidCondition = New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::ProcessIdProperty, $process.Id)
        $handle = if ($env:DOKIME_PICKER_WINDOW) { $env:DOKIME_PICKER_WINDOW } else { $env:DOKIME_PICKER_ENTERED }
        if ($handle -and ![DokimePicker.Window]::IsWindow([IntPtr][long]$handle)) { Write-Output 'closed'; exit 0 }
        try {
          $windows = if ($handle) { @([System.Windows.Automation.AutomationElement]::FromHandle([IntPtr][long]$handle)) } else { $root.FindAll([System.Windows.Automation.TreeScope]::Children, $pidCondition) }
        } catch {
          if ($handle -and ![DokimePicker.Window]::IsWindow([IntPtr][long]$handle)) { Write-Output 'closed'; exit 0 }
          throw
        }
        foreach ($window in $windows) {
          if (!$window -or $window.Current.ProcessId -ne $process.Id -or $window.Current.ClassName -ne "#32770") { continue }
          [Console]::Error.WriteLine(('picker handle={0} title={1} offscreen={2}' -f $window.Current.NativeWindowHandle, $window.Current.Name, $window.Current.IsOffscreen))
          if ($env:DOKIME_PICKER_WINDOW) {
            if ($window.Current.NativeWindowHandle.ToString() -eq $env:DOKIME_PICKER_WINDOW) {
              Write-Output 'open'
              exit 0
            }
            continue
          }
          if ($env:DOKIME_PICKER_ENTERED -and $window.Current.NativeWindowHandle.ToString() -ne $env:DOKIME_PICKER_ENTERED) { continue }
          $fieldId = New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::AutomationIdProperty, $filenameId)
          $fieldType = New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::ControlTypeProperty, [System.Windows.Automation.ControlType]::Edit)
          $fieldCondition = New-Object System.Windows.Automation.AndCondition($fieldId, $fieldType)
          if ($env:DOKIME_PICKER_OPERATION -eq 'Save') {
            $fieldName = New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::NameProperty, 'File name:')
            $fieldCondition = New-Object System.Windows.Automation.AndCondition($fieldCondition, $fieldName)
          }
          $field = $window.FindFirst($scope, $fieldCondition)
          $buttonId = New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::AutomationIdProperty, '1')
          $buttonClass = New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::ClassNameProperty, 'Button')
          $buttonCondition = New-Object System.Windows.Automation.AndCondition($buttonId, $buttonClass)
          $button = $window.FindFirst($scope, $buttonCondition)
          if (!$field -or !$button) {
            foreach ($candidate in $window.FindAll($scope, [System.Windows.Automation.Condition]::TrueCondition)) {
              if ($candidate.Current.AutomationId -in @($filenameId, '1')) {
                [Console]::Error.WriteLine(('picker control id={0} type={1} class={2} name={3}' -f $candidate.Current.AutomationId, $candidate.Current.ControlType.ProgrammaticName, $candidate.Current.ClassName, $candidate.Current.Name))
              }
            }
          }
          if ($field -and $button -and $button.Current.IsEnabled) {
            $selectedWindow = $window.Current.NativeWindowHandle
            $value = $field.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern)
            # Invoke Only A Filename That Survived The Dialog's Own Initialization Since An Earlier Observation.
            if (!$env:DOKIME_PICKER_ENTERED -or ![String]::Equals($value.Current.Value, $env:DOKIME_PICKER_FILE, [StringComparison]::Ordinal)) {
              if ($env:DOKIME_PICKER_OPERATION -eq 'Save') {
                # Replace The Selection In One Native Edit, Including Its EN_CHANGE Notification.
                if (-not ("DokimePicker.Input" -as [type])) { Add-Type -Namespace DokimePicker -Name Input -MemberDefinition '[DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern System.IntPtr SendMessage(System.IntPtr window, uint message, System.IntPtr wide, System.IntPtr wide2); [DllImport("user32.dll", EntryPoint = "SendMessageW", ExactSpelling = true, CharSet = CharSet.Unicode)] public static extern System.IntPtr ReplaceSelection(System.IntPtr window, uint message, System.IntPtr wide, string text);' }
                $edit = [System.IntPtr]$field.Current.NativeWindowHandle
                if ($edit -eq [System.IntPtr]::Zero) { throw 'The Native Save Filename Box Has No Window Handle' }
                [void][DokimePicker.Input]::SendMessage($edit, 0x00B1, [System.IntPtr]0, [System.IntPtr](-1))
                [void][DokimePicker.Input]::ReplaceSelection($edit, 0x00C2, [System.IntPtr]1, $env:DOKIME_PICKER_FILE)
              } else {
                $value.SetValue($env:DOKIME_PICKER_FILE)
              }
              [Console]::Error.WriteLine(('picker entered={0}' -f $value.Current.Value))
              if (![String]::Equals($value.Current.Value, $env:DOKIME_PICKER_FILE, [StringComparison]::Ordinal)) {
                throw 'The Native Picker Filename Differs From The Requested File After Entry'
              }
              Write-Output ('entered:' + $selectedWindow)
              exit 0
            }
            # A Native Button May Be Projected As A Pane Without InvokePattern.
            $invoke = $null
            if ($button.TryGetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern, [ref]$invoke)) {
              [Console]::Error.WriteLine('picker action=invoke')
              $invoke.Invoke()
            } else {
              $buttonHandle = [System.IntPtr]$button.Current.NativeWindowHandle
              if ($buttonHandle -eq [System.IntPtr]::Zero -or ![DokimePicker.Window]::PostMessage($buttonHandle, 0x00F5, [System.IntPtr]::Zero, [System.IntPtr]::Zero)) {
                throw 'The native picker action could not be posted'
              }
              [Console]::Error.WriteLine('picker action=button-message')
            }
            Write-Output ('selected:' + $selectedWindow)
            exit 0
          }
        }
      }
      if ($env:DOKIME_PICKER_WINDOW) { Write-Output 'closed' }
      elseif ($env:DOKIME_PICKER_ENTERED) { Write-Output 'vanished' }`;
  /* The dialog resolves what is typed, so it receives the long form of a short 8.3 temporary path. */
  const target = join(await realpath(dirname(resolve(file))), basename(file));
  const result = await windowsPicker(script, {
    DOKIME_PICKER_SHELL: resolve(shellBinary),
    DOKIME_PICKER_DRIVER: String(owner.pid),
    DOKIME_PICKER_FILE: target,
    DOKIME_PICKER_OPERATION: operation,
    DOKIME_PICKER_ENTERED: picker.entered ? String(picker.entered) : "",
    DOKIME_PICKER_WINDOW: picker.window === null ? "" : String(picker.window),
  });
  await appendFile(join(evidence, "native-picker.log"), `${result.stderr}\n${result.stdout}\n`);
  const outcome = result.stdout.trim();
  assert.notEqual(outcome, "vanished", "The native dialog that received the filename closed before its selection");
  if (outcome.startsWith("entered:")) {
    picker.entered = Number(outcome.slice("entered:".length));
    assert.ok(Number.isSafeInteger(picker.entered), "The entered native dialog has a window handle");
  }
  if (outcome.startsWith("selected:")) {
    picker.window = Number(outcome.slice("selected:".length));
    assert.ok(Number.isSafeInteger(picker.window), "The selected native dialog has a window handle");
  }
  return outcome === "closed";
}

async function distributionIdentityJourney() {
  await closeSession();
  const directory = join(work, "distributions");
  const shared = join(directory, "user");
  const suffix = windows ? ".exe" : "";
  const identities = ["com.stokker.dokime", "com.example.testhub"];
  const roots = [];
  for (const identifier of identities) {
    const installation = join(directory, identifier);
    await mkdir(installation, { recursive: true });
    const shell = join(installation, `dokime-gui${suffix}`);
    await copyFile(shellBinary, shell);
    if (identifier !== identities[0]) {
      await writeFile(join(installation, "distribution.json"), JSON.stringify({ schema: 1, identifier }));
    }
    session = await startSession(shared, undefined, evidence, { identifier, shell });
    await session.connected();
    const application = session.applicationRoot;
    roots.push(application);
    assert.ok(existsSync(join(application, "data")), "The owned backend uses this distribution's data directory");
    const logs = windows ? join(application, "logs") : join(shared, "shell", "state", identifier, "logs");
    assert.ok(
      (await readdir(logs)).some((name) => name.startsWith("dokime-gui.")),
      "Desktop logging uses this distribution's log directory",
    );
    const webview = join(application, windows ? "EBWebView" : "webview");
    assert.ok(existsSync(webview), "The native WebView uses this distribution's profile");
    await writeFile(join(application, "identity-proof"), identifier);
    await closeSession();
  }
  assert.notEqual(roots[0], roots[1]);
  for (let index = 0; index < identities.length; index += 1) {
    assert.equal(await readFile(join(roots[index], "identity-proof"), "utf8"), identities[index]);
    assert.ok(existsSync(join(roots[index], "data")), "Opening the other distribution preserves this one's data");
  }
  session = await startSession(work, backend.endpoint, evidence);
  await session.connected();
}

async function organizationJourney() {
  const root = join(work, "shell", "application", "organization");
  const archive = join(work, "organization.zip");
  const manifest = `[app]
label = "Stokker Technologies"
title_template = "{app}"
[app.logos.transparent]
on_light = "logos/mark.svg"
[updates]
mode = "disabled"
source = "public"
[updates.sources.public]
kind = "direct"
manifest = "https://127.0.0.1:1/missing.json"
[updates.sources.private]
kind = "gitlab"
connection = "lab"
project = "lab/dokime"
[updates.connections.lab]
host = "https://127.0.0.1:1"
`;
  await executeFile(
    "python",
    [
      "-c",
      `import sys, zipfile
with zipfile.ZipFile(sys.argv[1], "w") as archive:
    archive.writestr("organization.toml", sys.argv[2])
    archive.writestr("logos/mark.svg", '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20"><circle cx="10" cy="10" r="8" fill="currentColor"/></svg>')
`,
      archive,
      manifest,
    ],
    { windowsHide: true, timeout: 120000 },
  );
  const choose = async () => {
    await session.goto("/settings/organization");
    await session.openPicker(await session.named("button", "Choose File…"));
    const picker = { window: null };
    await until("native organization file picker", () => chooseNativeFile(picker, archive), 120000);
    await until("pending organization pack", async () => hasText(await session.text(), "Applies on next start."));
  };
  const restart = async () => {
    await closeSession();
    session = await startSession(work, backend.endpoint, evidence);
    await sizeViewport(session, densities.desktop);
    await session.connected();
    await session.goto("/settings/organization");
  };
  await choose();
  assert.equal(
    await session.execute("return window.__DOKIME_ORGANIZATION_PACK__.pack.label;"),
    "Dokime",
    "Staging does not change this boot's appearance",
  );
  await restart();
  await until("native organization activation", async () => hasText(await session.text(), "Active since this start."));
  assert.ok(hasText(await session.text(), "Stokker Technologies"), "The mounted shell applies the imported label");
  assert.equal(
    await readFile(join(root, "active", "organization.toml"), "utf8"),
    manifest,
    "The resolved application root owns the active pack",
  );
  assert.equal(existsSync(join(root, "pending")), false, "Activation promotes the pending directory");
  await session.screenshot("organization-active");

  await session.goto("/settings/about");
  await session.clickNamed("button", "Check For Updates");
  const selection = (label) => session.execute(`return document.querySelector('select[aria-label="${label}"]')?.value;`);
  await until("disabled organization update policy", async () => (await selection("Update Checks")) === "disabled");
  const checkDisabled = () =>
    session.execute(
      `return [...document.querySelectorAll('[role="dialog"] button')].find(button => button.textContent.trim() === 'Check For Updates')?.disabled;`,
    );
  assert.equal(await checkDisabled(), true);
  const select = async (label, value) => {
    await session.execute(
      `const select = document.querySelector('select[aria-label="${label}"]'); select.value = ${JSON.stringify(value)}; select.dispatchEvent(new Event('change', {bubbles: true}));`,
    );
    await until(
      `saved ${label}`,
      async () =>
        (await selection(label)) === value &&
        (await session.execute(`return !document.querySelector('select[aria-label="${label}"]').disabled;`)),
    );
  };
  await select("Release Source", "private");
  await until("selected public GitLab source", async () => (await selection("Release Source")) === "private");
  assert.equal(await checkDisabled(), true);
  await select("Release Source", "public");
  await select("Update Checks", "manual");
  await session.execute(
    `const button = [...document.querySelectorAll('[role="dialog"] button')].find(button => button.textContent.trim() === 'Check For Updates'); button.click();`,
  );
  await until("update check failure notification", async () =>
    hasText(await toastText(session), "The update manifest could not be checked"),
  );
  assert.equal(
    await session.execute(
      `return [...document.querySelectorAll('[role="dialog"] button')].find(button => button.textContent.trim() === 'Install Update')?.disabled;`,
    ),
    true,
  );
  await select("Update Checks", "disabled");
  await select("Release Source", "private");
  await session.screenshot("organization-update-policy");
  await session.key("Escape");
  await session.waitForDom("update policy dialog closed", `return !document.querySelector('[role=dialog][data-state=open]');`);
  await choose();
  await writeFile(join(root, "pending", "organization.toml"), "invalid [");
  await restart();
  await session.goto("/settings/about");
  await session.clickNamed("button", "Check For Updates");
  await until("saved release source survives restart", async () => (await selection("Release Source")) === "private");
  assert.equal(await selection("Update Checks"), "disabled");
  await session.key("Escape");
  await session.waitForDom("update policy dialog closed", `return !document.querySelector('[role=dialog][data-state=open]');`);
  await session.goto("/settings/organization");
  assert.ok(hasText(await session.text(), "Stokker Technologies"), "An invalid pending pack retains the active appearance");
  assert.equal(await readFile(join(root, "active", "organization.toml"), "utf8"), manifest);
  assert.equal(
    await readFile(join(root, "pending", "organization.toml"), "utf8"),
    "invalid [",
    "Rejected input remains available for diagnosis",
  );
  await session.goto("/settings/diagnostics");
  await until("visible organization activation error", async () => /pending organization pack/i.test(await session.text()));
  await session.screenshot("organization-invalid-pending");
  await session.goto("/settings/organization");
  await session.clickNamed("button", "Remove");
  await until("organization removal queued", async () => hasText(await session.text(), "Bundled organization applies on next start."));
  await restart();
  assert.equal(await session.execute("return window.__DOKIME_ORGANIZATION_PACK__.pack.label;"), "Dokime");
  assert.ok(hasText(await session.text(), "Bundled Organization"), "Removal restores the bundled appearance on restart");
}

async function writeJunit() {
  if (!options.junit) return;
  const escapeXml = (value) =>
    [...redact(value)]
      .filter((character) => {
        const code = character.charCodeAt(0);
        return code >= 32 || code === 9 || code === 10 || code === 13;
      })
      .join("")
      .replace(/[&<>"']/g, (character) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&apos;" })[character]);
  const cases = [...results];
  for (const failure of failures) {
    const existing = cases.find((result) => result.name === failure.name);
    if (existing) existing.failure = failure.reason;
    else cases.push({ name: failure.name, seconds: 0, failure: failure.reason });
  }
  const failed = cases.filter((result) => result.failure);
  const requested = cases.filter((result) =>
    selectedJourneys.size ? selectedJourneys.has(result.name) : !workflow || workflow.journeys.has(result.name),
  );
  const skipped =
    failed.length === 0 &&
    (requested.length === 0 ||
      (selectedJourneys.size && requested.length !== selectedJourneys.size) ||
      (!selectedJourneys.size && workflow && requested.length !== workflow.journeys.size) ||
      requested.some((result) => result.skipped));
  const name = selectedJourneys.size
    ? `Selected Desktop Journeys: ${[...selectedJourneys].join(", ")}`
    : (workflow?.title ?? "Desktop Backend Walkthrough");
  const seconds = (performance.now() - walkthroughStarted) / 1000;
  const detail = cases
    .map((result) => `${result.name}: ${result.failure ?? (result.skipped ? "Skipped" : "Passed")} (${result.seconds}s)`)
    .join("\n");
  const outcome = failed.length
    ? `<failure message="${escapeXml(`${failed.length} scene(s) failed`)}">${escapeXml(detail)}</failure>`
    : skipped
      ? "<skipped/>"
      : "";
  const story = `<testcase classname="dokime.native.${process.platform}" name="${escapeXml(name)}" time="${seconds}">${outcome}<system-out>${escapeXml(detail)}</system-out></testcase>`;
  await mkdir(dirname(resolve(options.junit)), { recursive: true });
  await writeFile(
    options.junit,
    `<?xml version="1.0" encoding="UTF-8"?>\n<testsuites><testsuite name="Native Desktop" tests="1" failures="${failed.length ? 1 : 0}" errors="0" skipped="${skipped ? 1 : 0}" time="${seconds}">${story}</testsuite></testsuites>\n`,
  );
}

let secretBus;
let secretKeyring;

async function startSecretService() {
  if (windows) return;
  const home = join(work, "secret-service");
  await mkdir(home, { recursive: true, mode: 0o700 });
  process.env.XDG_RUNTIME_DIR = home;
  delete process.env.AT_SPI_BUS_ADDRESS;
  secretBus = spawn("dbus-daemon", ["--session", "--nofork", "--print-address=1"], {
    detached: !windows,
    windowsHide: true,
    stdio: ["ignore", "pipe", "inherit"],
  });
  let address = "";
  let startupError;
  secretBus.on("error", (error) => {
    startupError = error;
  });
  secretBus.stdout.on("data", (data) => {
    address += data.toString();
  });
  await until("isolated secret-service bus", () => {
    if (startupError) throw startupError;
    if (secretBus.exitCode !== null) throw new Error("The private D-Bus exited before startup");
    return address.includes("\n");
  });
  process.env.DBUS_SESSION_BUS_ADDRESS = address.trim();
  const accessibility = await executeFile(
    "dbus-send",
    ["--session", "--print-reply", "--dest=org.a11y.Bus", "/org/a11y/bus", "org.a11y.Bus.GetAddress"],
    { windowsHide: true, timeout: 120000 },
  );
  const accessibilityAddress = accessibility.stdout.match(/\bstring "([^"]+)"/);
  assert.ok(accessibilityAddress, "The private session owns an accessibility bus");
  process.env.AT_SPI_BUS_ADDRESS = accessibilityAddress[1];
  // Activated Registry Processes Must Use This Bus Too, Before Consulting The Shared Display.
  await executeFile("dbus-update-activation-environment", ["AT_SPI_BUS_ADDRESS"], {
    env: { ...process.env, DBUS_SESSION_BUS_ADDRESS: process.env.AT_SPI_BUS_ADDRESS },
    windowsHide: true,
    timeout: 120000,
  });
  await executeFile(
    "dbus-send",
    [
      `--bus=${process.env.AT_SPI_BUS_ADDRESS}`,
      "--print-reply",
      "--dest=org.freedesktop.DBus",
      "/org/freedesktop/DBus",
      "org.freedesktop.DBus.StartServiceByName",
      "string:org.a11y.atspi.Registry",
      "uint32:0",
    ],
    { windowsHide: true, timeout: 120000 },
  );
  secretKeyring = spawn("gnome-keyring-daemon", ["--foreground", "--components=secrets", "--unlock", "--control-directory", home], {
    env: { ...process.env, HOME: home, XDG_DATA_HOME: home, XDG_RUNTIME_DIR: home },
    detached: !windows,
    windowsHide: true,
    stdio: ["pipe", "ignore", "inherit"],
  });
  secretKeyring.on("error", (error) => {
    startupError = error;
  });
  secretKeyring.stdin.end("native-fixture-keyring-password");
  await until("isolated unlocked OS keyring", async () => {
    if (startupError) throw startupError;
    if (secretKeyring.exitCode !== null) throw new Error("The private keyring exited before startup");
    const result = await executeFile(
      "dbus-send",
      [
        "--session",
        "--print-reply",
        "--dest=org.freedesktop.DBus",
        "/org/freedesktop/DBus",
        "org.freedesktop.DBus.NameHasOwner",
        "string:org.freedesktop.secrets",
      ],
      { windowsHide: true, timeout: 120000 },
    );
    return result.stdout.includes("boolean true");
  });
}

async function main() {
  if (!windows && process.platform !== "linux") throw new Error("Native acceptance requires Windows WebView2 or Linux WebKitGTK.");
  if (releaseValidation && (selectedJourneys.size || workflow))
    throw new Error("Installed release validation requires every maintained journey.");
  if (!windows && !process.env.DISPLAY && !process.env.WAYLAND_DISPLAY) {
    throw new Error("Driving the native WebView needs a running display session.");
  }
  // Retain Native Evidence In The Lane Output; JUnit May Use Disposable Scratch.
  await mkdir(reportsRoot, { recursive: true });
  evidence = await mkdtemp(join(reportsRoot, "native-desktop-e2e-"));
  work = await mkdtemp(join(tmpdir(), "dokime-native-desktop-"));
  process.stdout.write(`Evidence: ${evidence}\n`);
  if (releaseValidation && !windows) throw new Error("Installed release validation currently requires Windows.");
  if (process.env.DOKIME_E2E_DEV_URL)
    throw new Error("Packaged desktop validation requires embedded assets, without a dev-origin override.");
  if (releaseValidation) {
    const installed = await journey("installation", installWindowsCandidate);
    if (!installed.ok) throw new Error("Windows installer acceptance failed");
  }
  for (const [label, path] of [
    ["the backend", backendBinary],
    ["the desktop shell", shellBinary],
    ["the CLI", cliBinary],
    ["the OBC charger plugin bundle", pluginBundle],
  ]) {
    if (!existsSync(path)) throw new Error(`Missing ${label} at ${path}. Use the same candidate's prepared artifacts.`);
  }

  const preparedChrome = join(workspace, "target/tooling/desktop/chrome-linux64/chrome");
  const chrome =
    process.env.DOKIME_E2E_CHROME ??
    (windows
      ? join(process.env["ProgramFiles(x86)"] ?? process.env.ProgramFiles, "Microsoft/Edge/Application/msedge.exe")
      : existsSync(preparedChrome)
        ? preparedChrome
        : "google-chrome");
  process.env.DOKIME_E2E_CHROME = chrome;
  reportBrowser = await ReportBrowser.start(work, chrome);
  await startSecretService();
  const owned = await journey("owned-startup", ownedStartupJourney, (scene) => scene.reports);
  if (!owned.ok) throw new Error("Native ownership acceptance failed");
  timingScene = "attached-setup";
  await closeSession();
  backend = await startBackend(work, evidence, undefined, "backend", false);
  session = await startSession(work, backend.endpoint, evidence);
  await sizeViewport(session, densities.desktop);

  await journey("startup", () => startupJourney(session, notes));
  const installed = await journey("plugin-install", pluginInstallJourney);
  if (!installed.ok) throw new Error("Native plugin installation failed");
  await installSaveTap(session);

  /* The Library is where cases come from, so it runs whatever the home said:
     a wrong count on Cycles is not a reason to certify nothing behind it. */
  const library = await journey("library", () => libraryJourney(session));
  const cycleId = library.value?.id;
  await journey("active-cycle", () => activeCycleJourney(session, library.value?.title ?? null));

  if (cycleId) {
    await journey(
      "cycle",
      () => cycleJourney(session, cycleId),
      (scene) => scene.reports,
    );
    await journey("touch", () => touchJourney(session, cycleId));
    await journey(
      "execution-stop",
      () => executionStopJourney(session, cycleId),
      (scene) => scene.reports,
    );
    await journey(
      "containers",
      () => containersJourney(session, cycleId),
      (scene) => scene.reports,
    );
    await journey("cycles-home", () => cyclesHomeJourney(session, library.value.title));
    await journey("history", () => historyJourney(session, cycleId, library.value.title, library.value.measuringExecutionId));
  } else if (!library.skipped) {
    for (const name of ["cycle", "touch", "execution-stop", "containers", "cycles-home", "history"]) {
      failures.push({
        kind: "failure",
        name,
        reason: "skipped: the library journey produced no cycle",
      });
    }
  }

  await journey("idle", idleJourney);
  await journey("settings", () => settingsJourney(session, notes));
  await journey("distribution-identity", distributionIdentityJourney);
  await journey("organization", organizationJourney);
  await journey(
    "remote-access",
    () => remoteAccessJourney([...pendingReports]),
    (scene) => scene.reports,
  );

  /* The ownership contract the redesign did not change: a shell attached to a
     backend it does not own must leave that backend running. */
  await journey("attachment", async () => {
    const activeCycleId = await createEmptyContainer(session, "Surviving Server Evidence");
    await cyclePhase(session, "Active");
    assert.ok(await listening(backend.endpoint), "the attached backend must still be listening");
    await session.goto("/cycles");
    const status = await session.executeAsync(`
      const done = arguments[arguments.length - 1];
      globalThis.__TAURI_INTERNALS__.invoke('desktop_session_status').then(
        (value) => done({value}),
        (error) => done({error: String(error)}),
      );`);
    assert.equal(status?.value?.ownership, "attached", "the session must report itself attached");
    assert.equal(
      status?.value?.connection?.endpoint?.replace(/\/$/, ""),
      backend.endpoint,
      "the session must name the endpoint it was attached to",
    );
    await closeSession();
    assert.ok(await listening(backend.endpoint), "closing an attachment preserves the server");
    session = await startSession(work, backend.endpoint, evidence);
    await session.connected();
    await session.goto(`/cycles/${activeCycleId}?view=cycle`);
    await cyclePhase(session, "Active");
    await until("reconnected cycle admission ready", () => session.named("button", "Seal Cycle"));
    assert.ok(await session.exists("button", "Seal Cycle"), "reconnecting preserves Active admission");
  });
  await journey(
    "restart",
    () => restartJourney([...pendingReports]),
    (scene) => scene.reports,
  );
  if (!selectedJourneys.size) {
    assert.deepEqual([...pendingReports], [], "Every materialized execution has an explicit report disposition");
  }
}

/* Mechanical Native Fixture Shared By Independent Desktop Stories. */
export { chooseNativeFile, startBackend, startSession, stopBackend, terminate, until, unusedPort };

export async function openNativeReport(path) {
  const reopened = await fetch(reportBrowser.endpoint, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ path }),
  });
  assert.ok(reopened.ok, "The owned report browser reopens saved HTML");
  await reportBrowser.closeTabs();
}

export async function prepareNativeFixture(root, output) {
  work = root;
  evidence = output;
  const prepared = join(workspace, "target/tooling/desktop/chrome-linux64/chrome");
  const chrome =
    process.env.DOKIME_E2E_CHROME ??
    (windows
      ? join(process.env["ProgramFiles(x86)"] ?? process.env.ProgramFiles, "Microsoft/Edge/Application/msedge.exe")
      : existsSync(prepared)
        ? prepared
        : "google-chrome");
  reportBrowser = await ReportBrowser.start(work, chrome);
  await startSecretService();
  return async () => {
    await reportBrowser.close();
    await stopNativePicker();
    await terminate(driverProcess);
    for (const child of [secretKeyring, secretBus]) await terminate(child);
    await writeFile(join(evidence, "timings.json"), JSON.stringify(timings, null, 2));
  };
}

/* === Entry Point === */

async function runDesktopWalkthrough() {
  try {
    await main();
  } catch (failure) {
    failures.push({ kind: "failure", name: "harness", reason: redact(failure.message) });
    process.stdout.write(`\nHarness failure: ${redact(failure.message)}\n`);
  } finally {
    timingScene = "cleanup";
    for (const [name, cleanup] of [
      ["shell shutdown", closeSession],
      [
        "report browser cleanup",
        async () => {
          await reportBrowser?.close();
          reportBrowserClosed = true;
        },
      ],
      ["native picker cleanup", stopNativePicker],
      ["backend shutdown", stopBackend],
      [
        "desktop logs",
        async () => {
          if (!work) return;
          const directory = join(work, "shell", "application", "logs");
          if (!existsSync(directory)) return;
          for (const file of await readdir(directory)) {
            if (!/^dokime-gui\..*\.log$/.test(file)) continue;
            await writeFile(join(evidence, `desktop-${file}`), redact(await readFile(join(directory, file), "utf8")));
          }
        },
      ],
    ]) {
      try {
        await cleanup();
      } catch (error) {
        failures.push({ kind: "failure", name, reason: error.message });
      }
    }
    if (driverProcess)
      await terminate(driverProcess).catch((error) => failures.push({ kind: "failure", name: "shell cleanup", reason: error.message }));
    if (backend) {
      await terminate(backend.child).catch((error) => failures.push({ kind: "failure", name: "backend cleanup", reason: error.message }));
      await backend.log.close().catch((error) => failures.push({ kind: "failure", name: "backend log cleanup", reason: error.message }));
    }
    for (const child of [secretKeyring, secretBus]) {
      if (!child) continue;
      try {
        await terminate(child);
        await until("secret service stopped", () => child.exitCode !== null || child.signalCode !== null, 120000);
      } catch (error) {
        failures.push({ kind: "failure", name: "secret service cleanup", reason: error.message });
      }
    }
    if (installerAttempted) {
      try {
        const uninstallers = (await readdir(installation)).filter((name) => /^uninstall.*\.exe$/i.test(name));
        assert.equal(uninstallers.length, 1, "the candidate installs its native uninstaller");
        await executeFile(join(installation, uninstallers[0]), ["/S", `_?=${installation}`], {
          windowsHide: true,
          windowsVerbatimArguments: true,
          timeout: 120000,
        });
      } catch (error) {
        failures.push({ kind: "failure", name: "installer cleanup", reason: error.message });
      }
    }
    if (work)
      await rm(join(work, "secrets"), { force: true, recursive: true }).catch(() =>
        failures.push({ kind: "failure", name: "secret cleanup", reason: "Private temporary secret removal failed" }),
      );
    if (work && reportBrowserClosed && !failures.length)
      await rm(work, { force: true, recursive: true }).catch((error) =>
        failures.push({ kind: "failure", name: "workspace cleanup", reason: error.message }),
      );
    if (evidence) {
      await writeFile(join(evidence, "timings.json"), JSON.stringify(timings, null, 2));
      const totals = {};
      for (const entry of timings) totals[entry.category] = (totals[entry.category] ?? 0) + entry.milliseconds;
      process.stdout.write(`\nTiming Categories (Nested): ${JSON.stringify(totals)}\n`);
    }
    await writeJunit();
  }

  for (const note of notes) process.stdout.write(`\nnote: ${note}\n`);

  if (failures.length) {
    /* A defect is the product refusing what the redesign promised; a failure is
     this gate not getting through. Both are non-zero, and they are not the
     same report. */
    for (const kind of ["defect", "failure"]) {
      const group = failures.filter((entry) => entry.kind === kind);
      if (!group.length) continue;
      process.stdout.write(`\n${group.length} ${kind === "defect" ? "product defect(s)" : "journey failure(s)"}:\n`);
      for (const entry of group) process.stdout.write(`  - ${entry.name}: ${redact(entry.reason)}\n`);
    }
    process.stdout.write(`\nEvidence: ${evidence}\n`);
    process.exitCode = 1;
  } else {
    process.stdout.write(`\nNative WebView: ${passedJourneys.join(", ")} passed.\n`);
    process.stdout.write(`Evidence: ${evidence}\n`);
    if (!process.env.DOKIME_E2E_KEEP && !options.junit) await rm(evidence, { force: true, recursive: true });
  }
}

if (resolve(process.argv[1]) === fileURLToPath(import.meta.url)) await runDesktopWalkthrough();
