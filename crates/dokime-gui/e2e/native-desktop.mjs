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
// Journeys, in order, each asserting on DOM text and roles and screenshotting
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
//   f'. plugin-install  native bundle picker, digest consent, restart activation and catalog
//   f'. organization  native pack import, restart activation and invalid-pending retention
//   f'. remote-access  TLS/key setup, saved secrets, restart, rotation and lab modes
//   g. report       Download Report, then the saved file rendered in headless Chrome
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
//     taps inline document saves at the Tauri IPC boundary. Report saves instead
//     use the real native verified pull into its private cache, then copy that
//     file into `<evidence>/downloads` in place of choosing a picker destination.
//     Every domain RPC still
//     crosses the real transport, and so does `desktop_save_diagnostics_bundle`,
//     which needs no picker: the throwaway home is given an xdg-user-dirs
//     Downloads folder and the shell writes the archive there itself.

/* === Imports === */

// Standard Library Imports

import assert from "node:assert/strict";

import { execFile, spawn } from "node:child_process";
import { createHash } from "node:crypto";
import { once } from "node:events";

import { createReadStream, existsSync } from "node:fs";

import { appendFile, copyFile, mkdir, mkdtemp, open, readdir, readFile, realpath, rm, writeFile } from "node:fs/promises";

import http from "node:http";

import net from "node:net";
import { tmpdir } from "node:os";

import { basename, dirname, join, resolve } from "node:path";

import { setTimeout as delay } from "node:timers/promises";

import { fileURLToPath } from "node:url";

import { parseArgs, promisify } from "node:util";

// Third Party Imports

// Local Imports
import { ReportBrowser } from "./report-browser.ts";

/* === Re-Exports === */

/* === Constants === */

const workspace = resolve(dirname(fileURLToPath(import.meta.url)), "../../..");

const { values: options } = parseArgs({
  options: { junit: { type: "string" }, release: { type: "boolean", default: false }, profile: { type: "string" } },
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
const pluginBundle = process.env.DOKIME_E2E_PLUGIN ?? join(built, "plugins/obc-plugin.dokime");

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
const WEBDRIVER_KEYS = { Enter: "\uE007", Escape: "\uE00C", Tab: "\uE004" };

/** The journeys everything after them depends on, whatever the filter says. */
const REQUIRED_JOURNEYS = new Set(["startup", "library"]);

/** Opt out of real key presses when the driver cannot honour the Actions API. */
const syntheticKeys = process.env.DOKIME_E2E_SYNTHETIC_KEYS === "1";

/**
 * Which journeys to run, as a comma-separated list; every one by default.
 *
 * `startup` and `library` always run whatever the filter says: they establish
 * the connection and materialize the cycle every journey after them addresses.
 */
const selectedJourneys = new Set(
  (process.env.DOKIME_E2E_JOURNEYS ?? "")
    .split(",")
    .map((name) => name.trim())
    .filter(Boolean),
);

/**
 * Write Public Documents Into Evidence And Access Keys Into Private Temporary Storage.
 *
 * The payload crosses in slices. A written report is megabytes of base64, and
 * one `execute` carrying all of it does not come back -- the document was
 * already taken off the page's list by then, so it was lost rather than late,
 * and the gate reported a report that had in fact been written.
 */
const DOCUMENT_SLICE_CHARS = 512 * 1024;

const releaseValidation = options.release;
const windows = process.platform === "win32";
let reportBrowser;
let reportBrowserClosed = false;
const results = [];
const failures = [];
const passedJourneys = [];

const notes = [];
const secrets = new Set();
let savedKeyCount = 0;

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
    socket.setTimeout(30000, () => {
      socket.destroy();
      done(false);
    });
  });
}

