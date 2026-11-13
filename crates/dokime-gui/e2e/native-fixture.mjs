// Dokime — SPDX-License-Identifier: Apache-2.0
import assert from "node:assert/strict";

import { execFile, spawn } from "node:child_process";
import { once } from "node:events";

import { existsSync } from "node:fs";

import { appendFile, cp, mkdir, mkdtemp, open, readdir, readFile, realpath, rm, writeFile } from "node:fs/promises";

import net from "node:net";
import { tmpdir } from "node:os";

import { basename, dirname, join, resolve } from "node:path";

import { createInterface } from "node:readline";

import { setTimeout as delay } from "node:timers/promises";

import { fileURLToPath } from "node:url";

import { parseArgs, promisify } from "node:util";

// Third Party Imports

// Local Imports
import { ReportBrowser } from "./report-browser.ts";
import { traceSpan } from "./test-trace.mjs";

/* === Constants === */
const workspace = resolve(dirname(fileURLToPath(import.meta.url)), "../../..");
const WAIT_MS = 240000;
const STARTUP_WAIT_MS = 360000;
const options = parseArgs({
  options: {
    profile: { type: "string", default: "dev" },
    junit: { type: "string" },
    workflow: { type: "string" },
    release: { type: "boolean" },
    scene: { type: "string" },
  },
}).values;
const profile = options.profile;
const nativeTarget = (process.platform === "win32" ? "windows" : "linux") + "-" + (process.arch === "x64" ? "x86_64" : "aarch64");
const built = join(workspace, "target/build", profile, nativeTarget);
const executableSuffix = process.platform === "win32" ? ".exe" : "";
const backendBinary = process.env.DOKIME_E2E_CLI ?? join(built, "fixtures/desktop/dokime-next" + executableSuffix);
const shellBinary = process.env.DOKIME_E2E_SHELL ?? join(built, "bin/dokime-gui" + executableSuffix);
const cliBinary = backendBinary;
const pluginExecutable = join(built, "fixtures/host/dokime-rust-thermal" + executableSuffix);
const credentialHost =
  process.env.DOKIME_HOST_FIXTURE ?? join(built, (profile === "bench" ? "bin/" : "fixtures/host/") + "dokime-host-next" + executableSuffix);
const windows = process.platform === "win32";
const syntheticKeys = process.env.DOKIME_E2E_SYNTHETIC_KEYS === "1";
const WEBDRIVER_KEYS = { Enter: "\uE007", Escape: "\uE00C", Tab: "\uE004", Control: "\uE009", Backspace: "\uE003", End: "\uE010" };
const STARTUP_PROMPT = "Is the Door Closed?";
const OPERATOR_NOTE = "Desktop Story";
const timings = [],
  promptSightings = [],
  cycleInputSightings = [];
const secrets = new Set();
let timingScene = "setup",
  work,
  evidence,
  reportBrowser,
  nativePickerWorker,
  driverProcess,
  backend,
  nativeScratch,
  secretBus,
  secretKeyring;
let scratchEnvironment = {};
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
    socket.setTimeout(WAIT_MS, () => {
      socket.destroy();
      done(false);
    });
  });
}

