// ***
//
// Dokime
//
// SPDX-License-Identifier: Apache-2.0
//
// ***

// Real Linux WebView ownership gate for the redesigned shell. No HTTP facade,
// no mocked domain IPC, no browser fixtures.
//
// The gate starts a throwaway backend with the Mini charger testbench plugin,
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
//   d. cycles-home  the groups after a run, Show Cleared, the filter box
//   e. history      rows, one execution, its checkpoint views, the Back chevron
//   f. settings     every section, the density switch, storage, plugins, diagnostics
//   g. report       Write Report, then the written file loaded in this WebView
//   h. touch        the same surfaces at touch density: collapsed rail, Cases sheet
//
// Two deliberate substitutions, both of the operating system rather than of the
// product:
//
//   * `xdg-open` is a no-op script on PATH, so "Open Folder" cannot spawn a file
//     manager over the session.
//   * `desktop_save_document` opens a native GTK/portal save dialog no WebDriver
//     session can reach, which would hang the gate on Download Bundle and Write
//     Report. The gate taps that one command at the Tauri IPC boundary and
//     writes the bytes the application produced into `<evidence>/downloads`
//     itself, which is what the picker would have done. Every domain RPC still
//     crosses the real transport, and so does `desktop_save_diagnostics_bundle`,
//     which needs no picker: the throwaway home is given an xdg-user-dirs
//     Downloads folder and the shell writes the archive there itself.

/* === Imports === */

// Standard Library Imports

import assert from "node:assert/strict";

import { execFile, spawn } from "node:child_process";

import { once } from "node:events";

import { existsSync } from "node:fs";

import { mkdir, mkdtemp, open, readdir, readFile, rm, writeFile } from "node:fs/promises";

import http from "node:http";

import net from "node:net";

import { dirname, join, resolve } from "node:path";

import { setTimeout as delay } from "node:timers/promises";

import { fileURLToPath } from "node:url";

import { promisify } from "node:util";

// Third Party Imports

// Local Imports

/* === Re-Exports === */

/* === Constants === */

const workspace = resolve(dirname(fileURLToPath(import.meta.url)), "../../..");

const targetRoot = resolve(workspace, process.env.CARGO_TARGET_DIR ?? "target");

/** The dev-built binaries and the bundle the gate drives, all overridable. */
const backendBinary = process.env.DOKIME_E2E_BACKEND ?? join(targetRoot, "debug/dokime");

const shellBinary = process.env.DOKIME_E2E_SHELL ?? join(targetRoot, "debug/dokime-gui");

const pluginBundle =
  process.env.DOKIME_E2E_PLUGIN ?? join(targetRoot, "debug/plugins/mini-sidecar.dokime");

/** A shell built without `tauri/custom-protocol` loads this origin. */
const configuredOrigin = process.env.DOKIME_E2E_DEV_URL;

const densities = {
  desktop: { density: "comfortable", height: 1000, width: 1600 },
  touch: { density: "touch", height: 1200, width: 1920 },
};

/** The suites and templates the Mini charger plugin registers, by rendered label. */
const PROTECTION_SUITE = "Overcurrent Protection";

const PROTECTION_TEMPLATE = "Overcurrent Protection";

const STARTUP_SUITE = "Charger Power-Up";

const STARTUP_TEMPLATE = "Power Up the Charger";

/** A cartesian list parameter, so chips and expressions have somewhere to go. */
const LIST_PARAMETER = "load_steps";

/** A plain number, so an edit has a Reset affordance to reveal. */
const NUMBER_PARAMETER = "frequency_hz";

const OPERATOR_NOTE = "Trace steady, no fault indicated.";

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
 * Write every tapped document into the evidence downloads directory.
 *
 * The payload crosses in slices. A written report is megabytes of base64, and
 * one `execute` carrying all of it does not come back -- the document was
 * already taken off the page's list by then, so it was lost rather than late,
 * and the gate reported a report that had in fact been written.
 */
const DOCUMENT_SLICE_CHARS = 512 * 1024;

const failures = [];
const passedJourneys = [];

const notes = [];

/** The journey whose WebView died, once one has: everything after it is moot. */
let webviewGone = null;

let session;

let backend;

let work;

let evidence;

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

function hasText(body, expected) {
  return normalize(body).toLowerCase().includes(normalize(expected).toLowerCase());
}

async function startBackend(work, evidence) {
  const port = await unusedPort();
  const log = await open(join(evidence, "backend.log"), "w");
  const child = spawn(
    backendBinary,
    [
      "serve",
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
      env: { ...process.env, DOKIME_TEST_HUB_OPERATOR: "Gate Operator", HOME: work },
      stdio: ["ignore", log.fd, log.fd],
    },
  );
  await until("backend listening", () => listening(`http://127.0.0.1:${port}`), 180000);
  return { child, endpoint: `http://127.0.0.1:${port}`, log };
}