async function until(description, check, timeout = 30000, interval = 150) {
  const deadline = Date.now() + timeout;
  let last;
  while (Date.now() < deadline) {
    try {
      const value = await check();
      if (value) return value;
    } catch (failure) {
      last = failure;
    }
    await delay(interval);
  }
  throw new Error(`Timed out: ${description}${last ? ` (last error: ${last.message})` : ""}`);
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
  for (let offset = 0; offset <= bounds.width; offset += 4) {
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
  const plot = document.querySelector('[data-plot-id="obc.charger.startup.sequence.voltage_distribution"]');
  if (!plot) return null;
  const bins = [...plot.querySelectorAll("button[aria-label]")].map((button) => button.getAttribute("aria-label"));
  return bins.length ? { bins, text: plot.innerText } : null;
}

function assertHistogram(presentation) {
  assert.ok(presentation, "The voltage distribution must render bins");
  assert.match(presentation.text, /Count/);
  assert.match(presentation.text, /[1-9]\d* samples/);
  assert.ok(
    presentation.bins.every((label) => /V.* to .*V.*: \d+$/.test(label)),
    "Histogram bins must name voltage ranges and counts",
  );
  assert.ok(
    presentation.bins.some((label) => /: [1-9]\d*$/.test(label)),
    "At least one histogram bin must contain samples",
  );
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

async function startBackend(work, evidence, endpoint, label = "backend", launchPlugin = true) {
  const port = endpoint ? Number(new URL(endpoint).port) : await unusedPort();
  const log = await open(join(evidence, `${label}.log`), "w");
  const child = spawn(
    backendBinary,
    [
      "server",
      "--shutdown-stdin",
      ...(launchPlugin ? ["--plugin", pluginBundle] : []),
      "--port",
      String(port),
      "--output-root",
      join(work, "data"),
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
  await until(
    "backend listening",
    () => {
      if (child.startupError) throw child.startupError;
      if (child.exitCode !== null) throw new Error(`Native backend exited during startup: ${child.exitCode}`);
      return listening(backend.endpoint);
    },
    180000,
  );
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
async function startSession(work, endpoint, evidence) {
  const home = join(work, "shell");
  const downloads = join(evidence, "downloads");
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
        DOKIME_APPLICATION_ROOT: join(home, "application"),
        ...(windows
          ? {
              WEBVIEW2_USER_DATA_FOLDER: join(home, "webview"),
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
      stdio: ["ignore", "ignore", "inherit"],
    },
  );
  const webdriver = driverProcess;
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
              binary: resolve(shellBinary),
              args: endpoint ? [`--endpoint=${endpoint}`] : [],
            },
          }
        : {
            browserName: "wry",
            "webkitgtk:browserOptions": {
              args: endpoint ? [`--endpoint=${endpoint}`] : [],
              binary: resolve(shellBinary),
            },
          },
    },
  });

  const session = {
    closed: false,
    downloads,
    id: created.sessionId,
    lastBody: "",
    process: webdriver,
  };
  let queue = Promise.resolve();
  session.request = (method, path, body) => {
    const queued = queue.then(() => request(method, `/session/${session.id}${path}`, body));
    queue = queued.catch(() => undefined);
    return queued;
  };
  session.execute = (script, args = []) => session.request("POST", "/execute/sync", { args, script });
  session.executeAsync = (script, args = []) => session.request("POST", "/execute/async", { args, script });
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
       element.scrollIntoView({block: 'center', inline: 'center', behavior: 'instant'});
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
    return session.find("xpath", `//${tag}[${text}${enabled ? " and not(@disabled)" : ""}]`);
  };
  session.clickNamed = async (role, name, options) =>
    session.click(await until(`${role} ${name}`, () => session.named(role, name, options)));
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
    if (!value.length) {
      /* `clear` empties the element without the input event a controlled
         React field listens for, so an emptied box keeps its old value. */
      return session.execute(
        `const input = arguments[0];
         const setter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value').set;
         setter.call(input, '');
         input.dispatchEvent(new Event('input', {bubbles: true}));
         input.dispatchEvent(new Event('change', {bubbles: true}));`,
        [element],
      );
    }
    await session.request("POST", `/element/${key}/clear`, {});
    await session.request("POST", `/element/${key}/value`, { text: value, value: [...value] });
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
  /**
   * Go to a route by loading it.
   *
   * A load of the dev origin drops the native session and the shell attaches
   * again, which costs a few seconds per route and is the price of driving the
   * real deep links rather than poking the router's history from outside it.
   */
  session.goto = async (path) => {
    await session.load(path.startsWith("file:") ? path : session.origin + path);
    if (path.startsWith("file:")) return;
    await session.connected();
    await until(`route ${path}`, async () => (await session.location()).pathname === new URL(session.origin + path).pathname, 30000);
    await installSaveTap(session);
  };
  session.close = async () => {
    if (session.closed) return;
    try {
      await session.execute(`
        globalThis.__TAURI_INTERNALS__.invoke('plugin:window|close', {label: 'main'});
      `);
      await until(
        "native window completes orderly close",
        async () => {
          try {
            return (await session.request("GET", "/window/handles")).length === 0;
          } catch (error) {
            if (/invalid session id|disconnected|connection refused/i.test(error.message)) return true;
            throw error;
          }
        },
        40000,
      );
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
    await delay(350);
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
  await until("Tauri IPC bridge", () => session.execute("return typeof globalThis.__TAURI_INTERNALS__?.invoke === 'function'"), 60000);
  const state = await session.execute(`
    globalThis.__dokimeSavedDocuments = globalThis.__dokimeSavedDocuments ?? [];
    globalThis.__dokimeInvokes = globalThis.__dokimeInvokes ?? [];
    globalThis.__dokimeStreams = globalThis.__dokimeStreams ?? [];
    globalThis.__dokimeStreamReads = globalThis.__dokimeStreamReads ?? 0;
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
        if (command === 'desktop_plugin_stage_file') {
          const payload = String(init && init.body);
          entry.keys = Object.keys(JSON.parse(payload));
          entry.size = payload.length;
        }
        ledger.push(entry);
        let reportSave = null;
        if (command === 'desktop_artifact_deliver') {
          const payload = JSON.parse(String(init && init.body));
          if (payload.request.kind === 'html_report' && payload.request.action === 'save') {
            reportSave = { filename: payload.request.filename, kind: payload.request.kind };
            payload.request.action = 'open';
            init = { ...init, body: JSON.stringify(payload) };
          }
        }
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
            entry.state = 'settled';
            entry.ms = Date.now() - entry.at;
            if (reportSave && value.headers.get('Tauri-Response') === 'ok') {
              return value.clone().json().then((delivered) => {
                if (!delivered?.path) throw new Error('native report pull returned no verified path');
                saved.push({ ...reportSave, path: delivered.path });
                return value;
              });
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

async function drainSavedDocuments(session) {
  const documents = await session.execute(`
    return (globalThis.__dokimeSavedDocuments ?? []).map((document) => ({
      filename: document.filename,
      kind: document.kind,
      path: document.path ?? null,
      length: document.bytesBase64?.length ?? 0,
    }));
  `);
  const written = [];
  for (const [index, document] of documents.entries()) {
    assert.equal(basename(document.filename), document.filename, "saved documents have a plain filename");
    if (document.path) {
      const target = join(session.downloads, document.filename);
      await copyFile(document.path, target);
      written.push({ kind: document.kind, path: target });
      continue;
    }
    let base64 = "";
    for (let offset = 0; offset < document.length; offset += DOCUMENT_SLICE_CHARS) {
      base64 += await session.execute(
        "return globalThis.__dokimeSavedDocuments[arguments[0]].bytesBase64.slice(arguments[1], arguments[2]);",
        [index, offset, offset + DOCUMENT_SLICE_CHARS],
      );
    }
    if (base64.length !== document.length) {
      throw new Error(`The tapped ${document.filename} transferred ${base64.length} of ${document.length} characters`);
    }
    const bytes = Buffer.from(base64, "base64");
    let target = join(session.downloads, document.filename);
    if (document.kind === "remote_key") {
      assert.ok(bytes.toString("utf8").trim().length > 0, "the saved access key is nonempty");
      secrets.add(base64);
      secrets.add(bytes.toString("utf8").trim());
      const directory = join(work, "secrets");
      await mkdir(directory, { recursive: true, mode: 0o700 });
      target = join(directory, `remote-${++savedKeyCount}.key`);
    }
    await writeFile(target, bytes, { mode: 0o600 });
    written.push({ kind: document.kind, path: target });
  }
  /* Cleared only once every document is on disk, so a transfer that fails
     leaves the document on the page to try again rather than losing it. */
  if (documents.length) await session.execute("globalThis.__dokimeSavedDocuments.splice(0, arguments[0]);", [documents.length]);
  return written;
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
        const row = rows.get(entry.command) ?? {
          command: entry.command,
          calls: 0,
          pending: 0,
          slowestMs: 0,
        };
        row.calls += 1;
        if (entry.state === 'pending') {
          row.pending += 1;
          row.waitingMs = Math.max(row.waitingMs ?? 0, now - entry.at);
        }
        if (entry.state === 'rejected') row.lastError = String(entry.error).slice(0, 300);
        row.slowestMs = Math.max(row.slowestMs, entry.ms ?? 0);
        rows.set(entry.command, row);
      }
      return {
        recorded: entries.length,
        streams: globalThis.__dokimeStreams ?? [],
        peakStreams: globalThis.__dokimeStreamPeak ?? 0,
        network: globalThis.__dokimeNetwork ?? 0,
        commands: [...rows.values()].sort((left, right) => right.calls - left.calls),
      };
    `)
    .catch((failure) => ({ error: String(failure) }));
}

/* Shared Interactions */

/** Set the density through Appearance, which is the only honest switch. */
async function setDensity(session, variant) {
  await session.goto("/settings/appearance");
  await until("appearance section", async () => hasText(await session.text(), "Density"), 60000);
  await session.clickNamed("radio", variant.density === "touch" ? "Touch" : "Comfortable");
  await until(
    `density ${variant.density}`,
    async () => (await session.execute("return document.documentElement.dataset.density")) === variant.density,
    30000,
  );
  await sizeViewport(session, variant);
  await delay(400);
}

/** Let the transient notifications clear before the next pointer-like click. */
async function settle(session) {
  await until(
    "transient notifications clear",
    () =>
      session.execute(`
        return [...document.querySelectorAll('[data-sonner-toast]')].every((toast) => {
          if (toast.dataset.visible === 'false') return true;
          const style = getComputedStyle(toast);
          return style.pointerEvents === 'none' || !toast.getClientRects().length;
        });
      `),
    30000,
  ).catch(() => undefined);
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
      await delay(800);
    }
    /* The queue is gone the moment its last prompt is answered, so what it
     showed is read here, before answering, and asserted once the run is over. */
    const sighting = await session.execute(`
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
    await delay(400);
    const submitted = await session.execute(`
    const panel = document.querySelector('li[data-prompt-state="open"] [aria-labelledby="active-prompt-title"]');
    if (!panel) return false;
    const submit = [...panel.querySelectorAll('button')].find((button) =>
      /^(Yes|Submit|Confirm|Accept|Continue)$/.test(button.innerText.trim()),
    );
    if (!submit) return false;
    submit.click();
    return true;
  `);
    if (submitted) await delay(1500);
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
async function runUntil(session, description, check, timeout) {
  const deadline = Date.now() + timeout;
  /* A run that never starts looks exactly like a run that is slow, and the
     difference is in the toast: the workspace reports every command's own
     outcome there, including the backend's refusal text. Carrying the last
     announcement into the timeout is what turns five silent minutes into the
     reason. */
  let announcement = "";
  while (Date.now() < deadline) {
    if (await check()) return;
    announcement = (await toastText(session).catch(() => "")) || announcement;
    await answerPendingPrompts(session);
    await delay(1200);
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
    30000,
  );
  /* A row menu is a modal layer: the next row is unreachable until it is gone. */
  await closeAnyMenu(session);
  await session.clickNamed("button", target.trigger);
  await until("the row menu", () => session.find("css selector", "[role=menu]"), 30000);
  await session.clickNamed("menuitem", "Start Execution");
  await closeAnyMenu(session);
  return until(
    `the ${fragment} case to start`,
    async () => {
      const row = await session.execute(
        `return [...document.querySelectorAll('nav[aria-label=Navigation] li[data-level=case]')]
          .find((candidate) => candidate.dataset.executionId === arguments[0])?.innerText ?? '';`,
        [target.executionId],
      );
      return (
        Boolean(row) &&
        (row !== target.before ||
          /Running|Waiting|Finalizing|Passed|Failed/.test(row) ||
          /Execution Started/.test(await toastText(session)))
      );
    },
    30000,
  ).catch(() => false);
}

/** Press a menu trigger until its menu is up, and answer with the menu. */
async function openMenu(session, name, attempts = 2) {
  let last;
  for (let attempt = 0; attempt < attempts; attempt += 1) {
    await session.clickNamed("button", name);
    const menu = await until(`the ${name} menu`, () => session.find("css selector", "[role=menu]"), 30000).catch((failure) => {
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
  await until(
    "no open menu",
    async () => {
      const open = await session.execute("return document.querySelectorAll('[role=menu]').length;");
      if (!open) return true;
      await session.execute(`
        for (const menu of document.querySelectorAll('[role=menu]')) {
          for (const type of ['keydown', 'keyup']) {
            menu.dispatchEvent(new KeyboardEvent(type, {bubbles: true, key: 'Escape'}));
          }
        }
      `);
      return false;
    },
    30000,
  ).catch(() => undefined);
  await delay(300);
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
    await until("active cycle picker", () => session.find("css selector", "[role=dialog] [data-testid=open-cycles]"), 30000);
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
    until("dialogs closed", async () => (await session.execute("return document.querySelectorAll('[role=dialog]').length")) === 0, 30000);

  await session.goto("/library");
  await until("suite rows", async () => hasText(await session.text(), STARTUP_SUITE), 60000);
  assert.equal(
    await session.execute("return document.querySelector('select[aria-label=\"Destination Cycle\"]');"),
    null,
    "The Library carries no destination control",
  );

  /* New Cycle in the picker creates a cycle and makes it active at once. */
  await openPicker();
  await clickInDialog("New Cycle");
  const titleField = await until("new cycle dialog", () => session.find("css selector", "#new-cycle-title"), 30000);
  /* A title of its own, so no row or toast can match the library journey's cycle. */
  const title = `Active Cycle Journey ${Date.now()}`;
  await session.type(titleField, title);
  await session.clickNamed("button", "Create Cycle");
  await dialogsClosed();
  await until("the created cycle active", async () => hasText((await element()) ?? "", title), 60000);

  /* Choosing a row makes that cycle active; choosing the new one again restores it. */
  if (otherTitle) {
    await openPicker();
    await clickInDialog(otherTitle);
    await dialogsClosed();
    await until("the chosen cycle active", async () => hasText((await element()) ?? "", otherTitle), 30000);
  }
  await openPicker();
  await clickInDialog(title);
  await dialogsClosed();
  await until("the new cycle active again", async () => hasText((await element()) ?? "", title), 30000);
  assert.match((await element()) ?? "", /Active · Idle/, "The element states the cycle's own state");

  /* The template dialog names the active cycle with Change, and the case joins it. */
  await session.clickNamed("button", STARTUP_SUITE);
  await until("startup template", async () => hasText(await session.text(), STARTUP_TEMPLATE), 30000);
  await session.clickCss('[data-testid="suite-template-obc.charger.startup.sequence"] button');
  const inline = () =>
    session.execute("return document.querySelector('[role=dialog] [data-testid=template-active-cycle]')?.innerText ?? null;");
  const named = (expected) => until(`the dialog naming ${expected}`, async () => hasText((await inline()) ?? "", expected), 30000);
  await named(title);
  assert.ok(hasText((await inline()) ?? "", "Change"), "The template dialog must offer Change");

  /* Change opens the same picker over the dialog, and the choice applies in place. */
  for (const chosen of otherTitle ? [otherTitle, title] : [title]) {
    await clickInDialog("Change");
    await until("picker over the dialog", () => session.find("css selector", "[role=dialog] [data-testid=open-cycles]"), 30000);
    await clickInDialog(chosen);
    await until(
      "picker closed",
      async () => (await session.execute("return document.querySelectorAll('[data-testid=open-cycles]').length")) === 0,
      30000,
    );
    await named(chosen);
  }
  await clickInDialog("Next");
  await dialogsClosed();
  const added = await until("add announced", async () => {
    const text = await toastText(session);
    return /Added 1 case to /.test(text) ? text : null;
  });
  assert.ok(hasText(added, title), `The case must join the active cycle (got ${normalize(added)})`);
}

/**
 * Library: the only route by which cases enter a cycle.
 *
 * The cycle comes first, through New Cycle, and becomes the active cycle that
 * `appendTo` in the URL names. Then one suite expanded, one template
 * configured, the editor exercised on a number and on a list, the summary
 * read, and the cases materialized twice into that same cycle.
 */
async function libraryJourney(session) {
  await session.goto("/library");
  await until("suite rows", async () => hasText(await session.text(), PROTECTION_SUITE), 60000);
  assert.equal(
    await session.execute("return document.querySelector('select[aria-label=\"Destination Cycle\"]');"),
    null,
    "The active cycle is shell state, not a Library control",
  );

  /* New Cycle opens a dialog titled by the moment it opened. */
  await session.clickNamed("button", "New Cycle");
  await until("new cycle dialog", () => session.find("css selector", "#new-cycle-title"), 30000);
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
    60000,
  );
  await until(
    "new cycle dialog closed",
    async () => (await session.execute("return document.querySelectorAll('[role=dialog]').length")) === 0,
    30000,
  );
  await until("the created cycle active in the shell", async () =>
    hasText((await session.execute("return document.querySelector('[data-testid=status-active-cycle]')?.innerText ?? '';")) ?? "", title),
  );
  assert.ok(await session.exists("button", "Open Cycle"), "With an active cycle the page bar action must become Open Cycle");

  await session.clickNamed("button", PROTECTION_SUITE);
  await until("expanded suite", async () => hasText(await session.text(), "Configure"), 30000);
  assert.ok(hasText(await session.text(), "Template"), "An expanded suite must state its template count");

  /* The row itself opens the dialog; Configure is the same outcome named. */
  await session.clickNamed("button", "Configure");
  await until("template dialog", () => session.find("css selector", "[role=dialog]"), 30000);
  const step1 = await session.text();
  assert.ok(hasText(step1, "Parameters"), "Step 1 must be Parameters");
  assert.ok(hasText(step1, "Summary"), "The step indicator must name Summary");
  assert.ok(hasText(step1, "Values you change are remembered for this template."), "Step 1 must state that edits persist");
  await until("parameter grid", () => session.find("css selector", "[data-testid=parameter-editor-grid]"));
  assert.ok(await session.exists("button", "Reset All"), "The parameter step must carry Reset All");

  /* A number edit reveals that row's Reset affordance and its gilt rule. */
  const number = await session.find("css selector", `[data-testid="parameter-input-${NUMBER_PARAMETER}"]`);
  await session.type(number, "60");
  await delay(500);
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
  await until("added chip", async () => ((await chips()) ?? []).length === before.length + 1, 30000);
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
    30000,
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
  await delay(400);

  /* Step 2 reads the preview back: the deltas, the count, and the target. */
  await session.clickNamed("button", "Next");
  await until("summary step", async () => hasText(await session.text(), "Finish And Add"), 60000);
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
    60000,
  );
  assert.equal((await session.location()).searchParams.get("appendTo"), appendTo, "Adding cases must keep the active cycle");

  /* A second template joins the same cycle, so the run has a prompt in it. */
  await settle(session);
  await session.clickNamed("button", STARTUP_SUITE);
  await until("startup template", async () => hasText(await session.text(), STARTUP_TEMPLATE), 30000);
  await session.clickCss('[data-testid="suite-template-obc.charger.startup.sequence"] button');
  await until("second dialog", () => session.find("css selector", "[role=dialog]"), 30000);
  /* One case at its registered values has nothing to review: Next adds it at
     once and closes the dialog, without a summary step. */
  await session.clickNamed("button", "Next");
  await until(
    "second add finished",
    async () => (await session.execute("return document.querySelectorAll('[role=dialog]').length")) === 0,
    60000,
  );
  const added = await until("second add announced", async () => {
    const text = await toastText(session);
    return /Added 1 case to /.test(text) ? text : null;
  });
  assert.ok(hasText(added, title.trim()), `The second add must join the same active cycle (got ${normalize(added)})`);
  assert.equal((await session.location()).searchParams.get("appendTo"), appendTo, "Adding cases must keep the active cycle");

  await session.clickNamed("button", "Open Cycle");
  await until("cycle workspace", async () => (await session.location()).pathname === `/cycles/${appendTo}`, 30000);
  const measuringExecutionId = await until(
    "the materialized measuring execution identity",
    () =>
      session.execute(
        `return [...document.querySelectorAll('nav[aria-label=Navigation] li[data-level=case]')]
      .find((row) => row.innerText.includes(arguments[0]))?.dataset.executionId ?? null;`,
        [PROTECTION_TEMPLATE],
      ),
    30000,
  );
  return { id: appendTo, title: title.trim(), measuringExecutionId };
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
    60000,
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
  await until("the cycle page", async () => (await session.location()).searchParams.get("view") === "cycle", 30000);
  const header = await session.text();
  assert.ok(/\d+ of \d+ cases finished/.test(normalize(header)), "The header must state progress");
  assert.ok(hasText(header, "Active · Idle"), "A materialized cycle that runs nothing must render its Active · Idle state");
  assert.ok(await session.exists("button", "Cycle Actions"), "The header must carry the overflow menu");
  assert.ok(await session.exists("button", "Export Evidence"), "The cycle header must name its evidence export");
  assert.ok(await session.exists("button", "Download Report"), "The cycle header must name its report download");
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
    30000,
  );

  /* Open Report while the cycle is in flight: no seal yet, so the runtime
     renders one from the run so far and the shell hands it to the opener
     (`xdg-open` is a no-op here). It is never disabled and never refused. */
  const inFlight = await until(
    "the cycle header to show an in-flight lifecycle",
    async () => /\b(Running|Paused|Stopping)\b/.test(normalize(await session.text())),
    30000,
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
      60000,
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
    60000,
  );
  const selection = await until(
    "execution selected in the URL",
    async () => {
      const location = await session.location();
      const executionId = location.searchParams.get("executionId");
      return executionId ? { executionId, caseId: location.searchParams.get("caseId") } : null;
    },
    30000,
  );
  assert.equal(selection.caseId, null, "Execution rows must select solely by executionId");
  const evidence = await until(
    "execution evidence",
    async () => {
      const text = await session.text();
      return /Execution \d+/i.test(normalize(text)) ? text : null;
    },
    60000,
  );
  assert.match(normalize(evidence), /Execution \d+/i);
  await checkParameterGeometry(session);
  assert.ok(await session.exists("button", "Export Evidence"), "The execution header must name its evidence export");
  assert.ok(await session.exists("button", "Download Report"), "The execution header must name its report download");
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
    60000,
  );
  assert.notEqual(rerunId, selection.executionId, "Re-Run must create a distinct execution");
  assert.match(await sidebarText(session), /Re-Run Of #\d+/, "The new row must name its previous execution");
  await until("Start on the fresh attempt", () => session.exists("button", "Start"), 30000);
  await session.clickNamed("button", "Start");
  await until("the new attempt admission", async () => /Execution Started/.test(await toastText(session)), 30000);
  /* Plot Settings Persist In The Native Frozen Document. */
  const plotId = await until(
    "the plot identity to configure",
    () =>
      session.execute(`
    return document.querySelector('button[aria-label^="Plot settings for "]')?.closest('[data-plot-id]')?.dataset.plotId ?? null;
  `),
    30000,
  );
  const settingsSelector = `[data-plot-id=${JSON.stringify(plotId)}] button[aria-label^='Plot settings for ']`;
  await session.clickCss(settingsSelector);
  const settingsText = await until(
    "plot settings dialog",
    () =>
      session.execute(`
        const dialog = document.querySelector('[role=dialog][data-state=open]');
        return dialog?.innerText;
      `),
    30000,
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
      30000,
    );
  } catch (error) {
    throw new Error(`${error.message}; the open dialog reads: ${normalize(openDialog)}`);
  }
  await session.goto("/cycles");
  await session.goto(`/cycles/${cycleId}?executionId=${rerunId}`);
  await until(
    "the same plot frame after remounting",
    () =>
      session.execute(
        `
    const frame = [...document.querySelectorAll('[data-live-mounted="true"][data-plot-id]')]
      .find((node) => node.dataset.plotId === arguments[0]);
    return frame && !frame.innerText.includes('Loading Plot') && Boolean(frame.querySelector('button[aria-label^="Plot settings for "]'));
  `,
        [plotId],
      ),
    30000,
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
    30000,
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
  assert.match(rerunOutcome, /\bPassed\b/, "Closing plot settings must leave the prompting re-run able to pass");
  const promptHovers = await session.executeAsync(`(${promptHoverLabels.toString()})().then(arguments[arguments.length - 1]);`);
  assertPromptHovers(promptHovers);
  assertHistogram(await until("live voltage histogram", () => session.execute(`return (${histogramPresentation.toString()})();`)));
  const statisticsHeaders = await until(
    "the native trace statistics headers",
    () =>
      session.execute(`
        const tables = [...document.querySelectorAll('table[aria-label="Trace statistics"]')];
        if (!tables.length) return null;
        return tables.flatMap(table => [...table.querySelectorAll('th')].map(header => {
          const leaves = [...header.querySelectorAll('span')].filter(node => !node.children.length && node.textContent.trim());
          return {
            text: header.innerText,
            clipped: leaves.some(node => {
              const range = document.createRange();
              range.selectNodeContents(node);
              const text = range.getBoundingClientRect();
              let parent = node;
              while (parent && parent !== table.parentElement) {
                const style = getComputedStyle(parent);
                const bounds = parent.getBoundingClientRect();
                if (['hidden', 'clip'].includes(style.overflowX) && (text.left < bounds.left - 1 || text.right > bounds.right + 1)) return true;
                if (['hidden', 'clip'].includes(style.overflowY) && (text.top < bounds.top - 1 || text.bottom > bounds.bottom + 1)) return true;
                parent = parent.parentElement;
              }
              return false;
            }),
          };
        }));
      `),
    30000,
  );
  assert.ok(
    statisticsHeaders.some((header) => /Y Cursor/.test(header.text)),
    "Native statistics include cursor headings",
  );
  assert.ok(
    statisticsHeaders.some((header) => /\([^)]\S*\)/.test(header.text)),
    "Native statistics include physical units",
  );
  assert.deepEqual(
    statisticsHeaders.filter((header) => header.clipped),
    [],
    "Statistics titles and units remain readable without clipping",
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
    30000,
  );

  /* Focus is a URL flag, so it survives a reload and can be linked. The router
     serializes the flag as `focus=true`; `focusSearchSchema` also accepts the
     `focus=1` a person would type, so both spellings are the flag. */
  await session.clickNamed("button", "Focus");
  await until("focus flag set", async () => ["1", "true"].includes((await session.location()).searchParams.get("focus")), 30000);
  assert.ok(await session.exists("button", "Leave Focus"), "Focus view must say how to leave it");
  await session.clickNamed("button", "Leave Focus");
  await until("focus flag cleared", async () => (await session.location()).searchParams.get("focus") !== "1", 30000);

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
  await delay(600);
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
  await session.clickNamed("button", "Jump to Latest");
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
  await delay(600);
  const levels = await session.execute("return [...document.querySelectorAll('[role=option]')].map((item) => item.innerText.trim());");
  for (const level of ["Trace", "Info", "Error"]) {
    assert.ok(levels.includes(level), `The level selector must offer ${level} (got ${levels})`);
  }
  await session.key("Escape");
  await delay(400);
  await session.key("Escape");
  await until(
    "console closed by Escape",
    async () => (await session.execute("return document.querySelectorAll('section[aria-label=Console]').length")) === 0,
    30000,
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
  await until("the cycle page", () => session.exists("button", "Cycle Actions"), 30000);
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
  return selection;
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
    30000,
  );
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
  await delay(3000);
  assert.equal(
    (await session.location()).searchParams.get("executionId"),
    unstarted,
    "An unstarted execution the operator selected must stay selected while another runs",
  );
  assert.ok(hasText(await session.text(), "This case has not run yet"), "An unstarted execution shows its planned parameters");
  await checkParameterGeometry(session);
  /* An attempt that has not started is cancelled, not stopped: Stop belongs
     to a running attempt alone. */
  await until("Cancel Execution on the unstarted attempt", () => session.exists("button", "Cancel Execution"), 30000);
  assert.ok(!(await session.exists("button", "Stop Execution")), "An unstarted execution must not offer Stop");
  assert.ok(!hasText(await session.text(), "No step in progress"), "An unstarted execution states that it waits for Start");
  await session.execute("document.querySelector('nav[aria-label=Navigation] li[data-level=overview] button[data-tree-select]').click();");
  await until("the cycle selection", async () => (await session.location()).searchParams.get("view") === "cycle", 30000);
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
  await until("Stop Execution", () => session.exists("button", "Stop Execution"), 30000);

  // === Stop Only That Execution === //

  await session.clickNamed("button", "Stop Execution");
  await until("the stop acknowledgement", async () => /Execution Stopped And Evidence Saved/.test(await toastText(session)), 30000);
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
  await until("Cancel Execution", () => session.exists("button", "Cancel Execution"), 30000);
  await session.clickNamed("button", "Cancel Execution");
  await until("the cancel acknowledgement", async () => /Execution Cancelled/.test(await toastText(session)), 30000);
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
  return copyId;
}

/** Completed executions seal independently; the operator closes their container. */
async function containersJourney(session, cycleId) {
  await session.goto(`/cycles/${cycleId}?view=cycle`);
  await cyclePhase(session, "Active");
  await exportAvailability(session, false);
  const select = await until("a finished execution row", () =>
    session.find("css selector", "nav[aria-label=Navigation] li[data-level=case] button[data-tree-select]"),
  );
  await session.click(select);
  const executionId = await until("a finished execution selected", async () => (await session.location()).searchParams.get("executionId"));
  await exportAvailability(session, true);

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
  await exportAvailability(session, true);
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
}

/** The cycle's own state reads "Active · Idle" and the like while Active, then "Sealing" and "Sealed". */
async function cyclePhase(session, phase) {
  await until(`cycle phase ${phase}`, async () => hasText(await session.text(), phase === "Active" ? "Active ·" : phase));
}

async function exportAvailability(session, enabled) {
  const state = await until(`evidence export ${enabled ? "enabled" : "disabled"}`, () =>
    session.execute(
      `
    const button = [...document.querySelectorAll('button')].find((candidate) => candidate.innerText.trim() === 'Export Evidence');
    if (!button || button.disabled === arguments[0]) return null;
    return { disabled: button.disabled, title: button.title };
  `,
      [enabled],
    ),
  );
  assert.equal(state.disabled, !enabled);
  assert.equal(state.title, enabled ? "Download SQLite, Files And Saved Reports" : "Seal This Cycle Before Exporting");
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
    60000,
  );
  assert.ok(hasText(body, "Attention"), "The group must render its column headings");
  await checkStatusBounds(session);
  assert.ok(/\d+ \/ \d+/.test(normalize(body)), "A row must state executions over cases");
  assert.ok(/\d+[smhd] Ago/.test(normalize(body)), "A row must state when the cycle last moved");

  /* Nothing is archived yet, so the Archived view is one empty group, and
     leaving it brings the run's cycle back. */
  await session.clickNamed("button", "Archived");
  assert.ok(
    await until("archived view", async () => hasText(await session.text(), "No Archived Cycles")),
    "The Archived view must say when nothing is archived",
  );
  await session.clickNamed("button", "Archived");
  await until("default view", async () => !hasText(await session.text(), "No Archived Cycles"), 30000);

  const filter = await session.find("css selector", "input[aria-label='Filter Cycles']");
  await session.type(filter, "zzz-no-such-cycle");
  await until("filter excludes every row", async () => hasText(await session.text(), "No Cycles Match This Filter"), 30000);
  await session.type(filter, "");
  await until("filter cleared", async () => !hasText(await session.text(), "No Cycles Match This Filter"), 30000);
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
        /\\b[1-9]\\d* passed\\b/.test(row.innerText)
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
  await until("history detail", async () => (await session.location()).pathname === `/history/${cycleId}`, 30000);
  const checkpoint = await until(
    "the executed measuring checkpoint",
    () =>
      session.execute(
        `
    const identity = [...document.querySelectorAll('main p')].find((node) => node.textContent.trim() === arguments[0]);
    const entry = identity?.closest('section');
    if (!entry || !/\\b(passed|failed)\\b/i.test(entry.innerText)) return null;
    return [...entry.querySelectorAll('button')]
      .find((button) => button.innerText.trim() === 'Inspect checkpoint' && !button.disabled) ?? null;
  `,
        [executionId],
      ),
    60000,
  );
  await session.click(checkpoint);
  await until("captured execution", async () => hasText(await session.text(), "Recorded Steps"), 60000);
  const captured = await session.text();
  assert.ok(hasText(captured, "Recorded status"), "The checkpoint must state its recorded status");
  for (const view of ["Records", "Telemetry", "Files", "Context"]) {
    assert.ok(await session.exists("button", view), `The checkpoint must offer the ${view} view`);
  }
  await session.clickNamed("button", "Records");
  await until(
    "captured records",
    async () => {
      const section = await session.find("css selector", "section[aria-label='Captured records']").catch(() => null);
      if (!section) return false;
      return !hasText(await session.textOf(section), "No execution records at this checkpoint.");
    },
    60000,
  );
  await session.clickNamed("button", "Context");
  await until("captured context", async () => hasText(await session.text(), "Notes At Capture"));

  await session.clickNamed("button", "History Library");
  await until("back to history", async () => (await session.location()).pathname === "/history", 30000);
}

/** Remote Access Uses Mounted Controls And Real TLS CLI Calls. */
async function remoteAccessJourney() {
  const input = (label) =>
    until(label, () => session.find("xpath", `//input[@id=//label[normalize-space(.)=${xpathLiteral(label)}]/@for]`));
  /* Every control commits to the host at once and stays disabled until the host has answered. */
  const settled = () =>
    until("remote settings settled", () =>
      session.execute("return [...document.querySelectorAll('input[id], button[role=switch]')].every((control) => !control.disabled);"),
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
  /* A field applies when the operator presses Enter, as its description says;
     leaving one field for the next would otherwise commit it mid-typing and
     disable every control until the host answers. */
  const commitField = async (label, value) => {
    await settled();
    await session.type(await input(label), value);
    await session.key("Enter");
    await settled();
  };
  /* A notice still on screen from an earlier change cannot stand in for this one. */
  const notice = async (text, action) => {
    await until(`no earlier ${text}`, async () => !hasText(await toastText(session), text), 30000);
    await action();
    await until(text, async () => hasText(await toastText(session), text));
  };
  const endpoint = () =>
    until("remote listening address", async () => {
      const match = /Listening on (127\.0\.0\.1:\d+)/i.exec(await session.text());
      return match ? `https://${match[1]}` : null;
    });
  const connect = async (address, keyFile, certificate, accepted) => {
    const args = ["catalog", "list", "--endpoint", address, "--timeout-seconds", "30"];
    if (keyFile) args.push("--key-file", keyFile);
    if (certificate) args.push("--ca", certificate);
    const env = { ...process.env, DOKIME_APPLICATION_ROOT: join(work, "cli") };
    delete env.DOKIME_KEY;
    delete env.DOKIME_ENDPOINT;
    let result;
    try {
      result = await promisify(execFile)(cliBinary, args, { windowsHide: true, cwd: work, env, timeout: 30000 });
    } catch (error) {
      assert.ok(!accepted, "the saved remote secrets establish a real CLI connection");
      assert.equal(error.killed, false, "secret refusal returns without killing the client");
      assert.ok(Number.isInteger(error.code) && error.code !== 0, "secret refusal has a native failure exit code");
      return;
    }
    assert.ok(accepted, "superseded or missing secrets must be refused");
    assert.ok(result.stdout.trim().length > 0, "the connected CLI reads the real catalog");
    assert.ok(
      ![...secrets].some((secret) => result.stdout.includes(secret) || result.stderr.includes(secret)),
      "CLI output contains no key",
    );
  };
  const saveKey = async () => {
    await until("one-time key dialog", async () => hasText(await session.text(), "Save Your Access Key"));
    const value = await session.execute("return document.querySelector('[aria-label=\"Generated Access Key\"]').value;");
    assert.ok(typeof value === "string" && value.length > 0, "the one-time key is available");
    secrets.add(value);
    secrets.add(Buffer.from(`${value}\n`).toString("base64"));
    await sizeViewport(session, { width: 1080, height: 760 });
    const keyLayout = await session.execute(`
      const input = document.querySelector('[aria-label="Generated Access Key"]');
      const dialog = input.closest('[role=dialog]');
      const bounds = dialog.getBoundingClientRect();
      const controls = [...dialog.querySelectorAll('textarea, button')].map(node => {
        const rect = node.getBoundingClientRect();
        return rect.width > 0 && rect.height > 0 && rect.left >= bounds.left && rect.right <= bounds.right
          && rect.top >= bounds.top && rect.bottom <= bounds.bottom;
      });
      return {controls, fits: bounds.left >= 0 && bounds.right <= innerWidth && bounds.top >= 0 && bounds.bottom <= innerHeight,
        keyFits: input.scrollWidth <= input.clientWidth && input.scrollHeight <= input.clientHeight,
        overflow: dialog.scrollWidth > dialog.clientWidth};
    `);
    await writeFile(join(evidence, "access-key-layout.json"), JSON.stringify(keyLayout, null, 2));
    assert.ok(
      keyLayout.fits && keyLayout.keyFits && !keyLayout.overflow,
      "The access key and its dialog must fit the small native viewport",
    );
    assert.ok(keyLayout.controls.length >= 5 && keyLayout.controls.every(Boolean), "Every key action must remain inside the dialog");
    assert.equal(
      await session.execute("return document.hasFocus();"),
      true,
      "The native key dialog must own keyboard focus before copying",
    );
    await session.clickNamed("button", "Copy Key");
    await until("native clipboard write", () => session.exists("button", "Copied"));
    await session.clickNamed("button", "Save Key File…");
    await until("saved key acknowledgement", async () => hasText(await session.text(), "Key File Saved"));
    await session.clickNamed("button", "Save Public Certificate…");
    await until("saved public certificate acknowledgement", async () => hasText(await session.text(), "Certificate Saved"));
    const documents = await drainSavedDocuments(session);
    const key = documents.find((document) => document.kind === "remote_key")?.path;
    const certificate = documents.find((document) => document.kind === "public_certificate")?.path;
    assert.ok(key && certificate, "the GUI produces both secret files");
    assert.ok((await readFile(key, "utf8")).trim() === value, "the saved key matches the one-time value");
    assert.match(await readFile(certificate, "utf8"), /-----BEGIN CERTIFICATE-----/);
    await session.clickNamed("button", "I Have Saved The Key");
    await sizeViewport(session, densities.desktop);
    return { key, certificate };
  };

  // Enable TLS And Authentication Through The Native GUI.
  await session.goto("/settings/remote");
  await until("remote settings", async () => hasText(await session.text(), "Enable Remote Access"));
  const settingsLayout = await session.execute(`
    const card = title => [...document.querySelectorAll('main section')].find(section => section.querySelector('h2')?.textContent === title);
    const listener = card('Listener').getBoundingClientRect();
    const secrets = card('Secrets').getBoundingClientRect();
    const sessions = card('Recent Remote Sessions').getBoundingClientRect();
    return {columns: Math.abs(listener.top - secrets.top) < 2 && listener.right <= secrets.left,
      equal: Math.abs(listener.width - secrets.width) < 2,
      full: Math.abs(sessions.left - listener.left) < 2 && Math.abs(sessions.right - secrets.right) < 2};
  `);
  assert.ok(
    settingsLayout.columns && settingsLayout.equal && settingsLayout.full,
    "Remote settings must use two equal columns and a full-width sessions table",
  );
  await commitField("Listen Address", "127.0.0.1:0");
  await notice("Certificate Generated", () => commitField("Certificate DNS Names And IP Addresses", "127.0.0.1, localhost"));
  await toggle("TLS Encryption", true);
  await toggle("Require An Access Key", true);
  await session.clickNamed("button", "Generate Access Key");
  const original = await saveKey();
  await notice("Remote Access Enabled", () => toggle("Enable Remote Access", true));
  let address = await endpoint();
  await connect(address, original.key, original.certificate, true);
  await connect(address, null, original.certificate, false);

  // A Failed Bind Restores The Previous Listener And Leaves The Desktop Connected.
  const occupied = net.createServer();
  occupied.listen(0, "127.0.0.1");
  await once(occupied, "listening");
  try {
    await notice("Remote Access Change Failed", () => commitField("Listen Address", `127.0.0.1:${occupied.address().port}`));
    assert.equal(await endpoint(), address, "failed listener replacement retains the original bound address");
    await connect(address, original.key, original.certificate, true);
  } finally {
    await new Promise((done) => occupied.close(done));
  }

  // Restart Both Clients And Server To Prove Persisted Settings And Secrets.
  const nativeEndpoint = backend.endpoint;
  await closeSession();
  await stopBackend();
  backend = await startBackend(work, evidence, nativeEndpoint, "backend-remote-restart");
  session = await startSession(work, nativeEndpoint, evidence);
  await session.connected();
  await session.goto("/settings/remote");
  address = await endpoint();
  assert.ok(await session.exists("button", "Rotate Access Key…"), "only the key verifier is restored");
  assert.equal(await session.execute("return document.querySelector('[aria-label=\"Generated Access Key\"]') === null;"), true);
  const browserStorage = await session.execute("return JSON.stringify([Object.entries(localStorage), Object.entries(sessionStorage)]);");
  assert.ok(![...secrets].some((secret) => browserStorage.includes(secret)), "browser storage contains no access key");
  await connect(address, original.key, original.certificate, true);

  // Keep An Admitted Execution Waiting For Its Operator Throughout Rotation.
  const activeCycle = (await libraryJourney(session)).id;
  assert.ok(await startCaseFromItsRow(session, STARTUP_TEMPLATE), "the native GUI admits the rotation execution");
  await until("operator prompt before rotation", async () => hasText(await session.text(), STARTUP_PROMPT), 60000);
  await session.goto("/settings/remote");
  await session.clickNamed("button", "Rotate Access Key…");
  await session.clickNamed("button", "Rotate");
  const replacement = await saveKey();
  address = await endpoint();
  await connect(address, original.key, replacement.certificate, false);
  await connect(address, replacement.key, replacement.certificate, true);
  const previousCertificate = join(work, "secrets", "previous.pem");
  await copyFile(replacement.certificate, previousCertificate);
  await notice("Certificate Generated", () => session.clickNamed("button", "Regenerate Certificate"));
  address = await endpoint();
  await session.clickNamed("button", "Save Certificate…");
  await until("saved regenerated certificate acknowledgement", async () => hasText(await session.text(), "Certificate Saved"));
  const documents = await drainSavedDocuments(session);
  const certificate = documents.find((document) => document.kind === "public_certificate")?.path;
  assert.ok(certificate, "the regenerated public certificate is saved");
  await connect(address, replacement.key, previousCertificate, false);
  await connect(address, replacement.key, certificate, true);
  await session.goto(`/cycles/${activeCycle}?view=cycle`);
  await cyclePhase(session, "Active");
  /* The cycle page names the execution waiting on input; the prompt itself is answered there. */
  await until("the cycle names the execution its surviving prompt waits on", async () =>
    hasText(await session.text(), "Input needed on #"),
  );
  const running = await caseRowText(session, STARTUP_TEMPLATE);
  assert.match(running, /\b(Running|Waiting)\b/, "rotation leaves the admitted execution running");
  await session.execute(
    `const row = [...document.querySelectorAll('nav[aria-label=Navigation] li[data-level=case]')]
      .find((candidate) => candidate.innerText.includes(arguments[0]));
     row.querySelector('button[data-tree-select]').click();`,
    [STARTUP_TEMPLATE],
  );
  await until("the surviving prompt on its execution", async () => hasText(await session.text(), STARTUP_PROMPT), 30000);
  const decision = await session.execute(`
    const panel = document.querySelector('[aria-labelledby="active-prompt-title"]');
    return {actions: [...panel.querySelectorAll('button')].map(button => button.innerText.trim()),
      checkboxes: panel.querySelectorAll('[role=checkbox], input[type=checkbox]').length};
  `);
  assert.ok(decision.actions.includes("No") && decision.actions.includes("Yes"), "The inspection decision must offer No and Yes");
  assert.equal(decision.checkboxes, 0, "The decision belongs in the footer without a duplicate checkbox");
  await session.clickNamed("button", "Stop Execution");
  await until("rotation execution stopped", async () => hasText(await toastText(session), "Execution Stopped And Evidence Saved"));

  // Authentication And Encryption Are Independent Explicit Lab Choices.
  await session.goto("/settings/remote");
  await toggle("Require An Access Key", false);
  address = await endpoint();
  await connect(address, null, certificate, true);
  await toggle("Require An Access Key", true);
  await toggle("TLS Encryption", false);
  address = (await endpoint()).replace("https:", "http:");
  await connect(address, replacement.key, null, true);
  await connect(address, null, null, false);

  // Leave The Host In Its Verified TLS And Authentication Configuration.
  await toggle("TLS Encryption", true);
  const restoredAddress = await endpoint();
  await connect(restoredAddress, replacement.key, certificate, true);
  for (let index = 0; index < 21; index += 1) await connect(restoredAddress, replacement.key, certificate, true);
  await session.goto("/settings/remote");
  const sessionRows = () =>
    session.execute(`
    const table = document.querySelector('table[aria-label="Recent Remote Sessions"]');
    if (!table) return [];
    table.scrollIntoView({block: 'center'});
    return [...table.querySelectorAll('tbody tr[data-window-row]')].map(row => row.cells[0].innerText);
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
}

/** Settings: every section, and the rows the redesign promises in each. */
async function settingsJourney(session, notes) {
  const sections = [
    ["general", "General"],
    ["appearance", "Appearance"],
    ["identity", "Identity"],
    ["plugins", "Plugins"],
    ["secrets", "Secret Storage"],
    ["account-updates", "Account"],
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
      60000,
    );
    assert.ok(hasText(body, "Settings"), `The ${id} page bar must name Settings`);
    const rows = await session.execute("return document.querySelectorAll('main section, main dl > div').length");
    assert.ok(rows > 0, `The ${id} section must render rows`);
  }

  /* Appearance: the density radios autosave, with no Save button anywhere. */
  await session.goto("/settings/appearance");
  await until("appearance", async () => hasText(await session.text(), "Density"), 60000);
  assert.ok(!(await session.exists("button", "Save")), "Settings autosave; no section may carry a Save button");
  await session.clickNamed("radio", "Touch");
  await until(
    "touch density applied",
    async () => (await session.execute("return document.documentElement.dataset.density")) === "touch",
    30000,
  );
  await session.clickNamed("radio", "Comfortable");
  await until(
    "comfortable density applied",
    async () => (await session.execute("return document.documentElement.dataset.density")) === "comfortable",
    30000,
  );

  /* Storage sources: one row per configured source. */
  await session.goto("/settings/general");
  const general = await until(
    "storage sources",
    async () => {
      const text = await session.text();
      return hasText(text, "Storage Sources") ? text : null;
    },
    60000,
  );
  const storageRows = await session.execute(
    `const list = document.querySelector('[aria-label="Storage source priority"]');
     return list ? list.children.length : 0;`,
  );
  assert.ok(
    storageRows >= 1 || hasText(general, "No storage sources are configured."),
    "Storage Sources must render its rows or say there are none",
  );

  /* The attached desktop saves policy in the backend's execution root. */
  const interval = await until("backend policy input", () =>
    session.execute(
      `const label = [...document.querySelectorAll('label')].find(label => label.textContent.trim() === 'Compaction Interval (Hours)');
     return label?.control ?? null;`,
    ),
  );
  await session.type(interval, "2");
  /* The backend keeps the policy in its own state store: the save is proven by
     its restart notice and by the value it reads back after navigation. */
  await until("policy restart requirement", async () => hasText(await session.text(), "Restart the connected backend"), 30000);
  await session.goto("/settings/appearance");
  await session.goto("/settings/general");
  await until("saved backend policy reread", () =>
    session.execute(
      `const label = [...document.querySelectorAll('label')].find(label => label.textContent.trim() === 'Compaction Interval (Hours)');
     return label?.control?.value === '2';`,
    ),
  );

  /* Plugins: rows with a Settings button, and the plugin page behind it. */
  await session.goto("/settings/plugins");
  await until("the plugins section", async () => hasText(await session.text(), "Install A Plugin"), 60000);
  const rows = await session.execute(`return document.querySelectorAll('button[aria-label="Plugin Settings"]').length`);
  const configurable = await session.execute(
    `return [...document.querySelectorAll('button[aria-label="Plugin Settings"]')].filter(
       (button) => !button.disabled,
     ).length;`,
  );
  if (!rows) {
    /* Nothing is in the installation store: this backend was launched with
       `--plugin`, and the section has to say where its plugins came from
       rather than leaving the list unexplained. */
    assert.ok(
      hasText(await session.text(), "Loaded from the launch configuration; managed outside the installation store."),
      "With no installed plugin the Plugins section must still name the loaded ones",
    );
    notes.push(
      "settings: the gate launches the backend with --plugin, so the installation store is empty and the plugin row's Settings button is not on this path",
    );
  }
  if (configurable) {
    await session.clickNamed("button", "Plugin Settings");
    await until("plugin page", async () => (await session.location()).pathname.startsWith("/settings/plugins/"), 30000);
    assert.ok(hasText(await session.text(), "Plugins"), "The plugin page must carry its breadcrumb");
    await session.clickNamed("button", "Back");
    await until("back to plugins", async () => (await session.location()).pathname === "/settings/plugins", 30000);
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
      ["Name", name],
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
    await until("shared keyring entry saved", () => session.exists("button", `Edit ${name}`));
    assert.ok(!(await session.text()).includes(value), "stored secret values stay masked");
  }
  for (const name of ["e2e.settings.first", "e2e.settings.second"]) {
    await session.clickNamed("button", `Delete ${name}`);
    await session.clickNamed("button", "Confirm Delete");
    await until("shared keyring entry removed", async () => !(await session.exists("button", `Edit ${name}`)));
  }

  /* Diagnostics: read-only paths with Copy and Open, then the bundle itself. */
  await session.goto("/settings/diagnostics");
  const diagnostics = await until(
    "diagnostics",
    async () => {
      const text = await session.text();
      return hasText(text, "Storage Root") ? text : null;
    },
    60000,
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

/**
 * The frozen report: saved by the application's Download Report, then read back.
 *
 * The same component set renders it with `live={false}`, so the assertions are
 * about what a file can honour: identity, provenance, the outline and the
 * records -- and the absence of every control that would lie.
 */
async function reportJourney(session, cycleId) {
  await session.goto(`/cycles/${cycleId}?view=cycle`);
  await until("cycle workspace", () => session.exists("button", "Cycle Actions"), 60000);
  /* The report lists every planned member, run or not, as the live tree does. */
  const liveMembers = await session.execute("return document.querySelectorAll('nav[aria-label=Navigation] li[data-level=case]').length;");
  await installSaveTap(session);
  await session.clickNamed("button", "Download Report");
  /* The toast carries the backend's own words when the report fails, and a
     report that is never saved is otherwise five silent minutes. */
  let announcement = "";
  const written = await until(
    "report saved",
    async () => {
      announcement = (await toastText(session)) || announcement;
      const saved = await drainSavedDocuments(session);
      return saved.find((entry) => entry.path.endsWith(".html")) ?? null;
    },
    300000,
  ).catch((failure) => {
    throw new Error(`${failure.message}. The application said: ${announcement || "nothing at all"}`);
  });

  /* wry unwraps `http::Uri::from_str(webview.uri())` for every IPC message,
     and Tauri's own init scripts post one on each document load, so a report
     opened inside the shell's WebView aborts the process whenever the current
     URI is not an http URI (wry 0.55.1 webkitgtk/mod.rs:648). Operators open
     reports in a browser, so the gate reads the written file the same way:
     served over loopback and rendered by headless Chrome. */
  const html = await readFile(written.path, "utf8");
  // Select An Execution Through The Frozen Outline Before Inspecting Its Evidence.
  const selectExecution = `<script>
    const selection = setInterval(() => {
      const row = [...document.querySelectorAll('[data-level="case"]')].find(row => row.innerText.includes(${JSON.stringify(STARTUP_TEMPLATE)}));
      const button = row?.querySelector('button');
      if (button) { clearInterval(selection); button.click(); }
    }, 50);
    const markers = setInterval(async () => {
      if (!document.querySelector('[data-plot-id="obc.charger.startup.sequence.signals"] .u-over')) return;
      clearInterval(markers);
      const labels = await (${promptHoverLabels.toString()})();
      const result = document.createElement('pre');
      result.id = 'prompt-hover-proof';
      result.hidden = true;
      result.textContent = JSON.stringify(labels);
      document.body.append(result);
    }, 50);
    const geometry = setInterval(() => {
      const rows = (${parameterGeometry.toString()})();
      if (rows.length < 2) return;
      clearInterval(geometry);
      const result = document.createElement('pre');
      result.id = 'parameter-geometry-proof';
      result.hidden = true;
      result.textContent = JSON.stringify(rows);
      document.body.append(result);
    }, 50);
    const histogram = setInterval(() => {
      const presentation = (${histogramPresentation.toString()})();
      if (!presentation) return;
      clearInterval(histogram);
      const result = document.createElement('pre');
      result.id = 'histogram-proof';
      result.hidden = true;
      result.textContent = JSON.stringify(presentation);
      document.body.append(result);
    }, 50);
  </script>`;
  const diagnostic = `Voltage channel unavailable: ${"sensor/".repeat(70)}timeout`;
  const marker = '<script id="hub-report-data" type="application/json">';
  const payloadText = html.split(marker)[1].split("</script>")[0];
  const faultPayload = JSON.parse(payloadText);
  let injected = 0;
  for (const run of faultPayload.case_runs) {
    const plot = run.plots["obc.charger.startup.sequence.voltage_distribution"];
    if (!plot) continue;
    plot.analysis = { kind: "error", message: diagnostic };
    injected += 1;
  }
  assert.ok(injected > 0, "The fault fixture must replace a captured voltage distribution");
  const faultHtml = html.replace(payloadText, JSON.stringify(faultPayload));
  const server = http.createServer((request, response) => {
    response.writeHead(200, { "content-type": "text/html; charset=utf-8" });
    const page = request.url === "/plot-error.html" ? faultHtml : html;
    response.end(page.replace("</body>", `${selectExecution}</body>`));
  });
  await new Promise((ready) => server.listen(0, "127.0.0.1", ready));
  const served = `http://127.0.0.1:${server.address().port}/report.html`;
  let dom = "";
  let reportButtons = [];
  let chromeProcess;
  let socket;
  try {
    const chrome =
      process.env.DOKIME_E2E_CHROME ??
      (windows
        ? join(process.env["ProgramFiles(x86)"] ?? process.env.ProgramFiles, "Microsoft/Edge/Application/msedge.exe")
        : "google-chrome");
    const port = await unusedPort();
    chromeProcess = spawn(
      chrome,
      [
        "--headless=new",
        "--disable-gpu",
        "--no-sandbox",
        "--window-size=1600,1000",
        `--user-data-dir=${join(work, "chrome")}`,
        `--remote-debugging-port=${port}`,
        served,
      ],
      { stdio: "ignore", detached: !windows, windowsHide: true },
    );
    const page = await until(
      "frozen report browser",
      async () => {
        const response = await fetch(`http://127.0.0.1:${port}/json/list`);
        return (await response.json()).find((page) => page.type === "page");
      },
      30000,
    );
    socket = new WebSocket(page.webSocketDebuggerUrl);
    await new Promise((resolve, reject) => {
      socket.addEventListener("open", resolve, { once: true });
      socket.addEventListener("error", reject, { once: true });
    });
    let sequence = 0;
    const evaluate = (expression) =>
      new Promise((resolve, reject) => {
        const id = ++sequence;
        const timer = setTimeout(() => finish(new Error("Frozen report evaluation timed out")), 30000);
        const receive = (event) => {
          const response = JSON.parse(event.data);
          if (response.id !== id) return;
          if (response.error || response.result.exceptionDetails) finish(new Error(JSON.stringify(response)));
          else finish(null, response.result.result.value);
        };
        const finish = (error, value) => {
          clearTimeout(timer);
          socket.removeEventListener("message", receive);
          if (error) reject(error);
          else resolve(value);
        };
        socket.addEventListener("message", receive);
        socket.send(JSON.stringify({ id, method: "Runtime.evaluate", params: { expression, returnByValue: true } }));
      });
    await until(
      "frozen report mounted evidence",
      () => evaluate("['prompt-hover-proof', 'parameter-geometry-proof', 'histogram-proof'].every(id => document.getElementById(id))"),
      60000,
    ).catch(async (error) => {
      await writeFile(join(evidence, "failure-report-rendered.html"), await evaluate("document.documentElement.outerHTML"));
      throw error;
    });
    dom = await evaluate("document.documentElement.outerHTML");
    reportButtons = await evaluate(`
      [...document.querySelectorAll('button')].map(button => ({
        label: button.innerText.trim(),
        trace: Boolean(button.closest('table[aria-label="Trace statistics"]')),
      }))
    `);
    // A Controlled Analysis Failure Uses The Packaged Report's Actual Renderer And Styles.
    await evaluate(`location.assign(${JSON.stringify(served.replace("report.html", "plot-error.html"))}); true`);
    const errorLayout = await until("report plot diagnostic", () =>
      evaluate(`(() => {
      const overlay = document.querySelector('[data-plot-id="obc.charger.startup.sequence.voltage_distribution"] [role=alert]');
      if (!overlay) return null;
      overlay.scrollIntoView({block: 'center'});
      const bounds = overlay.getBoundingClientRect();
      const detail = overlay.lastElementChild;
      const text = detail.getBoundingClientRect();
      const children = [...overlay.children].map(node => node.getBoundingClientRect());
      const top = Math.min(...children.map(rect => rect.top));
      const bottom = Math.max(...children.map(rect => rect.bottom));
      return { text: detail.textContent, height: bounds.height,
        centered: Math.abs((top + bottom) / 2 - (bounds.top + bounds.bottom) / 2) < 3,
        contained: text.left >= bounds.left && text.right <= bounds.right && text.bottom <= bounds.bottom,
        tint: getComputedStyle(overlay).backgroundColor,
        overflow: overlay.scrollWidth > overlay.clientWidth };
    })()`),
    );
    assert.equal(errorLayout.text, diagnostic);
    assert.ok(errorLayout.height >= 230 && errorLayout.centered, "The diagnostic must occupy a centered full-height plot overlay");
    assert.ok(errorLayout.contained && !errorLayout.overflow, "Long diagnostics must wrap inside the plot");
    assert.notEqual(errorLayout.tint, "rgba(0, 0, 0, 0)");
    await writeFile(join(evidence, "plot-error-overlay.json"), JSON.stringify(errorLayout, null, 2));
  } finally {
    socket?.close();
    await terminate(chromeProcess);
    await new Promise((closed) => server.close(closed));
  }
  const report = dom.replace(/<script[\s\S]*?<\/script>/g, "").replace(/<[^>]+>/g, " ");
  const geometry = dom.match(/<pre id="parameter-geometry-proof"[^>]*>([^<]+)<\/pre>/);
  assert.ok(geometry, "The frozen report must mount its parameter grid");
  assertParameterGeometry(JSON.parse(geometry[1]));
  const hovers = dom.match(/<pre id="prompt-hover-proof"[^>]*>([^<]+)<\/pre>/);
  assert.ok(hovers, "The frozen report must mount its prompt markers");
  assertPromptHovers(JSON.parse(hovers[1]));
  const histogram = dom.match(/<pre id="histogram-proof"[^>]*>([^<]+)<\/pre>/);
  assert.ok(histogram, "The frozen report must mount its histogram");
  assertHistogram(JSON.parse(histogram[1]));
  assert.ok(hasText(report, "Frozen Report"), "The report's page bar must name what the file is");
  assert.ok(/Frozen Report .{0,3} Generated/.test(normalize(report)), "The report must state when it was generated");
  /* Step names may say "Start", so the controls are read off the buttons; the
     member tree's buttons name cases such as "Start-Up Sequence #1". */
  // Trace visibility buttons retain authored names, including a trace called "start".
  const controls = reportButtons.filter((button) => !button.trace).map((button) => normalize(button.label));
  const runControls = controls.filter((label) =>
    /^(?:(?:Start|Stop|Pause|Resume|Restart|Cancel)(?: (?:Execution|Cycle))?|Run Again(?: .+)?|Re-run)$/i.test(label),
  );
  assert.deepEqual(runControls, [], "A frozen report renders no run controls");
  assert.ok(hasText(report, "Records"), "The report renders its records section");
  assert.equal(
    (dom.match(/<li\b[^>]*\bdata-level="case"/g) ?? []).length,
    liveMembers,
    "The frozen report must list every planned execution the live cycle lists, pending ones included",
  );
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
    60000,
  );
  assert.ok(hasText(home, "Needs Attention"), "The groups keep their order at touch");

  await session.goto(`/cycles/${cycleId}?view=cycle`);
  await until("touch workspace", () => session.exists("button", "Cycle Actions"), 60000);
  /* The rail: collapsed at touch, with the children behind it. */
  if (await session.exists("button", "Collapse Sidebar")) {
    await session.clickNamed("button", "Collapse Sidebar");
  }
  await until("collapsed rail", () => session.exists("button", "Expand Sidebar"), 30000);
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
  await until("touch library", async () => hasText(await session.text(), PROTECTION_SUITE), 60000);
  await session.clickNamed("button", PROTECTION_SUITE);
  await until("touch template", async () => hasText(await session.text(), "Configure"), 30000);
  await session.clickNamed("button", "Configure");
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
  await delay(500);

  await setDensity(session, densities.desktop);
}

/**
 * Run one journey, unless the filter excludes it.
 *
 * A skipped journey is neither a pass nor a failure: it produced no evidence,
 * so it states that and nothing else. `startup` and `library` ignore the
 * filter because everything after them addresses what they establish.
 */
async function journey(name, run) {
  const started = Date.now();
  if (selectedJourneys.size && !selectedJourneys.has(name) && !REQUIRED_JOURNEYS.has(name)) {
    process.stdout.write(`\n== ${name}\n   skipped by DOKIME_E2E_JOURNEYS\n`);
    results.push({ name, seconds: 0, skipped: "Excluded by the journey filter" });
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
    const value = await run();
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
  }
}

async function terminate(child, signal = "SIGTERM") {
  if (!child?.pid || child.exitCode !== null || child.signalCode !== null) return;
  if (windows) {
    await promisify(execFile)("taskkill.exe", ["/PID", String(child.pid), "/T", "/F"], { windowsHide: true });
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
  await promisify(execFile)(resolve(installer), ["/S", `/D=${installation}`], {
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

async function closeSession() {
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

async function stopBackend() {
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
      delay(30000, undefined, { signal: timer.signal }).then(() => {
        throw new Error("Native backend did not drain within its existing 30 second deadline");
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
  const closing = await session.executeAsync(`
    const done = arguments[arguments.length - 1];
    globalThis.__TAURI_INTERNALS__.invoke('desktop_session_status').then(done, (error) => done({error: String(error)}));
  `);
  assert.equal(closing.ownership, "owned_local", "the cycle's current backend remains owned by this shell");
  assert.ok(closing.connection?.endpoint, "the current embedded session has an endpoint before window close");
  assert.ok(await listening(closing.connection.endpoint), "the current owned listener is live immediately before window close");
  const idleStarted = performance.now();
  await closeSession();
  await until("owned backend drains on window close", async () => !(await listening(closing.connection.endpoint)), 30000);
  const idleMs = performance.now() - idleStarted;
  session = await startSession(work, undefined, evidence);
  await session.connected();
  await emptyCycleHistory(session, cycleId, "Managed Shutdown Evidence", "owned_local");
  await installSaveTap(session);
  await stagePluginThroughPicker(session);
  await closeSession();
  session = await startSession(work, undefined, evidence);
  await session.connected();
  const runningCycle = await libraryJourney(session);
  assert.ok(await startCaseFromItsRow(session, STARTUP_TEMPLATE), "The owned backend starts its installed plugin");
  await until("owned execution awaits its operator", async () => hasText(await session.text(), STARTUP_PROMPT), 60000);
  const running = await session.executeAsync(`
    const done = arguments[arguments.length - 1];
    globalThis.__TAURI_INTERNALS__.invoke('desktop_session_status').then(done, (error) => done({error: String(error)}));
  `);
  assert.equal(running.ownership, "owned_local");
  const runningStarted = performance.now();
  await closeSession();
  await until("running owned backend drains on close", async () => !(await listening(running.connection.endpoint)), 30000);
  const runningMs = performance.now() - runningStarted;
  session = await startSession(work, undefined, evidence);
  await session.connected();
  await session.goto(`/history/${runningCycle.id}`);
  await until("cancelled owned execution survives restart", async () =>
    session.execute(
      `
    const row = [...document.querySelectorAll('section')].find((node) =>
      node.querySelector('h2')?.textContent.startsWith(arguments[0]));
    return row?.innerText.includes('cancelled') && !row.innerText.toLowerCase().includes('interrupted');
  `,
      [STARTUP_TEMPLATE],
    ),
  );
  assert.ok(await session.exists("button", "Open Report"), "Orderly close preserves the sealed cycle report");
  await session.screenshot("owned-running-recovered");
  await closeSession();
  await writeFile(join(evidence, "owned-startup.json"), JSON.stringify({ initial: status, closed: closing, idleMs, runningMs }, null, 2));
}

async function idleJourney() {
  await session.goto("/library");
  await installSaveTap(session);
  const library = await idleInvokes("Library");
  const cycleId = await createEmptyContainer(session, "Idle Refresh Evidence");
  await cyclePhase(session, "Active");
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
    const latest = calls.at(-1)?.at ?? 0;
    return !pending && Date.now() - latest >= 6000;
  `),
  );
  const started = await session.execute("return Date.now();");
  await delay(10000);
  const measured = await session.execute(
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
  return measured;
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
  const result = await promisify(execFile)(cliBinary, ["cycle", "export", cycleId, "--endpoint", endpoint, "--output", archive, "--json"], {
    windowsHide: true,
    cwd: work,
    timeout: 60000,
  });
  const receipt = JSON.parse(result.stdout);
  const bytes = await readFile(archive);
  assert.equal(receipt.size_bytes, bytes.length);
  assert.equal(receipt.sha256, createHash("sha256").update(bytes).digest("hex"));
  assert.equal(bytes.readUInt32LE(0), 0x04034b50, "saved evidence is a ZIP archive");
}

async function restartJourney() {
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
  backend = await startBackend(work, evidence, endpoint, "backend-restart");
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
  await until("saved evidence survives native restart", async () => (await session.findAll("css selector", "tbody tr")).length > 0, 60000);
  await setDensity(session, densities.desktop);
  const cycle = await libraryJourney(session);
  await cycleJourney(session, cycle.id);
  await session.screenshot("native-restart");
}

async function pluginInstallJourney() {
  const previous = backend;
  const root = join(work, "plugin-install");
  await mkdir(root, { recursive: true });
  await closeSession();
  try {
    backend = await startBackend(root, evidence, undefined, "plugin-install", false);
    session = await startSession(root, backend.endpoint, evidence);
    await sizeViewport(session, densities.desktop);
    await installSaveTap(session);
    await stagePluginThroughPicker(session);

    await closeSession();
    await stopBackend();
    backend = await startBackend(root, evidence, undefined, "plugin-activation", false);
    session = await startSession(root, backend.endpoint, evidence);
    await sizeViewport(session, densities.desktop);
    await session.goto("/settings/plugins");
    await until(
      "installed bundle active after restart",
      () =>
        session.execute(`
      const row = [...document.querySelectorAll('[aria-label="Installed Plugins"] tbody tr')]
        .find(row => row.innerText.includes('obc-plugin'));
      return Boolean(row && [...row.querySelectorAll('[data-slot="badge"]')].some(badge => badge.innerText === 'Active'));
    `),
      60000,
    );
    await session.goto("/library");
    await until("installed bundle contributes its catalog", async () => hasText(await session.text(), PROTECTION_SUITE), 60000);
    await session.screenshot("plugin-activation");
  } finally {
    await closeSession();
    if (backend !== previous) await stopBackend();
    backend = previous;
    session = await startSession(work, backend.endpoint, evidence);
    await sizeViewport(session, densities.desktop);
    await installSaveTap(session);
  }
}

async function stagePluginThroughPicker(session) {
  await session.goto("/settings/plugins");
  await until("plugin installer", async () => hasText(await session.text(), "Install A Plugin"), 60000);
  await session.execute("globalThis.__dokimeInvokes.length = 0;");
  await session.clickNamed("button", "Choose File");
  const picker = { window: null };
  await until("native plugin file picker", () => chooseNativeFile(picker, pluginBundle), 30000);
  await until("inspected plugin consent", async () => hasText(await session.text(), "Review Plugin"), 300000);
  const digest = await sha256(pluginBundle);
  assert.ok(hasText(await session.text(), digest), "Consent must display the digest of the selected native file");
  const invokes = await session.execute(
    "return globalThis.__dokimeInvokes.filter(entry => entry.command === 'desktop_plugin_stage_file');",
  );
  assert.equal(invokes.length, 1, "A bundle uses one native staging invocation");
  assert.deepEqual(invokes[0].keys.sort(), ["drop", "generation", "progress"], "Staging passes control arguments without bundle bytes");
  assert.ok(invokes[0].size < 1024, "The staging invocation stays bounded independently of bundle size");
  await session.clickNamed("button", "Install plugin");
  await until("saved plugin consent", async () => hasText(await session.text(), "Plugin changes are saved"), 30000);
  await session.clickNamed("button", "Later");
  await until("plugin restart requirement", async () => hasText(await session.text(), "Restart Required"), 30000);
  await session.screenshot("plugin-install");
}

async function chooseNativeFile(picker, file) {
  if (!windows) {
    const children = await readFile(`/proc/${driverProcess.pid}/task/${driverProcess.pid}/children`, "utf8");
    const executable = await realpath(shellBinary);
    for (const pid of children.trim().split(/\s+/)) {
      if (!pid || (await realpath(`/proc/${pid}/exe`)) !== executable) continue;
      const run = (args) => promisify(execFile)("xdotool", args, { windowsHide: true, timeout: 30000 });
      const found = await run(["search", "--all", "--onlyvisible", "--pid", pid, "--name", "^Open File$"]).catch((error) => {
        if (error.code === 1) return null;
        throw error;
      });
      if (!found) return picker.window !== null;
      const ids = found.stdout.trim().split(/\s+/);
      assert.equal(ids.length, 1, "The owned desktop exposes exactly one native file picker");
      if (picker.window !== null) {
        assert.equal(ids[0], picker.window, "Selection remains with its original native dialog");
        await run(["windowfocus", "--sync", picker.window]);
        const focus = await run(["getwindowfocus"]);
        await appendFile(join(evidence, "native-picker.log"), `picker window=${picker.window} focus=${focus.stdout.trim()} submit\n`);
        await run(["key", "--clearmodifiers", "Return"]);
        return false;
      }
      await run(["windowfocus", "--sync", ids[0]]);
      await run(["key", "--clearmodifiers", "ctrl+l"]);
      await run(["type", "--clearmodifiers", "--delay", "0", "--", resolve(file)]);
      await appendFile(join(evidence, "native-picker.log"), `picker window=${ids[0]} file=${resolve(file)} entered\n`);
      picker.window = ids[0];
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
      [Console]::Error.WriteLine(('picker shell={0} file={1} window={2}' -f $env:DOKIME_PICKER_SHELL, $env:DOKIME_PICKER_FILE, $env:DOKIME_PICKER_WINDOW))
      foreach ($process in Get-Process | Where-Object { $_.Path -eq $env:DOKIME_PICKER_SHELL }) {
        [Console]::Error.WriteLine(('picker process={0}' -f $process.Id))
        $pidCondition = New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::ProcessIdProperty, $process.Id)
        foreach ($window in $root.FindAll([System.Windows.Automation.TreeScope]::Children, $pidCondition)) {
          [Console]::Error.WriteLine(('picker handle={0} title={1} offscreen={2}' -f $window.Current.NativeWindowHandle, $window.Current.Name, $window.Current.IsOffscreen))
          if ($env:DOKIME_PICKER_WINDOW) {
            if ($window.Current.NativeWindowHandle.ToString() -eq $env:DOKIME_PICKER_WINDOW) {
              Write-Output 'open'
              exit 0
            }
            continue
          }
          $fieldId = New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::AutomationIdProperty, '1148')
          $fieldType = New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::ControlTypeProperty, [System.Windows.Automation.ControlType]::Edit)
          $fieldCondition = New-Object System.Windows.Automation.AndCondition($fieldId, $fieldType)
          $field = $window.FindFirst($scope, $fieldCondition)
          $buttonId = New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::AutomationIdProperty, '1')
          $buttonType = New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::ControlTypeProperty, [System.Windows.Automation.ControlType]::Button)
          $buttonCondition = New-Object System.Windows.Automation.AndCondition($buttonId, $buttonType)
          $button = $window.FindFirst($scope, $buttonCondition)
          if (!$field -or !$button) {
            foreach ($candidate in $window.FindAll($scope, [System.Windows.Automation.Condition]::TrueCondition)) {
              if ($candidate.Current.AutomationId -in @('1148', '1')) {
                [Console]::Error.WriteLine(('picker control id={0} type={1} class={2}' -f $candidate.Current.AutomationId, $candidate.Current.ControlType.ProgrammaticName, $candidate.Current.ClassName))
              }
            }
          }
          if ($field -and $button) {
            $selectedWindow = $window.Current.NativeWindowHandle
            $field.SetFocus()
            $field.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern).SetValue($env:DOKIME_PICKER_FILE)
            [Console]::Error.WriteLine(('picker entered={0}' -f $field.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern).Current.Value))
            $button.SetFocus()
            $button.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
            Write-Output ('selected:' + $selectedWindow)
            exit 0
          }
        }
      }
      if ($env:DOKIME_PICKER_WINDOW) { Write-Output 'closed' }`;
  const result = await promisify(execFile)("powershell.exe", ["-NoProfile", "-NonInteractive", "-Command", script], {
    windowsHide: true,
    env: {
      ...process.env,
      DOKIME_PICKER_SHELL: resolve(shellBinary),
      DOKIME_PICKER_FILE: resolve(file),
      DOKIME_PICKER_WINDOW: picker.window === null ? "" : String(picker.window),
    },
    timeout: 30000,
  });
  await appendFile(join(evidence, "native-picker.log"), `${result.stderr}\n${result.stdout}\n`);
  const outcome = result.stdout.trim();
  if (outcome.startsWith("selected:")) {
    picker.window = Number(outcome.slice("selected:".length));
    assert.ok(Number.isSafeInteger(picker.window), "The selected native dialog has a window handle");
  }
  return outcome === "closed";
}

async function organizationJourney() {
  const root = join(work, "shell", "application", "organization");
  const archive = join(work, "organization.zip");
  const manifest = `[app]
label = "Validation Lab"
title_template = "{app}"
slug = "ignored-lab"
identifier = "invalid.replacement.identity"
deep_link_scheme = "ignored-lab"
[app.logos.transparent]
on_light = "mark.svg"
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
oauth_client_id = "native-validation"
`;
  await promisify(execFile)(
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
    { windowsHide: true, timeout: 30000 },
  );
  const choose = async () => {
    await session.goto("/settings/appearance");
    await session.clickNamed("button", "Choose File…");
    const picker = { window: null };
    await until("native organization file picker", () => chooseNativeFile(picker, archive), 30000);
    await until("pending organization pack", async () => hasText(await session.text(), "Applies on next start."));
  };
  const restart = async () => {
    await closeSession();
    session = await startSession(work, backend.endpoint, evidence);
    await sizeViewport(session, densities.desktop);
    await session.connected();
    await session.goto("/settings/appearance");
  };
  await choose();
  assert.equal(
    await session.execute("return window.__DOKIME_ORGANIZATION_PACK__.pack;"),
    null,
    "Staging does not change this boot's appearance",
  );
  await restart();
  await until("native organization activation", async () => hasText(await session.text(), "Active since this start."));
  assert.ok(hasText(await session.text(), "Validation Lab"), "The mounted shell applies the imported label");
  assert.equal(
    await readFile(join(root, "active", "organization.toml"), "utf8"),
    manifest,
    "Compiled storage identity owns the active pack",
  );
  assert.equal(existsSync(join(root, "pending")), false, "Activation promotes the pending directory");
  await session.screenshot("organization-active");

  await session.goto("/settings/account-updates");
  await session.clickNamed("button", "Manage");
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
  await until("named release connection", async () => hasText(await session.text(), "Public Access"));
  assert.ok(hasText(await session.text(), "Sign In With GitLab"));
  assert.equal(await checkDisabled(), true);
  await select("Release Source", "public");
  await select("Update Checks", "manual");
  await session.execute(
    `const button = [...document.querySelectorAll('[role="dialog"] button')].find(button => button.textContent.trim() === 'Check For Updates'); button.click();`,
  );
  await until("missing release source error", async () =>
    session.execute(`return Boolean(document.querySelector('[role="dialog"] [role="alert"]')?.textContent);`),
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
  await restart();
  await session.goto("/settings/account-updates");
  await session.clickNamed("button", "Manage");
  await until("saved release source survives restart", async () => (await selection("Release Source")) === "private");
  assert.equal(await selection("Update Checks"), "disabled");
  await restart();

  await choose();
  await writeFile(join(root, "pending", "organization.toml"), "invalid [");
  await restart();
  assert.ok(hasText(await session.text(), "Validation Lab"), "An invalid pending pack retains the active appearance");
  assert.equal(await readFile(join(root, "active", "organization.toml"), "utf8"), manifest);
  assert.equal(
    await readFile(join(root, "pending", "organization.toml"), "utf8"),
    "invalid [",
    "Rejected input remains available for diagnosis",
  );
  await session.goto("/settings/diagnostics");
  await until("visible organization activation error", async () => /pending organization pack/i.test(await session.text()));
  await session.screenshot("organization-invalid-pending");
  await session.goto("/settings/appearance");
  await session.clickNamed("button", "Remove");
  await until("organization removal queued", async () => hasText(await session.text(), "Compiled brand applies on next start."));
  await restart();
  assert.equal(await session.execute("return window.__DOKIME_ORGANIZATION_PACK__.pack;"), null);
  assert.ok(hasText(await session.text(), "Compiled Brand"), "Removal restores the compiled appearance on restart");
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
  const skipped = failed.length === 0 && (cases.length === 0 || cases.some((result) => result.skipped));
  const seconds = cases.reduce((total, result) => total + result.seconds, 0);
  const detail = cases
    .map((result) => `${result.name}: ${result.failure ?? (result.skipped ? "Skipped" : "Passed")} (${result.seconds}s)`)
    .join("\n");
  const outcome = failed.length
    ? `<failure message="${escapeXml(`${failed.length} scene(s) failed`)}">${escapeXml(detail)}</failure>`
    : skipped
      ? "<skipped/>"
      : "";
  const story = `<testcase classname="dokime.native.${process.platform}" name="Desktop Backend Walkthrough" time="${seconds}">${outcome}<system-out>${escapeXml(detail)}</system-out></testcase>`;
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
    const result = await promisify(execFile)(
      "dbus-send",
      [
        "--session",
        "--print-reply",
        "--dest=org.freedesktop.DBus",
        "/org/freedesktop/DBus",
        "org.freedesktop.DBus.NameHasOwner",
        "string:org.freedesktop.secrets",
      ],
      { windowsHide: true, timeout: 30000 },
    );
    return result.stdout.includes("boolean true");
  });
}

async function main() {
  if (!windows && process.platform !== "linux") throw new Error("Native acceptance requires Windows WebView2 or Linux WebKitGTK.");
  if (releaseValidation && selectedJourneys.size) throw new Error("Installed release validation requires every maintained journey.");
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
  const owned = await journey("owned-startup", ownedStartupJourney);
  if (!owned.ok) throw new Error("Native ownership acceptance failed");
  backend = await startBackend(work, evidence);
  session = await startSession(work, backend.endpoint, evidence);
  await sizeViewport(session, densities.desktop);

  await journey("startup", () => startupJourney(session, notes));
  await installSaveTap(session);

  /* The Library is where cases come from, so it runs whatever the home said:
     a wrong count on Cycles is not a reason to certify nothing behind it. */
  const library = await journey("library", () => libraryJourney(session));
  const cycleId = library.value?.id;
  await journey("active-cycle", () => activeCycleJourney(session, library.value?.title ?? null));

  if (cycleId) {
    await journey("cycle", () => cycleJourney(session, cycleId));
    /* Report and touch read the original cycle; the copy owns an independent
       container and keeps the original executions unchanged. */
    await journey("report", () => reportJourney(session, cycleId));
    await journey("touch", () => touchJourney(session, cycleId));
    await journey("execution-stop", () => executionStopJourney(session, cycleId));
    await journey("containers", () => containersJourney(session, cycleId));
    await journey("cycles-home", () => cyclesHomeJourney(session, library.value.title));
    await journey("history", () => historyJourney(session, cycleId, library.value.title, library.value.measuringExecutionId));
  } else {
    for (const name of ["cycle", "report", "touch", "execution-stop", "containers", "cycles-home", "history"]) {
      failures.push({
        kind: "failure",
        name,
        reason: "skipped: the library journey produced no cycle",
      });
    }
  }

  await journey("idle", idleJourney);
  await journey("settings", () => settingsJourney(session, notes));
  await journey("plugin-install", pluginInstallJourney);
  await journey("organization", organizationJourney);
  await journey("remote-access", remoteAccessJourney);

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
    assert.ok(await session.exists("button", "Seal Cycle"), "reconnecting preserves Active admission");
  });
  await journey("restart", restartJourney);
}

/* === Entry Point === */

try {
  await main();
} catch (failure) {
  failures.push({ kind: "failure", name: "harness", reason: redact(failure.message) });
  process.stdout.write(`\nHarness failure: ${redact(failure.message)}\n`);
} finally {
  for (const [name, cleanup] of [
    ["shell shutdown", closeSession],
    [
      "report browser cleanup",
      async () => {
        await reportBrowser?.close();
        reportBrowserClosed = true;
      },
    ],
    ["backend shutdown", stopBackend],
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
      await until("secret service stopped", () => child.exitCode !== null || child.signalCode !== null, 30000);
    } catch (error) {
      failures.push({ kind: "failure", name: "secret service cleanup", reason: error.message });
    }
  }
  if (installerAttempted) {
    try {
      const uninstallers = (await readdir(installation)).filter((name) => /^uninstall.*\.exe$/i.test(name));
      assert.equal(uninstallers.length, 1, "the candidate installs its native uninstaller");
      await promisify(execFile)(join(installation, uninstallers[0]), ["/S", `_?=${installation}`], {
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