async function executeFile(command, args, options) {
  const category = command === cliBinary ? "cli" : command === "python" ? "fixture-preparation" : "native-process";
  return measured(category, basename(command), () =>
    promisify(execFile)(command, args, { ...options, env: { ...process.env, ...options?.env, ...scratchEnvironment } }),
  );
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

function until(description, check, timeout = WAIT_MS, interval = 25) {
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

async function startBackend(...args) {
  return measured("backend-launch", "startBackend", () => startBackendMeasured(...args));
}

async function startBackendMeasured(
  work,
  evidence,
  endpoint,
  label = "backend",
  launchPlugin = true,
  { dataRoot = join(work, "data"), environment = {} } = {},
) {
  for (const path of [backendBinary, cliBinary, credentialHost, ...(launchPlugin ? [pluginExecutable] : [])]) {
    if (!existsSync(path)) throw new Error(`Missing backend artifact at ${path}. Use the same candidate's prepared artifacts.`);
  }
  const port = endpoint ? Number(new URL(endpoint).port) : await unusedPort();
  const log = await open(join(evidence, `${label}.log`), "w");
  const child = spawn(backendBinary, ["server", "--endpoint", `127.0.0.1:${port}`, "--storage", dataRoot], {
    cwd: work,
    detached: !windows,
    windowsHide: true,
    env: {
      ...process.env,
      DOKIME_TEST_HUB_OPERATOR: "Gate Operator",
      ...environment,
      ...scratchEnvironment,
      ...(!windows ? { HOME: work } : {}),
    },
    stdio: ["pipe", log.fd, log.fd],
  });
  child.on("error", (error) => {
    child.startupError = error;
  });
  backend = { child, endpoint: `http://127.0.0.1:${port}`, log, root: dataRoot };
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
      STARTUP_WAIT_MS,
    );
  } finally {
    await writeFile(join(evidence, `${label}-readiness.json`), JSON.stringify({ ...readiness, elapsedMs: Date.now() - readyStarted }));
  }
  if (launchPlugin) {
    const plugin = spawn(pluginExecutable, ["--remote"], {
      cwd: work,
      env: { ...process.env, ...scratchEnvironment, HARMOS_ADDRESS: "127.0.0.1:0" },
      windowsHide: true,
      detached: !windows,
      stdio: ["ignore", "pipe", "inherit"],
    });
    backend.plugin = plugin;
    const lines = createInterface({ input: plugin.stdout });
    const announced = await Promise.race([
      once(lines, "line").then(([line]) => line),
      once(plugin, "exit").then(([code]) => {
        throw new Error(`Start SDK Fixture: Plugin Exited with ${code}`);
      }),
    ]);
    assert.ok(announced.startsWith("harmos listening "), "SDK fixture announces its bound endpoint");
    lines.on("line", () => {});
    await executeFile(
      cliBinary,
      ["--endpoint", backend.endpoint, "plugin", "add", `http://${announced.slice("harmos listening ".length)}`],
      { windowsHide: true, timeout: WAIT_MS },
    );
  }
  return backend;
}

async function startSession(...args) {
  return measured("gui-launch", "startSession", () => startSessionMeasured(...args));
}

