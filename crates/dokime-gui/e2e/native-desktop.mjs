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
//   f. settings     every section, the density switch, storage, plugins, diagnostics
//   f'. remote-access  TLS/key setup, saved credentials, restart, rotation and lab modes
//   g. report       Download Report, then the saved file rendered in headless Chrome
//   h. touch        the same surfaces at touch density: collapsed rail, Executions sheet
//
// Windows Requires An Interactive Account, WebView2 And A Version-
// Matched Microsoft Edge Driver. --release Requires A Disposable Account And
// DOKIME_E2E_INSTALLER Selects The Native NSIS
// Artifact; DOKIME_E2E_{INSTALLER,SHELL,BACKEND,CLI}_SHA256 Bind Its Installed
// Executables To The Candidate. DOKIME_E2E_PLUGIN And DOKIME_E2E_PLUGIN_SHA256
// Select And Verify Its Prepared OBC Fixture.
// The Optional --junit Path Is Supplied By The Repository Evidence Runner.
//
// Two deliberate substitutions, both of the operating system rather than of the
// product:
//
//   * `xdg-open` is a no-op script on PATH, so Open Report and "Open Folder"
//     cannot spawn a browser or a file manager over the session.
//   * `desktop_save_document` opens a native GTK/portal save dialog no WebDriver
//     session can reach, which would hang the gate on Download Report. The gate
//     taps that one command at the Tauri IPC boundary and writes the bytes the application produced into `<evidence>/downloads`
//     itself, which is what the picker would have done. Every domain RPC still
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

import { copyFile, mkdir, mkdtemp, open, readdir, readFile, rm, writeFile } from "node:fs/promises";

import http from "node:http";

import net from "node:net";
import { tmpdir } from "node:os";

import { basename, dirname, join, resolve } from "node:path";

import { setTimeout as delay } from "node:timers/promises";

import { fileURLToPath } from "node:url";

import { parseArgs, promisify } from "node:util";

// Third Party Imports