/**
 * One WebKitWebDriver session over one attached shell.
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
  await writeFile(join(home, "bin", "xdg-open"), "#!/bin/sh\nexit 0\n", { mode: 0o755 });
  /* `desktop_save_diagnostics_bundle` resolves the Downloads folder through
     Tauri's path API, which reads xdg-user-dirs rather than the
     `XDG_DOWNLOAD_DIR` environment variable. Without this file the throwaway
     home has no Downloads folder and the command fails before it writes. */
  await writeFile(
    join(home, "config", "user-dirs.dirs"),
    `XDG_DOWNLOAD_DIR="${downloads}"\nXDG_DOCUMENTS_DIR="${downloads}"\n`,
  );

  const port = await unusedPort();
  const webdriver = spawn("WebKitWebDriver", ["--host=127.0.0.1", `--port=${port}`], {
    detached: true,
    env: {
      ...process.env,
      HOME: home,
      PATH: `${join(home, "bin")}:${process.env.PATH ?? ""}`,
      TAURI_WEBVIEW_AUTOMATION: "true",
      XDG_CACHE_HOME: join(home, "cache"),
      XDG_CONFIG_HOME: join(home, "config"),
      XDG_DATA_HOME: join(home, "data"),
      XDG_DOWNLOAD_DIR: downloads,
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
      signal: AbortSignal.timeout(120000),
    }).catch((cause) => {
      throw new Error(`WebDriver ${method} ${path} failed: ${cause.message}`, { cause });
    });
    const result = (await response.json()).value;
    if (!response.ok || result?.error) {
      throw new Error(
        `WebDriver ${path}: ${result?.error ?? response.status}: ${result?.message ?? ""}`,
      );
    }
    return result;
  }

  await until("WebKitWebDriver startup", async () => {
    if (webdriver.exitCode !== null) throw new Error("WebKitWebDriver exited before startup");
    await request("GET", "/status");
    return true;
  });

  const created = await request("POST", "/session", {
    capabilities: {
      alwaysMatch: {
        browserName: "wry",
        "webkitgtk:browserOptions": {
          args: ["--endpoint", endpoint],
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
  session.execute = (script, args = []) =>
    session.request("POST", "/execute/sync", { args, script });
  session.executeAsync = (script, args = []) =>
    session.request("POST", "/execute/async", { args, script });
  session.url = () => session.request("GET", "/url");
  session.location = async () => new URL(await session.url());
  session.text = async () => {
    session.lastBody = await session.execute("return document.body.innerText");
    return session.lastBody;
  };
  session.find = (using, value) => session.request("POST", "/element", { using, value });
  session.findAll = (using, value) => session.request("POST", "/elements", { using, value });
  session.textOf = (element) =>
    session.execute("return arguments[0].innerText || arguments[0].textContent || '';", [element]);
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
  session.clickCss = async (selector) =>
    session.click(await until(selector, () => session.find("css selector", selector)));
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
  session.resize = (width, height) =>
    session.request("POST", "/window/rect", { height, width, x: 0, y: 0 });
  session.viewport = () => session.execute("return [window.innerWidth, window.innerHeight]");
  session.screenshot = async (label) => {
    const bytes = await session.request("GET", "/screenshot");
    const target = join(evidence, `${label}.png`);
    await writeFile(target, Buffer.from(bytes, "base64"));
    return target;
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
    await writeFile(target, `${JSON.stringify(value, null, 2)}\n`);
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
    until(
      "connected shell",
      () =>
        session.execute("return Boolean(document.querySelector('nav[aria-label=Navigation]'));"),
      180000,
    );
  session.load = async (url) => {
    await session.request("POST", "/url", { url });
    await until(
      `load ${url}`,
      async () => (await session.execute("return document.body.innerText.length")) > 20,
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
    await until(
      `route ${path}`,
      async () => (await session.location()).pathname === new URL(session.origin + path).pathname,
      30000,
    );
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
  session.origin = configuredOrigin ?? `${initial.protocol}//${initial.host}`;
  return session;
}

/** Size the native window until the WebView reports the exact target viewport. */
async function sizeViewport(session, target) {
  let frame = { height: target.height, width: target.width };
  for (let attempt = 0; attempt < 8; attempt += 1) {
    await session.resize(frame.width, frame.height);
    await delay(350);
    const [width, height] = await session.viewport();
    if (width === target.width && height === target.height) return;
    frame = {
      height: frame.height + (target.height - height),
      width: frame.width + (target.width - width),
    };
  }
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
 * returns, which is what hung Write Report in every run. One layer down,
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
  await until(
    "Tauri IPC bridge",
    () => session.execute("return typeof globalThis.__TAURI_INTERNALS__?.invoke === 'function'"),
    60000,
  );
  const state = await session.execute(`
    globalThis.__dokimeSavedDocuments = globalThis.__dokimeSavedDocuments ?? [];
    globalThis.__dokimeInvokes = globalThis.__dokimeInvokes ?? [];
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
        if (!url.includes('ipc')) return original(input, init);
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
          return Promise.resolve(
            new Response('true', {
              headers: { 'content-type': 'application/json', 'Tauri-Response': 'ok' },
              status: 200,
            }),
          );
        }
        return original(input, init).then(
          function (value) {
            entry.state = 'settled';
            entry.ms = Date.now() - entry.at;
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
  assert.ok(
    state?.installed,
    `The save tap did not take on window.fetch (${JSON.stringify(state)})`,
  );
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
      throw new Error(
        `The tapped ${document.filename} transferred ${base64.length} of ${document.length} characters`,
      );
    }
    const target = join(session.downloads, document.filename);
    await writeFile(target, Buffer.from(base64, "base64"));
    written.push({ kind: document.kind, path: target });
  }
  /* Cleared only once every document is on disk, so a transfer that fails
     leaves the document on the page to try again rather than losing it. */
  if (documents.length) await session.execute("globalThis.__dokimeSavedDocuments = [];");
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
    async () =>
      (await session.execute("return document.documentElement.dataset.density")) ===
      variant.density,
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

/** Answer every prompt the open cycle is waiting on, through the banner. */
async function answerPendingPrompts(session) {
  const pending = await session.execute(`
    const banner = document.querySelector('section[aria-label="Pending Prompts"]');
    if (!banner) return 0;
    const respond = [...banner.querySelectorAll('button')].filter(
      (button) => button.innerText.trim() === 'Respond',
    );
    respond[0]?.click();
    return respond.length;
  `);
  if (!pending) return 0;
  await delay(600);
  await session.execute(
    `const note = arguments[0];
     const panel = document.querySelector('[aria-labelledby="active-prompt-title"]') ?? document.body;
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
    const panel = document.querySelector('[aria-labelledby="active-prompt-title"]');
    if (!panel) return false;
    const submit = [...panel.querySelectorAll('button')].find((button) =>
      /^(Submit|Confirm|Accept|Continue)$/.test(button.innerText.trim()),
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
  throw new Error(
    `Timed out: ${description}. The application said: ${announcement || "nothing at all"}`,
  );
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
  const before = await caseRowText(session, fragment);
  const trigger = await session.execute(
    `const fragment = arguments[0];
     const row = [...document.querySelectorAll('nav[aria-label=Navigation] li[data-level=case]')].find(
       (candidate) => candidate.innerText.includes(fragment),
     );
     if (!row) throw new Error('No case row holds ' + fragment);
     return row.querySelector('button[aria-label^="Actions For "]').getAttribute('aria-label');`,
    [fragment],
  );
  /* A row menu is a modal layer: the next row is unreachable until it is gone. */
  await closeAnyMenu(session);
  await session.clickNamed("button", trigger);
  await until("the row menu", () => session.find("css selector", "[role=menu]"), 15000);
  await session.clickNamed("menuitem", "Run Case");
  await closeAnyMenu(session);
  return until(
    `the ${fragment} case to start`,
    async () => {
      const row = await caseRowText(session, fragment);
      return row !== before || /Running|Waiting|Finalizing|Passed|Failed/.test(row);
    },
    20000,
  ).catch(() => false);
}

/** Press a menu trigger until its menu is up, and answer with the menu. */
async function openMenu(session, name, attempts = 2) {
  let last;
  for (let attempt = 0; attempt < attempts; attempt += 1) {
    await session.clickNamed("button", name);
    const menu = await until(
      `the ${name} menu`,
      () => session.find("css selector", "[role=menu]"),
      8000,
    ).catch((failure) => {
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
  return session.execute(
    "return [...document.querySelectorAll('[data-sonner-toast]')].map((toast) => toast.innerText).join(' | ');",
  );
}

/** The sidebar's case rows as one string, which is where status lives. */
function sidebarText(session) {
  return session.execute(
    "const nav = document.querySelector('nav[aria-label=Navigation]'); return nav ? nav.innerText : '';",
  );
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
  for (const empty of [
    "Nothing is waiting on you.",
    "No cycle is running.",
    "No cycles have been loaded yet.",
  ]) {
    assert.ok(hasText(body, empty), `An empty group must state "${empty}"`);
  }
  assert.ok(await session.exists("button", "New Cycle"), "Cycles home must offer New Cycle");
  assert.ok(
    await session.find("css selector", "nav[aria-label=Navigation]"),
    "The shell must render the sidebar",
  );
}

/**
 * Library: the only route by which cases enter a cycle.
 *
 * One suite expanded, one template configured, the editor exercised on a number
 * and on a list, the summary read, and the cases materialized twice into the
 * same cycle -- which is what `appendTo` in the URL is for.
 */
async function libraryJourney(session) {
  await session.goto("/library");
  await until("suite rows", async () => hasText(await session.text(), PROTECTION_SUITE), 60000);

  await session.clickNamed("button", PROTECTION_SUITE);
  await until("expanded suite", async () => hasText(await session.text(), "Configure"), 30000);
  assert.ok(
    hasText(await session.text(), "Template"),
    "An expanded suite must state its template count",
  );

  /* The row itself opens the dialog; Configure is the same outcome named. */
  await session.clickNamed("button", "Configure");
  await until("template dialog", () => session.find("css selector", "[role=dialog]"), 30000);
  const step1 = await session.text();
  assert.ok(hasText(step1, "Parameters"), "Step 1 must be Parameters");
  assert.ok(hasText(step1, "Summary"), "The step indicator must name Summary");
  assert.ok(
    hasText(step1, "Values you change are remembered for this template."),
    "Step 1 must state that edits persist",
  );
  await until("parameter grid", () =>
    session.find("css selector", "[data-testid=parameter-editor-grid]"),
  );
  assert.ok(await session.exists("button", "Reset All"), "The parameter step must carry Reset All");

  /* A number edit reveals that row's Reset affordance and its gilt rule. */
  const number = await session.find(
    "css selector",
    `[data-testid="parameter-input-${NUMBER_PARAMETER}"]`,
  );
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
  await session.clickCss(
    `[data-testid="parameter-chips-${LIST_PARAMETER}"] button[aria-label^="Add "]`,
  );
  await until(
    "added chip",
    async () => ((await chips()) ?? []).length === before.length + 1,
    15000,
  );
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
  const reference = await until("expression reference", () =>
    session.find("css selector", "[data-testid=expression-reference]"),
  );
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
  assert.ok(
    hasText(summary, "New Cycle"),
    "Without appendTo the summary must name the cycle it will create",
  );
  assert.ok(
    hasText(summary, "Created On Add"),
    'The target must read "New Cycle -- Created On Add"',
  );
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
  const appendTo = await until(
    "appendTo in the URL",
    async () => {
      const location = await session.location();
      const id = location.searchParams.get("appendTo");
      return location.pathname === "/library" && id ? id : null;
    },
    60000,
  );
  assert.equal(
    await session.execute("return document.querySelectorAll('[role=dialog]').length"),
    0,
    "Finish And Add must close the dialog",
  );
  assert.ok(
    hasText(await session.text(), "Cases materialized here are added to"),
    "With a target cycle the Library must name it",
  );
  assert.ok(
    await session.exists("button", "Open Cycle"),
    "With a target cycle the page bar action must become Open Cycle",
  );

  /* A second template joins the same cycle, so the run has a prompt in it. */
  await settle(session);
  await session.clickNamed("button", STARTUP_SUITE);
  await until(
    "startup template",
    async () => hasText(await session.text(), STARTUP_TEMPLATE),
    30000,
  );
  await session.clickCss('[data-testid="suite-template-mini.charger.startup.sequence"] button');
  await until("second dialog", () => session.find("css selector", "[role=dialog]"), 30000);
  await session.clickNamed("button", "Next");
  await until("second summary", async () => hasText(await session.text(), "Finish And Add"), 60000);
  assert.ok(
    !hasText(await session.text(), "Created On Add"),
    "With appendTo the summary must name the target cycle, not a new one",
  );
  await session.clickNamed("button", "Finish And Add");
  await until(
    "second add finished",
    async () =>
      (await session.execute("return document.querySelectorAll('[role=dialog]').length")) === 0,
    60000,
  );

  await session.clickNamed("button", "Open Cycle");
  await until(
    "cycle workspace",
    async () => (await session.location()).pathname === `/cycles/${appendTo}`,
    30000,
  );
  return appendTo;
}

/**
 * The cycle workspace: run it, answer it, then read it.
 *
 * One layout throughout: the sidebar owns selection, the header owns every
 * whole-cycle command, and the evidence pane shows what the URL selects.
 */
async function cycleJourney(session, cycleId) {
  await session.goto(`/cycles/${cycleId}`);
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
      `No case of a cycle that has never run reads as planned or queued work. The rows read ${JSON.stringify(
        [
          ...new Set(
            normalize(sidebar).match(/Planned|Queued|Not Run|Skipped|Passed|Failed|Running/g) ?? [],
          ),
        ],
      )}.`,
    );
  }

  const header = await session.text();
  assert.ok(/\d+ of \d+ cases finished/.test(normalize(header)), "The header must state progress");
  assert.ok(hasText(header, "Ready"), "A materialized cycle must render its Ready status");
  assert.ok(
    await session.exists("button", "Cycle Actions"),
    "The header must carry the overflow menu",
  );
  /* Run Queued is the redesign's promise on a Ready cycle. Two things can
     deny it: a header that does not offer it, and a backend that refuses the
     admission. Either way the cases still have to run for everything behind
     them to be covered, so the gate states what happened and starts the work
     from the row menus instead. */
  let admitted = false;
  if (await session.exists("button", "Run Queued")) {
    await session.clickNamed("button", "Run Queued");
    /* The admission's own answer, before anything waits on its effect: the
       workspace toasts every command's outcome with the backend's own text,
       and a refused run is a refusal to state here rather than five silent
       minutes later. */
    const answer = await until(
      "the run admission's answer",
      async () => (await toastText(session)) || null,
      45000,
    ).catch(() => "");
    /* Silence is not an admission. A command that announced nothing at all is
       the failure this gate exists to catch, so it takes the same route as a
       stated refusal rather than waiting five minutes on an effect nothing
       promised. */
    admitted = /queued cases started/i.test(answer);
    if (!admitted) {
      defect(
        "cycle/run-queued",
        `Run Queued did not start the cycle. The application said: ${
          normalize(answer) || "nothing at all"
        }. Started the cases from their row menus instead.`,
      );
    }
  } else {
    defect(
      "cycle/run-queued",
      `Run Queued is absent on a Ready cycle whose header reads "${
        /(\d+ of \d+ cases finished)/.exec(normalize(header))?.[1] ?? "n of n cases finished"
      }". Started the cases from their row menus instead.`,
    );
  }

  if (!admitted) {
    /* One measuring case for the evidence and the report, and the prompting
       case for the banner: enough to cover everything behind the run without
       paying for five. */
    const started = [];
    for (const fragment of [PROTECTION_TEMPLATE, STARTUP_TEMPLATE]) {
      if (await startCaseFromItsRow(session, fragment)) started.push(fragment);
    }
    assert.ok(
      started.length > 0,
      "Run Case on a row must start a case when Run Queued does not run one",
    );
    if (started.length < 2) {
      defect(
        "cycle/run-case",
        `Run Case started only ${JSON.stringify(started)}; the other case stayed where it was.`,
      );
    }
  }

  await runUntil(
    session,
    "cycle completion",
    async () => {
      const rows = await sidebarText(session);
      if (/\b(Running|Waiting|Finalizing)\b/.test(rows)) return false;
      return /\b(Passed|Failed|Blocked)\b/.test(rows);
    },
    300000,
  );

  /* Selecting a case puts the case and its latest attempt in the URL. */
  await session.execute(`
    const rows = [
      ...document.querySelectorAll('nav[aria-label=Navigation] li[data-level=case]'),
    ].filter((row) => /Passed|Failed|Blocked/.test(row.innerText));
    if (!rows.length) throw new Error('No case row has a terminal status');
    rows[0].querySelector('button').click();
  `);
  const selection = await until(
    "case selected in the URL",
    async () => {
      const location = await session.location();
      const caseId = location.searchParams.get("caseId");
      return caseId ? { caseId, executionId: location.searchParams.get("executionId") } : null;
    },
    30000,
  );
  assert.ok(selection.executionId, "Selecting a case must select its latest attempt");
  /* The evidence toolbar sets the attempt label in small caps, so what
     `innerText` reports is the rendered "ATTEMPT 1 OF 1". The copy is what
     matters here, not the type treatment. */
  const evidence = await until(
    "attempt evidence",
    async () => {
      const text = await session.text();
      return /Attempt \d+ Of \d+/i.test(normalize(text)) ? text : null;
    },
    60000,
  );
  assert.match(normalize(evidence), /Attempt \d+ Of \d+/i);
  assert.ok(
    await session.exists("button", "Choose Attempt"),
    "An attempt must be choosable from the toolbar",
  );

  /* Discovery And Computation Cross The Real Native Plot Routes. */
  const analysisSummary = await until("analysis controls", () =>
    session.find("xpath", "//summary[normalize-space(.)='Evidence Analysis']"),
  );
  await session.click(analysisSummary);
  await until("native mean analysis discovery", () =>
    session.execute(`
    const label = [...document.querySelectorAll('label')].find(node => node.textContent === 'Analysis');
    const select = label && document.getElementById(label.htmlFor);
    return select && !select.disabled && [...select.options].some(option => option.value === 'dokime.analysis.mean');
  `),
  );
  await session.execute(`
    const label = [...document.querySelectorAll('label')].find(node => node.textContent === 'Analysis');
    const select = document.getElementById(label.htmlFor);
    Object.getOwnPropertyDescriptor(HTMLSelectElement.prototype, 'value').set.call(select, 'dokime.analysis.mean');
    select.dispatchEvent(new Event('change', {bubbles: true}));
  `);
  await session.clickNamed("button", "Apply Analysis");
  const computed = await until(
    "native scalar analysis result",
    () =>
      session.execute(`
    const heading = [...document.querySelectorAll('h3')].find(node => node.textContent.includes(' · Mean'));
    const result = heading?.closest('section');
    const value = result?.querySelector('dd')?.textContent;
    const provenance = result?.textContent.match(/([0-9]+) source samples · Version 1/);
    return value && provenance ? {value: parseFloat(value), samples: Number(provenance[1])} : null;
  `),
    30000,
  );
  assert.ok(Number.isFinite(computed.value), "The native mean must render a finite scalar");
  assert.ok(computed.samples > 0, "The analysis must identify its original evidence count");
  await session.execute(`
    const heading = [...document.querySelectorAll('h3')].find(node => node.textContent.includes(' · Mean'));
    heading.closest('section').scrollIntoView({block: 'center'});
  `);
  await session.screenshot("analysis-result");
  await session.execute(`
    const label = [...document.querySelectorAll('label')].find(node => node.textContent === 'Analysis');
    const select = document.getElementById(label.htmlFor);
    Object.getOwnPropertyDescriptor(HTMLSelectElement.prototype, 'value').set.call(select, '');
    select.dispatchEvent(new Event('change', {bubbles: true}));
  `);
  await session.clickNamed("button", "Use Original Plot");

  /* Focus is a URL flag, so it survives a reload and can be linked. The router
     serializes the flag as `focus=true`; `focusSearchSchema` also accepts the
     `focus=1` a person would type, so both spellings are the flag. */
  await session.clickNamed("button", "Focus");
  await until(
    "focus flag set",
    async () => ["1", "true"].includes((await session.location()).searchParams.get("focus")),
    20000,
  );
  assert.ok(await session.exists("button", "Leave Focus"), "Focus view must say how to leave it");
  await session.clickNamed("button", "Leave Focus");
  await until(
    "focus flag cleared",
    async () => (await session.location()).searchParams.get("focus") !== "1",
    20000,
  );

  /* The console: scopes follow the route, Escape closes the drawer. */
  await session.clickNamed("button", "Open Console");
  const drawer = await until("console drawer", () =>
    session.find("css selector", "section[aria-label=Console]"),
  );
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
  const levels = await session.execute(
    "return [...document.querySelectorAll('[role=option]')].map((item) => item.innerText.trim());",
  );
  for (const level of ["Trace", "Info", "Error"]) {
    assert.ok(levels.includes(level), `The level selector must offer ${level} (got ${levels})`);
  }
  await session.key("Escape");
  await delay(400);
  await session.key("Escape");
  await until(
    "console closed by Escape",
    async () =>
      (await session.execute(
        "return document.querySelectorAll('section[aria-label=Console]').length",
      )) === 0,
    20000,
  );
  /* A modal layer that closed must give the page back. Stating it here names
     the cause; without it the next control simply never responds. */
  assert.equal(
    await session.execute("return document.body.style.pointerEvents || '';"),
    "",
    "A closed console must leave the page interactive",
  );

  /* The overflow menu names every whole-cycle command, and Clear Cycle asks.
     The trigger is pressed again if the first press produced no menu: a real
     WebView occasionally drops the pointer sequence a dropdown opens on, and
     one silent miss is not a product defect. A second miss is, and fails. */
  const menu = await openMenu(session, "Cycle Actions");
  const items = await session.textOf(menu);
  for (const item of [
    "Copy Cycle",
    "Add Cases",
    "Runtime Policy",
    "Follow Active",
    "Download Bundle",
    "Write Report",
    "Clear Cycle",
  ]) {
    assert.ok(hasText(items, item), `The overflow menu must offer ${item}`);
  }
  await session.clickNamed("menuitem", "Clear Cycle");
  const confirm = await until("clear confirm", () => session.find("css selector", "[role=dialog]"));
  assert.ok(
    hasText(await session.textOf(confirm), "Clearing stops this cycle"),
    "Clear Cycle must state what it removes",
  );
  await session.clickNamed("button", "Cancel");
  /* Cancelling must leave the cycle where it was and give the page back.
     Radix keeps a dismissed dialog mounted while its exit animation runs, and
     in this WebView that node can outlive the animation it waits on — so the
     open layers are what this counts, and the page's own interactivity is the
     promise that actually matters. Whatever stays behind is written into the
     evidence, because a bare timeout names the condition and not the cause. */
  await until(
    "confirm cancelled",
    async () => {
      const layers = await session.modalLayers();
      return (
        layers.pathname === `/cycles/${cycleId}` &&
        layers.dialogs.every((dialog) => dialog.state !== "open") &&
        layers.bodyPointerEvents === ""
      );
    },
    20000,
  ).catch(async (failure) => {
    const target = await session.dump("cycle-confirm-cancelled", {
      expectedPathname: `/cycles/${cycleId}`,
      layers: await session.modalLayers(),
    });
    throw new Error(`${failure.message} (layers written to ${target})`);
  });
  /* A dismissed dialog that stays in the document is the WebView not
     delivering the animation end its exit waits on. It holds nothing — the
     page is interactive and the layer is closed — but it is why counting
     `[role=dialog]` nodes rather than open ones made this step time out, so
     each run records whether it happened. */
  const dismissed = await session.modalLayers();
  if (dismissed.dialogs.length) {
    notes.push(
      `cycle: cancelling Clear Cycle left ${dismissed.dialogs.length} dismissed dialog node(s) in the document (${dismissed.dialogs
        .map((dialog) => `${dialog.slot || "dialog"}:${dialog.state}`)
        .join(", ")})`,
    );
  }
  return selection;
}

/** Cycles home after the run: the Recent row, Show Cleared, and the filter. */
async function cyclesHomeJourney(session) {
  await session.goto("/cycles");
  const body = await until(
    "a cycle row",
    async () => {
      const text = await session.text();
      return hasText(text, "Recent") && /Completed|Failed|Ready|Stopped/.test(text) ? text : null;
    },
    60000,
  );
  assert.ok(hasText(body, "Attention"), "The group must render its column headings");
  assert.ok(/\d+ \/ \d+/.test(normalize(body)), "A row must state executions over cases");
  assert.ok(/\d+[smhd] Ago/.test(normalize(body)), "A row must state when the cycle last moved");

  await session.clickNamed("button", "Show Cleared");
  assert.ok(
    await until("hide cleared", () => session.exists("button", "Hide Cleared")),
    "Show Cleared must become Hide Cleared",
  );
  await session.clickNamed("button", "Hide Cleared");

  const filter = await session.find("css selector", "input[aria-label='Filter Cycles']");
  await session.type(filter, "zzz-no-such-cycle");
  await until(
    "filter excludes every row",
    async () => hasText(await session.text(), "No cycles match this filter."),
    20000,
  );
  await session.type(filter, "");
  await until(
    "filter cleared",
    async () => !hasText(await session.text(), "No cycles match this filter."),
    20000,
  );
  assert.ok(
    await session.execute(
      `return [...document.querySelectorAll('button')].some((button) =>
         button.innerText.includes(arguments[0]),
       );`,
      [PROTECTION_SUITE],
    ),
    "The run's cycle must be one of the rows",
  );
}

/** History: the recorded executions behind the run, and the Back chevron. */
async function historyJourney(session) {
  await session.goto("/history");
  await until(
    "history rows",
    async () => {
      const rows = await session.findAll("css selector", "tbody tr");
      return rows.length ? rows : null;
    },
    120000,
  );
  assert.ok(hasText(await session.text(), "History"), "The page bar must name History");
  const rows = await session.findAll("css selector", "tbody tr");
  await session.click(rows[0]);
  await until(
    "history detail",
    async () => (await session.location()).pathname.startsWith("/history/"),
    30000,
  );
  await until(
    "inspectable checkpoint",
    async () => hasText(await session.text(), "Inspect checkpoint"),
    60000,
  );
  /* Each case also carries a pre-dispatch placeholder execution with no
     evidence behind it, so the checkpoint to inspect is a finished one. */
  await session.execute(`
    /* The outcome sits beside the button on the entry's own row rather than in
       a landmark of its own, so the search walks up from each button until the
       nearest ancestor that states an outcome, and never addresses a class.
       The word boundaries are escaped because this is a template literal,
       where an unescaped backslash-b is a backspace character rather than a
       boundary: the regex then silently matched nothing. */
    const stated = (button) => {
      let node = button.parentElement;
      for (let depth = 0; node && depth < 5; depth += 1) {
        if (/\\b(passed|failed)\\b/i.test(node.innerText)) return true;
        node = node.parentElement;
      }
      return false;
    };
    const inspect = [...document.querySelectorAll('main button')]
      .filter((button) => button.innerText.trim() === 'Inspect checkpoint' && !button.disabled)
      .find(stated);
    if (!inspect) throw new Error('No finished execution in this history detail');
    inspect.click();
  `);
  await until(
    "captured execution",
    async () => hasText(await session.text(), "Recorded Steps"),
    60000,
  );
  const captured = await session.text();
  assert.ok(hasText(captured, "Recorded status"), "The checkpoint must state its recorded status");
  for (const view of ["Records", "Telemetry", "Files", "Context"]) {
    assert.ok(await session.exists("button", view), `The checkpoint must offer the ${view} view`);
  }
  await session.clickNamed("button", "Records");
  await until(
    "captured records",
    async () => {
      const section = await session
        .find("css selector", "section[aria-label='Captured records']")
        .catch(() => null);
      if (!section) return false;
      return !hasText(await session.textOf(section), "No execution records at this checkpoint.");
    },
    60000,
  );
  await session.clickNamed("button", "Context");
  await until("captured context", async () => hasText(await session.text(), "Notes At Capture"));

  await session.clickNamed("button", "Back");
  await until(
    "back to history",
    async () => (await session.location()).pathname === "/history",
    20000,
  );
}

/** Settings: every section, and the rows the redesign promises in each. */
async function settingsJourney(session, notes, work) {
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
    const rows = await session.execute(
      "return document.querySelectorAll('main section, main dl > div').length",
    );
    assert.ok(rows > 0, `The ${id} section must render rows`);
  }

  /* Appearance: the density radios autosave, with no Save button anywhere. */
  await session.goto("/settings/appearance");
  await until("appearance", async () => hasText(await session.text(), "Density"), 60000);
  assert.ok(
    !(await session.exists("button", "Save")),
    "Settings autosave; no section may carry a Save button",
  );
  await session.clickNamed("radio", "Touch");
  await until(
    "touch density applied",
    async () =>
      (await session.execute("return document.documentElement.dataset.density")) === "touch",
    20000,
  );
  await session.clickNamed("radio", "Comfortable");
  await until(
    "comfortable density applied",
    async () =>
      (await session.execute("return document.documentElement.dataset.density")) === "comfortable",
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
      `const label = [...document.querySelectorAll('label')].find(label => label.textContent.trim() === 'Save Interval (Seconds)');
     return label?.control ?? null;`,
    ),
  );
  await session.type(interval, "1.234");
  await until(
    "backend policy saved",
    async () => {
      try {
        const saved = JSON.parse(await readFile(join(work, "data/backend-policy.json"), "utf8"));
        return saved.revision > 0 && saved.policy.saveIntervalMs === 1234;
      } catch {
        return false;
      }
    },
    20000,
  );
  await until(
    "policy restart requirement",
    async () => hasText(await session.text(), "Restart the connected backend"),
    20000,
  );
  await session.goto("/settings/appearance");
  await session.goto("/settings/general");
  await until("saved backend policy reread", () =>
    session.execute(
      `const label = [...document.querySelectorAll('label')].find(label => label.textContent.trim() === 'Save Interval (Seconds)');
     return label?.control?.value === '1.234';`,
    ),
  );

  /* Plugins: rows with a Settings button, and the plugin page behind it. */
  await session.goto("/settings/plugins");
  await until(
    "the plugins section",
    async () => hasText(await session.text(), "Install A Plugin"),
    60000,
  );
  const rows = await session.execute(
    `return document.querySelectorAll('button[aria-label="Plugin Settings"]').length`,
  );
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
      hasText(
        await session.text(),
        "Loaded from the launch configuration; managed outside the installation store.",
      ),
      "With no installed plugin the Plugins section must still name the loaded ones",
    );
    notes.push(
      "settings: the gate launches the backend with --plugin, so the installation store is empty and the plugin row's Settings button is not on this path",
    );
  }
  if (configurable) {
    await session.clickNamed("button", "Plugin Settings");
    await until(
      "plugin page",
      async () => (await session.location()).pathname.startsWith("/settings/plugins/"),
      30000,
    );
    assert.ok(
      hasText(await session.text(), "Plugins"),
      "The plugin page must carry its breadcrumb",
    );
    await session.clickNamed("button", "Back");
    await until(
      "back to plugins",
      async () => (await session.location()).pathname === "/settings/plugins",
      20000,
    );
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
      const listed = await readdir(session.downloads);
      return listed.find((name) => /^dokime-diagnostics-.*\.zip$/.test(name)) ?? null;
    },
    120000,
  ).catch((failure) => {
    throw new Error(
      `${failure.message}. The application said: ${announcement || "nothing at all"}`,
    );
  });
  assert.match(bundle, /^dokime-diagnostics-.*\.zip$/);
}

/**
 * The frozen report: written by the application, then read back in this WebView.
 *
 * The same component set renders it with `live={false}`, so the assertions are
 * about what a file can honour: identity, provenance, the outline and the
 * records -- and the absence of every control that would lie.
 */
async function reportJourney(session, cycleId) {
  await session.goto(`/cycles/${cycleId}`);
  await until("cycle workspace", () => session.exists("button", "Cycle Actions"), 60000);
  await installSaveTap(session);
  await session.clickNamed("button", "Cycle Actions");
  await session.clickNamed("menuitem", "Write Report");
  /* The toast carries the backend's own words when generation fails, and a
     report that is never written is otherwise five silent minutes. */
  let announcement = "";
  const written = await until(
    "report written",
    async () => {
      announcement = (await toastText(session)) || announcement;
      const saved = await drainSavedDocuments(session);
      return saved.find((entry) => entry.path.endsWith(".html")) ?? null;
    },
    300000,
  ).catch((failure) => {
    throw new Error(
      `${failure.message}. The application said: ${announcement || "nothing at all"}`,
    );
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
    const chrome = process.env.DOKIME_E2E_CHROME ?? "google-chrome";
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
  assert.ok(
    /Frozen Report .{0,3} Generated/.test(normalize(report)),
    "The report must state when it was generated",
  );
  assert.ok(!/Run Queued|Pause|Stop\b/.test(report), "A frozen report renders no run controls");
  assert.ok(hasText(report, "Records"), "The report renders its records section");
}

/**
 * Touch density: the same surfaces, driven through the real Appearance switch.
 *
 * At touch the sidebar is a rail, so the cycle's cases are only reachable
 * through the header's Cases sheet, and the template dialog becomes a
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

  await session.goto(`/cycles/${cycleId}`);
  await until("touch workspace", () => session.exists("button", "Cycle Actions"), 60000);
  /* The rail: collapsed at touch, with the children behind it. */
  if (await session.exists("button", "Collapse Sidebar")) {
    await session.clickNamed("button", "Collapse Sidebar");
  }
  await until("collapsed rail", () => session.exists("button", "Expand Sidebar"), 20000);
  const railChildren = await session.execute(
    "return document.querySelectorAll('nav[aria-label=Navigation] li[data-level=case]').length;",
  );
  assert.equal(railChildren, 0, "A collapsed rail must hide its children");

  await session.clickNamed("button", "Cases");
  const sheet = await until("cases sheet", () =>
    session.find("css selector", "[data-slot=sheet-content]"),
  );
  const sheetText = await session.textOf(sheet);
  assert.ok(hasText(sheetText, "Cases"), "The Cases sheet must name itself");
  assert.ok(
    hasText(sheetText, PROTECTION_TEMPLATE) || hasText(sheetText, STARTUP_TEMPLATE),
    "The Cases sheet must hold the cycle's case rows",
  );
  await session.key("Escape");
  await delay(500);

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
  assert.ok(
    share >= 80,
    `At touch the dialog must be a full-width sheet (measured ${share}% of the viewport)`,
  );
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
  if (selectedJourneys.size && !selectedJourneys.has(name) && !REQUIRED_JOURNEYS.has(name)) {
    process.stdout.write(`\n== ${name}\n   skipped by DOKIME_E2E_JOURNEYS\n`);
    return { ok: true, skipped: true };
  }
  process.stdout.write(`\n== ${name}\n`);
  if (webviewGone) {
    const reason = `skipped: the WebView process exited during the ${webviewGone} journey`;
    failures.push({ kind: "failure", name, reason });
    process.stdout.write(`   ${reason}\n`);
    return { ok: false };
  }
  try {
    const value = await run();
    passedJourneys.push(name);
    process.stdout.write("   passed\n");
    return { ok: true, value };
  } catch (failure) {
    /* A dead session is one crash, not one failure per journey after it: the
       journeys behind it never ran, and saying they failed would bury the
       crash under its own consequences. */
    if (/invalid session id|Session terminated without a reply/.test(failure.message)) {
      webviewGone = name;
    }
    failures.push({ kind: "failure", name, reason: failure.message });
    process.stdout.write(`   FAILED: ${failure.message}\n`);
    if (session && !session.closed) {
      try {
        const shot = await session.screenshot(`failure-${name}`);
        await writeFile(
          join(evidence, `failure-${name}.txt`),
          await session.text().catch(() => session.lastBody ?? ""),
        );
        /* What the application asked the shell to do, and how long each call
           took: a journey that waits on something that never answers says so
           here and nowhere else. */
        await writeFile(
          join(evidence, `invokes-${name}.json`),
          JSON.stringify(await drainInvokeLedger(session), null, 2),
        );
        process.stdout.write(`   evidence: ${shot}\n`);
      } catch {
        // A dead WebView has no DOM left to photograph.
      }
    }
    return { ok: false };
  }
}

function terminate(pid, signal = "SIGTERM") {
  if (!pid) return;
  try {
    process.kill(-pid, signal);
  } catch {
    // The process group already exited.
  }
}

async function main() {
  if (process.platform !== "linux") {
    throw new Error("This gate requires Linux WebKitWebDriver.");
  }
  if (!process.env.DISPLAY && !process.env.WAYLAND_DISPLAY) {
    throw new Error("Driving the native WebView needs a running display session.");
  }
  for (const [label, path] of [
    ["the backend", backendBinary],
    ["the desktop shell", shellBinary],
    ["the Mini charger plugin bundle", pluginBundle],
  ]) {
    if (!existsSync(path))
      throw new Error(`Missing ${label} at ${path}. Run "mise run build:dev".`);
  }

  await mkdir(targetRoot, { recursive: true });
  evidence = await mkdtemp(join(targetRoot, "native-desktop-e2e-"));
  work = await mkdtemp(join(targetRoot, "native-desktop-work-"));
  process.stdout.write(`Evidence: ${evidence}\n`);

  backend = await startBackend(work, evidence);
  session = await startSession(work, backend.endpoint, evidence);
  await sizeViewport(session, densities.desktop);

  await journey("startup", () => startupJourney(session, notes));
  await installSaveTap(session);

  /* The Library is where cases come from, so it runs whatever the home said:
     a wrong count on Cycles is not a reason to certify nothing behind it. */
  const library = await journey("library", () => libraryJourney(session));
  const cycleId = library.value;

  if (cycleId) {
    await journey("cycle", () => cycleJourney(session, cycleId));
    await journey("cycles-home", () => cyclesHomeJourney(session));
    await journey("history", () => historyJourney(session));
  } else {
    for (const name of ["cycle", "cycles-home", "history"]) {
      failures.push({
        kind: "failure",
        name,
        reason: "skipped: the library journey produced no cycle",
      });
    }
  }

  await journey("settings", () => settingsJourney(session, notes, work));

  if (cycleId) {
    await journey("report", () => reportJourney(session, cycleId));
    await journey("touch", () => touchJourney(session, cycleId));
  } else {
    for (const name of ["report", "touch"]) {
      failures.push({
        kind: "failure",
        name,
        reason: "skipped: the library journey produced no cycle",
      });
    }
  }

  /* The ownership contract the redesign did not change: a shell attached to a
     backend it does not own must leave that backend running. */
  await journey("attachment", async () => {
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
  });
}

/* === Entry Point === */

try {
  await main();
} catch (failure) {
  failures.push({ kind: "failure", name: "harness", reason: failure.message });
  process.stdout.write(`\nHarness failure: ${failure.message}\n`);
} finally {
  if (session) {
    await session.close().catch(() => undefined);
    await session.request("DELETE", "").catch(() => undefined);
    terminate(session.process.pid);
  }
  if (backend) terminate(backend.child.pid);
  await delay(1500);
  if (session) terminate(session.process.pid, "SIGKILL");
  if (backend) terminate(backend.child.pid, "SIGKILL");
  if (work) await rm(work, { force: true, recursive: true }).catch(() => undefined);
}

for (const note of notes) process.stdout.write(`\nnote: ${note}\n`);

if (failures.length) {
  /* A defect is the product refusing what the redesign promised; a failure is
     this gate not getting through. Both are non-zero, and they are not the
     same report. */
  for (const kind of ["defect", "failure"]) {
    const group = failures.filter((entry) => entry.kind === kind);
    if (!group.length) continue;
    process.stdout.write(
      `\n${group.length} ${kind === "defect" ? "product defect(s)" : "journey failure(s)"}:\n`,
    );
    for (const entry of group) process.stdout.write(`  - ${entry.name}: ${entry.reason}\n`);
  }
  process.stdout.write(`\nEvidence: ${evidence}\n`);
  process.exitCode = 1;
} else {
  process.stdout.write(`\nNative WebView: ${passedJourneys.join(", ")} passed.\n`);
  process.stdout.write(`Evidence: ${evidence}\n`);
  if (!process.env.DOKIME_E2E_KEEP) await rm(evidence, { force: true, recursive: true });
}