async function startSessionMeasured(work, endpoint, evidence, distribution, { startLocal = true, downloadsRoot } = {}) {
  const home = join(work, "shell");
  const downloads = downloadsRoot ?? join(evidence, "downloads");
  const applicationRoot = distribution ? join(home, "data", distribution.identifier) : join(home, "application");
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
        ...scratchEnvironment,
        TAURI_WEBVIEW_AUTOMATION: "true",
        DOKIME_AUTOMATION_REPORT_OPENER: reportBrowser.endpoint,
        DOKIME_APPLICATION_ROOT: distribution ? "" : applicationRoot,
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
      signal: AbortSignal.timeout(WAIT_MS),
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
  session.waitForDom = (description, script, args = [], maximumWait) =>
    measured("dom-wait", description, async () => {
      const result = await session.executeAsync(
        `
      const args = arguments[0];
      const maximumWait = arguments[1];
      const done = arguments[arguments.length - 1];
      const ready = () => {${script}};
      const observer = new MutationObserver(check);
      let timer;
      let finished = false;
      function finish(result) {
        if (finished) return;
        finished = true;
        observer.disconnect();
        clearTimeout(timer);
        done(result);
      }
      function check() {
        try {
          const value = ready();
          if (!value) return;
          finish({value});
        } catch (error) {
          finish({failure: String(error)});
        }
      }
      observer.observe(document, {attributes: true, childList: true, characterData: true, subtree: true});
      if (maximumWait != null) timer = setTimeout(() => finish({value: false}), maximumWait);
      check();
    `,
        [args, maximumWait],
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
  session.click = async (element, { unobscured = false } = {}) => {
    const key = element["element-6066-11e4-a52e-4f735466cecf"];
    const settled = await session.executeAsync(
      `const element = arguments[0];
       const unobscured = arguments[1];
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
         if (position === previous && rect.width > 0 && rect.height > 0) {
           const target = document.elementFromPoint(rect.left + rect.width / 2, rect.top + rect.height / 2);
           return done(!unobscured || element.contains(target));
         }
         previous = position;
         requestAnimationFrame(frame);
       }
       requestAnimationFrame(frame);`,
      [element, unobscured],
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
    const result = await until(`stable pointer target ${selector}`, async () => {
      const element = await session.find("css selector", selector);
      try {
        return { value: await session.click(element, { unobscured: true }) };
      } catch (failure) {
        if (/stale element reference|pointer target must remain mounted/i.test(failure.message)) return null;
        return { failure };
      }
    });
    if (result.failure) throw result.failure;
    return result.value;
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
    const numeric = await session.execute("return arguments[0].type === 'number'", [element]);
    await session.click(element);
    await session.request("POST", "/actions", {
      actions: [
        {
          type: "key",
          id: "native-text",
          actions: [
            { type: "keyDown", value: WEBDRIVER_KEYS.Control },
            { type: "keyDown", value: "a" },
            { type: "keyUp", value: "a" },
            { type: "keyUp", value: WEBDRIVER_KEYS.Control },
            { type: "keyDown", value: WEBDRIVER_KEYS.Backspace },
            { type: "keyUp", value: WEBDRIVER_KEYS.Backspace },
          ],
        },
      ],
    });
    await session.request("DELETE", "/actions");
    // Let Native Number Key Events Commit Before Sending The Next Character.
    if (numeric) {
      for (const character of value) await session.key(character);
      return;
    }
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
    until(
      "connected shell",
      () => session.waitForDom("connected navigation", "return Boolean(document.querySelector('nav[aria-label=Navigation]'));", [], 1000),
      STARTUP_WAIT_MS,
    );
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
      WAIT_MS,
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
    await until(`route ${path}`, async () => (await session.location()).pathname === target.pathname, WAIT_MS);
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
        80000,
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
                timeout: WAIT_MS,
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

async function installSaveTap(session) {
  await until("Tauri IPC bridge", () => session.execute("return typeof globalThis.__TAURI_INTERNALS__?.invoke === 'function'"), WAIT_MS);
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

async function setDensity(session, variant) {
  await session.goto("/settings/appearance");
  await until("appearance section", async () => hasText(await session.text(), "Density"), WAIT_MS);
  await selectDensity(session, variant.density);
  await until(
    `density ${variant.density}`,
    async () => (await session.execute("return document.documentElement.dataset.density")) === variant.density,
    WAIT_MS,
  );
  await sizeViewport(session, variant);
}

async function selectDensity(session, density) {
  await session.clickNamed("combobox", "Density");
  const option = density[0].toUpperCase() + density.slice(1);
  await session.clickNamed("option", option);
}

async function settle(session) {
  await until(
    "toast dismissal",
    () =>
      session.execute(`
        const toasts = [...document.querySelectorAll('[data-sonner-toast]')];
        for (const toast of toasts) {
          if (toast.dataset.removed !== 'true') toast.querySelector('[data-close-button]')?.click();
        }
        return toasts.every(toast =>
          toast.dataset.visible === 'false' || getComputedStyle(toast).pointerEvents === 'none' || !toast.getClientRects().length);
      `),
    60000,
  );
}

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
      delay(WAIT_MS, undefined, { signal: timer.signal }).then(() => {
        throw new Error(`Native backend did not drain within its ${WAIT_MS / 1000} second deadline`);
      }),
    ]);
    assert.equal(signal, null, "native backend drains without a forced signal");
    assert.equal(code, 0, "native backend drains successfully");
  } finally {
    timer.abort();
  }
  await owned.log.close();
  await terminate(owned.plugin);
  await discardHostCredentials(owned.root);
  backend = undefined;
}

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
      delay(WAIT_MS, undefined, { signal: timer.signal }).then(() => {
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
    const children = await readFile(`/proc/${owner.pid}/task/${owner.pid}/children`, "utf8");
    const executable = await realpath(shellBinary);
    for (const pid of children.trim().split(/\s+/)) {
      if (!pid || (await realpath(`/proc/${pid}/exe`)) !== executable) continue;
      const run = (args) => executeFile("xdotool", args, { windowsHide: true, timeout: WAIT_MS });
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
        try:
            child = node.get_child_at_index(index)
        except gi.repository.GLib.GError:
            continue
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
      const result = await executeFile("/usr/bin/python3", ["-c", script, pid, action, resolve(file), operation], {
        windowsHide: true,
        timeout: WAIT_MS,
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
              # Replace The Selection In One Native Edit, Including Its EN_CHANGE Notification.
              if (-not ("DokimePicker.Input" -as [type])) { Add-Type -Namespace DokimePicker -Name Input -MemberDefinition '[DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern System.IntPtr SendMessage(System.IntPtr window, uint message, System.IntPtr wide, System.IntPtr wide2); [DllImport("user32.dll", EntryPoint = "SendMessageW", ExactSpelling = true, CharSet = CharSet.Unicode)] public static extern System.IntPtr ReplaceSelection(System.IntPtr window, uint message, System.IntPtr wide, string text);' }
              $edit = [System.IntPtr]$field.Current.NativeWindowHandle
              if ($edit -eq [System.IntPtr]::Zero) { throw 'The Native Picker Filename Box Has No Window Handle' }
              [void][DokimePicker.Input]::SendMessage($edit, 0x00B1, [System.IntPtr]0, [System.IntPtr](-1))
              [void][DokimePicker.Input]::ReplaceSelection($edit, 0x00C2, [System.IntPtr]1, $env:DOKIME_PICKER_FILE)
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
    { windowsHide: true, timeout: WAIT_MS },
  );
  const accessibilityAddress = accessibility.stdout.match(/\bstring "([^"]+)"/);
  assert.ok(accessibilityAddress, "The private session owns an accessibility bus");
  process.env.AT_SPI_BUS_ADDRESS = accessibilityAddress[1];
  // Activated Registry Processes Must Use This Bus Too, Before Consulting The Shared Display.
  await executeFile("dbus-update-activation-environment", ["AT_SPI_BUS_ADDRESS"], {
    env: { ...process.env, DBUS_SESSION_BUS_ADDRESS: process.env.AT_SPI_BUS_ADDRESS },
    windowsHide: true,
    timeout: WAIT_MS,
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
    { windowsHide: true, timeout: WAIT_MS },
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
      { windowsHide: true, timeout: WAIT_MS },
    );
    return result.stdout.includes("boolean true");
  });
}

async function openNativeReport(path) {
  const reopened = await fetch(reportBrowser.endpoint, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ path }),
  });
  assert.ok(reopened.ok, "The owned report browser reopens saved HTML: " + (await reopened.text()));
  await reportBrowser.closeTabs();
}

async function discardHostCredentials(root) {
  const pending = [root];
  while (pending.length) {
    const directory = pending.pop();
    if (existsSync(join(directory, "backend-id"))) {
      await executeFile(credentialHost, ["--storage", directory, "--remove-credentials"], { windowsHide: true, timeout: WAIT_MS });
    }
    for (const entry of await readdir(directory, { withFileTypes: true })) {
      if (entry.isDirectory()) pending.push(join(directory, entry.name));
    }
  }
}

async function prepareNativeFixture(root, output) {
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
  reportBrowser = await ReportBrowser.start(work, chrome, WAIT_MS);
  await startSecretService();
  nativeScratch = await mkdtemp(join(tmpdir(), "dokime-native-scratch-"));
  scratchEnvironment = { TEMP: nativeScratch, TMP: nativeScratch, TMPDIR: nativeScratch };
  return async () => {
    const errors = [];
    for (const close of [
      () => reportBrowser.close(),
      stopNativePicker,
      () => terminate(driverProcess),
      () => discardHostCredentials(root),
      () => terminate(secretKeyring),
      () => terminate(secretBus),
      async () => {
        if (process.env.DOKIME_KEEP_EVIDENCE === "1") await cp(nativeScratch, join(evidence, "native-scratch"), { recursive: true });
        await rm(nativeScratch, { recursive: true, force: true, maxRetries: 5, retryDelay: 200 });
      },
    ]) {
      try {
        await close();
      } catch (error) {
        errors.push(error);
      }
    }
    await writeFile(join(evidence, "timings.json"), JSON.stringify(timings, null, 2));
    if (errors.length) throw new AggregateError(errors, "Native fixture cleanup failed");
  };
}

export {
  chooseNativeFile,
  startBackend,
  startSession,
  stopBackend,
  terminate,
  until,
  unusedPort,
  sizeViewport,
  setDensity,
  settle,
  answerPendingPrompts,
  prepareNativeFixture,
  openNativeReport,
  discardHostCredentials,
};