// Local Imports

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
let backendBinary = process.env.DOKIME_E2E_BACKEND ?? join(built, `bin/dokime-server${executableSuffix}`);
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
    socket.setTimeout(1000, () => {
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

async function startBackend(work, evidence, endpoint, label = "backend") {
  const port = endpoint ? Number(new URL(endpoint).port) : await unusedPort();
  const log = await open(join(evidence, `${label}.log`), "w");
  const child = spawn(
    backendBinary,
    [
      "--shutdown-stdin",
      "--plugin",
      pluginBundle,
      "--port",
      String(port),
      "--output-root",
      join(work, "data"),
      "--application-root",
      join(work, "application"),
    ],
    {
      cwd: work,
      detached: true,
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
  const driver = process.env.DOKIME_E2E_DRIVER ?? (windows ? "msedgedriver.exe" : "WebKitWebDriver");
  driverProcess = spawn(driver, ["--host=127.0.0.1", `--port=${port}`], {
    detached: true,
    env: {
      ...process.env,
      TAURI_WEBVIEW_AUTOMATION: "true",
      DOKIME_APPLICATION_ROOT: join(home, "application"),
      ...(windows
        ? {
            WEBVIEW2_USER_DATA_FOLDER: join(home, "webview"),
          }
        : {
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
  });
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
              args: endpoint ? ["--endpoint", endpoint] : [],
            },
          }
        : {
            browserName: "wry",
            "webkitgtk:browserOptions": {
              args: endpoint ? ["--endpoint", endpoint] : [],
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
  session.clickCss = async (selector) => session.click(await until(selector, () => session.find("css selector", selector)));
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
      style.textContent = 'input[aria-label="Generated Access Key"] { visibility: hidden !important; }';
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
      await session.request("DELETE", "/window");
    } finally {
      session.closed = true;
    }
  };

  await until("main native window", async () => {
    const handles = await session.request("GET", "/window/handles");
    if (!handles.length) return false;
    await session.request("POST", "/window", { handle: handles[0] });
    return true;
  });
  const initial = await session.location();
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
 * tap records the document the application actually produced -- the same
 * verified bytes the picker would have written -- and nothing else: every
 * other command, and every domain RPC, still goes to the real backend.
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
            entry.state = 'settled';
            entry.ms = Date.now() - entry.at;
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
      length: document.bytesBase64.length,
    }));
  `);
  const written = [];
  for (const [index, document] of documents.entries()) {
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
    assert.equal(basename(document.filename), document.filename, "saved documents have a plain filename");
    const bytes = Buffer.from(base64, "base64");
    let target = join(session.downloads, document.filename);
    if (document.kind === "remote_key") {
      assert.ok(bytes.toString("utf8").trim().length > 0, "the saved access key is nonempty");
      secrets.add(base64);
      secrets.add(bytes.toString("utf8").trim());
      const directory = join(work, "credentials");
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
    20000,
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
    20000,
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
  /* The cycle page answers nothing an execution asked: it names where. */
  const cycleSighting = await session.execute(`
    const needed = document.querySelector('[data-input-needed]');
    if (!needed) return null;
    return {
      text: needed.innerText,
      respond: [...document.querySelectorAll('button')].some((button) => button.innerText.trim() === 'Respond'),
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
    return {
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
    20000,
  );
  /* A row menu is a modal layer: the next row is unreachable until it is gone. */
  await closeAnyMenu(session);
  await session.clickNamed("button", target.trigger);
  await until("the row menu", () => session.find("css selector", "[role=menu]"), 15000);
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
    20000,
  ).catch(() => false);
}

/** Press a menu trigger until its menu is up, and answer with the menu. */
async function openMenu(session, name, attempts = 2) {
  let last;
  for (let attempt = 0; attempt < attempts; attempt += 1) {
    await session.clickNamed("button", name);
    const menu = await until(`the ${name} menu`, () => session.find("css selector", "[role=menu]"), 8000).catch((failure) => {
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
    10000,
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
  await until("added chip", async () => ((await chips()) ?? []).length === before.length + 1, 15000);
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
    15000,
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
  await until("the cycle page", async () => (await session.location()).searchParams.get("view") === "cycle", 10000);
  const header = await session.text();
  assert.ok(/\d+ of \d+ cases finished/.test(normalize(header)), "The header must state progress");
  assert.ok(hasText(header, "Ready"), "A materialized cycle must render its Ready status");
  assert.ok(await session.exists("button", "Cycle Actions"), "The header must carry the overflow menu");
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

  /* The Overview Offers Every Answerable Prompt; Execution Pages Keep Their Own. */
  await session.execute("document.querySelector('nav[aria-label=Navigation] li[data-level=overview] button[data-tree-select]').click();");
  await until(
    "the cycle selection for operator prompts",
    async () => (await session.location()).searchParams.get("view") === "cycle",
    10000,
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
  /* The Cycle Page Names Where Input Is Needed And Answers Nothing Itself. */
  for (const seen of cycleInputSightings) {
    assert.match(seen.text, /Input needed on #\d+/, "The cycle page must name the execution waiting on input");
    assert.ok(!seen.respond, "The cycle page must not offer Respond for an execution's prompt");
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
  await session.type(pointsInput, "2500");
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
  await session.clickCss(settingsSelector);
  const persistedPoints = await until(
    "persisted display point budget",
    () =>
      session.execute(`
        const input = document.querySelector('[role=dialog][data-state=open] input[type=number]');
        return input ? {value: input.value} : null;
      `),
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
  await until("focus flag set", async () => ["1", "true"].includes((await session.location()).searchParams.get("focus")), 20000);
  assert.ok(await session.exists("button", "Leave Focus"), "Focus view must say how to leave it");
  await session.clickNamed("button", "Leave Focus");
  await until("focus flag cleared", async () => (await session.location()).searchParams.get("focus") !== "1", 20000);

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
    20000,
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
  await until("the cycle page", () => session.exists("button", "Cycle Actions"), 10000);
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
  /* An attempt that has not started is cancelled, not stopped: Stop belongs
     to a running attempt alone. */
  await until("Cancel Execution on the unstarted attempt", () => session.exists("button", "Cancel Execution"), 30000);
  assert.ok(!(await session.exists("button", "Stop Execution")), "An unstarted execution must not offer Stop");
  assert.ok(!hasText(await session.text(), "No step in progress"), "An unstarted execution states that it waits for Start");
  await session.execute("document.querySelector('nav[aria-label=Navigation] li[data-level=overview] button[data-tree-select]').click();");
  await until("the cycle selection", async () => (await session.location()).searchParams.get("view") === "cycle", 10000);
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
  await until("the stop acknowledgement", async () => /Execution Stopped And Evidence Saved/.test(await toastText(session)), 20000);
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
  await until("the cancel acknowledgement", async () => /Execution Cancelled/.test(await toastText(session)), 20000);
  const cancelledRow = await until("the cancelled row", () =>
    session.execute(
      `const text = [...document.querySelectorAll('nav[aria-label=Navigation] li[data-level=case]')]
        .find((row) => row.dataset.executionId === arguments[0])?.innerText ?? '';
       return /\\b(Stopped|Skipped|Cancelled)\\b/.test(text) ? text : null;`,
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
  await until("default view", async () => !hasText(await session.text(), "No Archived Cycles"), 20000);

  const filter = await session.find("css selector", "input[aria-label='Filter Cycles']");
  await session.type(filter, "zzz-no-such-cycle");
  await until("filter excludes every row", async () => hasText(await session.text(), "No Cycles Match This Filter"), 20000);
  await session.type(filter, "");
  await until("filter cleared", async () => !hasText(await session.text(), "No Cycles Match This Filter"), 20000);
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
    () => session.find("xpath", `//tbody//button[normalize-space(.)=${xpathLiteral(cycleTitle)}]`),
    120000,
  );
  assert.ok(hasText(await session.text(), "History"), "The page bar must name History");
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
  await until("back to history", async () => (await session.location()).pathname === "/history", 20000);
}

/** Remote Access Uses Mounted Controls And Real TLS CLI Calls. */
async function remoteAccessJourney() {
  const input = (label) =>
    until(label, () => session.find("xpath", `//input[@id=//label[normalize-space(.)=${xpathLiteral(label)}]/@for]`));
  const toggle = async (label, checked) => {
    const control = await until(label, () =>
      session.find("xpath", `//*[@role='switch' and @id=//label[normalize-space(.)=${xpathLiteral(label)}]/@for]`),
    );
    const current = await session.execute("return arguments[0].getAttribute('aria-checked') === 'true';", [control]);
    if (current !== checked) await session.click(control);
  };
  const confirm = async (name, notice) => {
    await session.clickNamed("button", name);
    await session.clickNamed("button", "Confirm");
    await until(notice, async () => hasText(await session.text(), notice));
  };
  const endpoint = () =>
    until("remote listening address", async () => {
      const match = /Listening On (127\.0\.0\.1:\d+)/.exec(await session.text());
      return match ? `https://${match[1]}` : null;
    });
  const connect = async (address, keyFile, certificate, accepted) => {
    const args = ["catalog", "list", "--endpoint", address, "--timeout-seconds", "5"];
    if (keyFile) args.push("--key-file", keyFile);
    if (certificate) args.push("--ca", certificate);
    const env = { ...process.env, DOKIME_APPLICATION_ROOT: join(work, "cli") };
    delete env.DOKIME_KEY;
    delete env.DOKIME_ENDPOINT;
    let result;
    try {
      result = await promisify(execFile)(cliBinary, args, { cwd: work, env, timeout: 15000 });
    } catch (error) {
      assert.ok(!accepted, "the saved remote credentials establish a real CLI connection");
      assert.equal(error.killed, false, "credential refusal returns without killing the client");
      assert.ok(Number.isInteger(error.code) && error.code !== 0, "credential refusal has a native failure exit code");
      return;
    }
    assert.ok(accepted, "superseded or missing credentials must be refused");
    assert.ok(result.stdout.trim().length > 0, "the connected CLI reads the real catalog");
    assert.ok(
      ![...secrets].some((secret) => result.stdout.includes(secret) || result.stderr.includes(secret)),
      "CLI output contains no key",
    );
  };
  const saveKey = async () => {
    await until("one-time key dialog", async () => hasText(await session.text(), "Save Your Access Key"));
    const value = await session.execute("return document.querySelector('input[aria-label=\"Generated Access Key\"]').value;");
    assert.ok(typeof value === "string" && value.length > 0, "the one-time key is available");
    secrets.add(value);
    secrets.add(Buffer.from(`${value}\n`).toString("base64"));
    await session.clickNamed("button", "Copy Key");
    await until("native clipboard write", () => session.exists("button", "Copied"));
    await session.clickNamed("button", "Save Key File…");
    await until("saved key acknowledgement", async () => hasText(await session.text(), "Key File Saved"));
    await session.clickNamed("button", "Save Public Certificate…");
    await until("saved public certificate acknowledgement", async () => hasText(await session.text(), "Certificate Saved"));
    const documents = await drainSavedDocuments(session);
    const key = documents.find((document) => document.kind === "remote_key")?.path;
    const certificate = documents.find((document) => document.kind === "public_certificate")?.path;
    assert.ok(key && certificate, "the GUI produces both credential files");
    assert.ok((await readFile(key, "utf8")).trim() === value, "the saved key matches the one-time value");
    assert.match(await readFile(certificate, "utf8"), /-----BEGIN CERTIFICATE-----/);
    await session.clickNamed("button", "I Have Saved The Key");
    return { key, certificate };
  };

  // Enable TLS And Authentication Through The Native GUI.
  await session.goto("/settings/remote");
  await until("remote settings", async () => hasText(await session.text(), "Enable Remote Access"));
  await session.type(await input("Listen Address"), "127.0.0.1:0");
  await session.type(await input("Certificate DNS Names And IP Addresses"), "127.0.0.1, localhost");
  await toggle("TLS Encryption", true);
  await toggle("Require An Access Key", true);
  await confirm("Regenerate Certificate…", "Certificate Regenerated");
  await session.clickNamed("button", "Generate Access Key");
  const original = await saveKey();
  await toggle("Enable Remote Access", true);
  await confirm("Apply…", "Settings Applied");
  let address = await endpoint();
  await connect(address, original.key, original.certificate, true);
  await connect(address, null, original.certificate, false);

  // A Failed Bind Restores The Previous Listener And Leaves The Desktop Connected.
  const occupied = net.createServer();
  occupied.listen(0, "127.0.0.1");
  await once(occupied, "listening");
  try {
    await session.type(await input("Listen Address"), `127.0.0.1:${occupied.address().port}`);
    await session.clickNamed("button", "Apply…");
    await session.clickNamed("button", "Confirm");
    await until("listener rollback error", async () => hasText(await session.text(), "Change Failed"));
    assert.equal(await endpoint(), address, "failed listener replacement retains the original bound address");
    await connect(address, original.key, original.certificate, true);
  } finally {
    await new Promise((done) => occupied.close(done));
  }

  // Restart Both Clients And Server To Prove Persisted Settings And Credentials.
  const nativeEndpoint = backend.endpoint;
  await closeSession();
  await stopBackend();
  backend = await startBackend(work, evidence, nativeEndpoint, "backend-remote-restart");
  session = await startSession(work, nativeEndpoint, evidence);
  await session.connected();
  await session.goto("/settings/remote");
  address = await endpoint();
  assert.ok(await session.exists("button", "Rotate Access Key…"), "only the key verifier is restored");
  assert.equal(await session.execute("return document.querySelector('input[aria-label=\"Generated Access Key\"]') === null;"), true);
  const browserStorage = await session.execute("return JSON.stringify([Object.entries(localStorage), Object.entries(sessionStorage)]);");
  assert.ok(![...secrets].some((secret) => browserStorage.includes(secret)), "browser storage contains no access key");
  await connect(address, original.key, original.certificate, true);

  // Keep An Admitted Execution Waiting For Its Operator Throughout Rotation.
  const activeCycle = (await libraryJourney(session)).id;
  assert.ok(await startCaseFromItsRow(session, STARTUP_TEMPLATE), "the native GUI admits the rotation execution");
  await until("operator prompt before rotation", async () => hasText(await session.text(), STARTUP_PROMPT), 60000);
  await session.goto("/settings/remote");
  await confirm("Rotate Access Key…", "Save Your Access Key");
  const replacement = await saveKey();
  address = await endpoint();
  await connect(address, original.key, replacement.certificate, false);
  await connect(address, replacement.key, replacement.certificate, true);
  const previousCertificate = join(work, "credentials", "previous.pem");
  await copyFile(replacement.certificate, previousCertificate);
  await confirm("Regenerate Certificate…", "Certificate Regenerated");
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
  assert.ok(hasText(await session.text(), "Input needed on #"), "the cycle names the execution its surviving prompt waits on");
  const running = await caseRowText(session, STARTUP_TEMPLATE);
  assert.match(running, /\b(Running|Waiting)\b/, "rotation leaves the admitted execution running");
  await session.execute(
    `const row = [...document.querySelectorAll('nav[aria-label=Navigation] li[data-level=case]')]
      .find((candidate) => candidate.innerText.includes(arguments[0]));
     row.querySelector('button[data-tree-select]').click();`,
    [STARTUP_TEMPLATE],
  );
  await until("the surviving prompt on its execution", async () => hasText(await session.text(), STARTUP_PROMPT), 30000);
  await session.clickNamed("button", "Stop Execution");
  await until("rotation execution stopped", async () => hasText(await toastText(session), "Execution Stopped And Evidence Saved"));

  // Authentication And Encryption Are Independent Explicit Lab Choices.
  await session.goto("/settings/remote");
  await toggle("Require An Access Key", false);
  await confirm("Apply…", "Settings Applied");
  address = await endpoint();
  await connect(address, null, certificate, true);
  await toggle("Require An Access Key", true);
  await toggle("TLS Encryption", false);
  await confirm("Apply…", "Settings Applied");
  address = (await endpoint()).replace("https:", "http:");
  await connect(address, replacement.key, null, true);
  await connect(address, null, null, false);

  // Leave The Host In Its Verified TLS And Authentication Configuration.
  await toggle("TLS Encryption", true);
  await confirm("Apply…", "Settings Applied");
  await connect(await endpoint(), replacement.key, certificate, true);
  await session.screenshot("remote-access");
}

/** Settings: every section, and the rows the redesign promises in each. */
async function settingsJourney(session, notes) {
  const sections = [
    ["general", "General"],
    ["appearance", "Appearance"],
    ["identity", "Identity"],
    ["plugins", "Plugins"],
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
    20000,
  );
  await session.clickNamed("radio", "Comfortable");
  await until(
    "comfortable density applied",
    async () => (await session.execute("return document.documentElement.dataset.density")) === "comfortable",
    20000,
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
  await until("policy restart requirement", async () => hasText(await session.text(), "Restart the connected backend"), 20000);
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
    await until("back to plugins", async () => (await session.location()).pathname === "/settings/plugins", 20000);
  } else {
    notes.push("settings: no installed plugin declares settings, so the plugin page stayed shut");
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
      const button = document.querySelector('[data-level="case"] button');
      if (button) { clearInterval(selection); button.click(); }
    }, 50);
  </script>`;
  const server = http.createServer((_request, response) => {
    response.writeHead(200, { "content-type": "text/html; charset=utf-8" });
    response.end(html.replace("</body>", `${selectExecution}</body>`));
  });
  await new Promise((ready) => server.listen(0, "127.0.0.1", ready));
  const served = `http://127.0.0.1:${server.address().port}/report.html`;
  let dom = "";
  try {
    const chrome = process.env.DOKIME_E2E_CHROME ?? (windows ? "msedge.exe" : "google-chrome");
    /* Headless Chrome's virtual time budget is wall-clock blind: under build
       load the bundle may still be hydrating when the budget runs out, so the
       render is retried with a growing budget until the page bar has drawn. */
    for (const budget of [8000, 20000, 45000]) {
      const rendered = await promisify(execFile)(
        chrome,
        [
          "--headless=new",
          "--disable-gpu",
          "--no-sandbox",
          `--user-data-dir=${join(work, "chrome")}`,
          `--virtual-time-budget=${budget}`,
          "--dump-dom",
          served,
        ],
        { maxBuffer: 64 * 1024 * 1024 },
      );
      dom = rendered.stdout;
      if (/Frozen Report/.test(dom) && /Attempt 1 Of/.test(dom)) break;
    }
  } finally {
    await new Promise((closed) => server.close(closed));
  }
  const report = dom.replace(/<script[\s\S]*?<\/script>/g, "").replace(/<[^>]+>/g, " ");
  assert.ok(hasText(report, "Frozen Report"), "The report's page bar must name what the file is");
  assert.ok(/Frozen Report .{0,3} Generated/.test(normalize(report)), "The report must state when it was generated");
  /* Step names may say "Start", so the controls are read off the buttons. */
  const controls = [...dom.matchAll(/<button\b[^>]*>([\s\S]*?)<\/button>/g)].map((match) => normalize(match[1].replace(/<[^>]+>/g, " ")));
  const runControls = controls.filter((label) => /\bStart\b|Run Again|Pause|Stop\b/.test(label));
  assert.deepEqual(runControls, [], "A frozen report renders no run controls");
  assert.ok(hasText(report, "Records"), "The report renders its records section");
  assert.equal(
    (dom.match(/data-level="case"/g) ?? []).length,
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
  await until("collapsed rail", () => session.exists("button", "Expand Sidebar"), 20000);
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
  }
}

async function terminate(child, signal = "SIGTERM") {
  if (!child?.pid || child.exitCode !== null || child.signalCode !== null) return;
  if (windows) {
    await promisify(execFile)("taskkill.exe", ["/PID", String(child.pid), "/T", "/F"]);
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
  if (process.env.DOKIME_E2E_DEV_URL || process.env.DOKIME_E2E_SHELL || process.env.DOKIME_E2E_BACKEND) {
    throw new Error(
      "Windows acceptance drives the installed shell and backend with bundled frontend assets; development overrides are invalid.",
    );
  }
  const installer = process.env.DOKIME_E2E_INSTALLER;
  if (!installer || !existsSync(installer)) throw new Error("DOKIME_E2E_INSTALLER must name the candidate NSIS installer.");
  const checks = [
    ["installer", installer, process.env.DOKIME_E2E_INSTALLER_SHA256],
    ["shell", join(work, "installation", "dokime-gui.exe"), process.env.DOKIME_E2E_SHELL_SHA256],
    ["backend", join(work, "installation", "dokime-server.exe"), process.env.DOKIME_E2E_BACKEND_SHA256],
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
    windowsVerbatimArguments: true,
    timeout: 120000,
  });
  const hashes = {};
  for (const [label, path, expected] of checks.slice(1)) {
    hashes[label] = await sha256(path);
    assert.equal(hashes[label], expected, `installed ${label} matches the selected candidate`);
  }
  shellBinary = checks[1][1];
  backendBinary = checks[2][1];
  cliBinary = checks[3][1];
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
  assert.equal(status.ownership, "owned_local", "the native shell owns its companion backend");
  assert.ok(status.connection?.endpoint, "the owned companion reports its negotiated endpoint");
  assert.ok(await listening(status.connection.endpoint), "the native companion is listening");
  const cycleId = await createEmptyContainer(session, "Managed Shutdown Evidence");
  await cyclePhase(session, "Active");
  await session.screenshot("owned-startup");
  const closing = await session.executeAsync(`
    const done = arguments[arguments.length - 1];
    globalThis.__TAURI_INTERNALS__.invoke('desktop_session_status').then(done, (error) => done({error: String(error)}));
  `);
  assert.equal(closing.ownership, "owned_local", "the cycle's current backend remains owned by this shell");
  assert.ok(closing.connection?.endpoint, "the current owned companion has an endpoint before window close");
  assert.ok(await listening(closing.connection.endpoint), "the current owned listener is live immediately before window close");
  await closeSession();
  await until("owned backend drains on window close", async () => !(await listening(closing.connection.endpoint)), 30000);
  session = await startSession(work, undefined, evidence);
  await session.connected();
  await emptyCycleHistory(session, cycleId, "Managed Shutdown Evidence", "owned_local");
  await closeSession();
  await writeFile(join(evidence, "owned-startup.json"), JSON.stringify({ initial: status, closed: closing }, null, 2));
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
    "return (globalThis.__dokimeInvokes ?? []).filter((entry) => entry.command === 'desktop_open_document').length;",
  );
  await session.clickNamed("button", "Open Report");
  await until("the saved empty-cycle report opened", () =>
    session.execute(
      `
    const opened = (globalThis.__dokimeInvokes ?? []).filter((entry) => entry.command === 'desktop_open_document');
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

  await journey("settings", () => settingsJourney(session, notes));
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
  if (installerAttempted) {
    try {
      const uninstallers = (await readdir(installation)).filter((name) => /^uninstall.*\.exe$/i.test(name));
      assert.equal(uninstallers.length, 1, "the candidate installs its native uninstaller");
      await promisify(execFile)(join(installation, uninstallers[0]), ["/S", `_?=${installation}`], {
        windowsVerbatimArguments: true,
        timeout: 120000,
      });
    } catch (error) {
      failures.push({ kind: "failure", name: "installer cleanup", reason: error.message });
    }
  }
  if (work)
    await rm(join(work, "credentials"), { force: true, recursive: true }).catch(() =>
      failures.push({ kind: "failure", name: "credential cleanup", reason: "Private temporary credential removal failed" }),
    );
  if (work && !failures.length)
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
