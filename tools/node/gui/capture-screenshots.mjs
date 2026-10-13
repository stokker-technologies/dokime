// ***
//
// Dokime
//
// SPDX-License-Identifier: Apache-2.0
//
// ***

// Capture the product screenshots that `screenshots/` publishes.
//
// The live operator shell speaks native gRPC through Tauri IPC, so there is no
// HTTP facade a headless browser could load: the only honest capture is the real
// WebView. This drives the built desktop shell through WebKitWebDriver, exactly
// as `crates/dokime-gui/e2e/native-desktop.mjs` does, against a backend started
// here with the Mini charger testbench plugin loaded.
//
// The published set follows the redesigned routes:
//
//   cycles.png       Cycles home, the groups holding a finished and a live cycle
//   library.png      the Library with a suite expanded and its step timeline
//   dialog.png       the template dialog on step 1, Parameters
//   cycle.png        the cycle workspace while a case runs, with live plots
//   history.png      the History detail for a recorded cycle and its checkpoint
//   settings.png     Settings, on the section with the most to show
//   touch-cycle.png  the same workspace at touch density
//
// Usage: mise run screenshots:capture

/* === Imports === */

// Standard Library Imports

import { execFile, spawn } from "node:child_process";

import { once } from "node:events";

import { existsSync } from "node:fs";

import { mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";

import net from "node:net";

import { join, resolve } from "node:path";

import { setTimeout as delay } from "node:timers/promises";

import { fileURLToPath } from "node:url";

import { promisify } from "node:util";

// Third Party Imports

// Local Imports

/* === Re-Exports === */

/* === Constants === */

const run = promisify(execFile);

const root = fileURLToPath(new URL("../../../", import.meta.url));

const backendBinary = process.env.DOKIME_SHOTS_BACKEND ?? join(root, "target/debug/dokime-server");

const shellBinary = process.env.DOKIME_SHOTS_SHELL ?? join(root, "target/debug/dokime-gui");

const pluginBundle =
  process.env.DOKIME_SHOTS_PLUGIN ?? join(root, "target/debug/plugins/mini-sidecar.dokime");

const screenshots = join(root, "screenshots");

/**
 * A shell built with `tauri/custom-protocol` serves its own frontend from
 * `tauri://localhost`; a plain dev build instead loads the Vite dev server
 * directly, as `crates/dokime-gui/e2e/native-desktop.mjs` does. The binary
 * under `target/debug` here is the latter, so the capture drives the same
 * dev origin rather than the custom-protocol one.
 */
const appOrigin = process.env.DOKIME_SHOTS_ORIGIN ?? "http://127.0.0.1:5173";

const desktop = { density: "comfortable", height: 1000, width: 1600 };

const touch = { density: "touch", height: 1200, width: 1920 };

// The backend runs with a placeholder operator identity rather than the
// account and git configuration this machine happens to have, through
// `DOKIME_TEST_HUB_OPERATOR` and a throwaway `HOME`, exactly as
// `crates/dokime-gui/e2e/native-desktop.mjs` does. That script does not also
// fake the hostname, and the redesigned routes this capture visits
// (cycles/library/cycle/history/settings-appearance) never render
// `hostname_label` -- only the Diagnostics section would, and the capture
// does not shoot it -- so faking it here bought nothing but the
// `unshare --user --map-root-user` namespace, which is unrelated overhead.
const benchOperator = "Bench Operator";

/** The suite the captures compose from, and a longer observation so the plots fill. */
const SUITE = "Steady Charging";

const OBSERVATION_DURATION = "6";

const PROTECTION_SUITE = "Overcurrent Protection";

const OPERATOR_NOTE = "Trace steady, no fault indicated.";

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

function listening(port) {
  return new Promise((done) => {
    const socket = net.connect(port, "127.0.0.1");
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

async function until(description, check, timeout = 60000) {
  const deadline = Date.now() + timeout;
  while (Date.now() < deadline) {
    let value = false;
    try {
      value = await check();
    } catch {
      value = false;
    }
    if (value) return value;
    await delay(200);
  }
  throw new Error(`Timed out: ${description}`);
}

function xpathLiteral(value) {
  if (!value.includes("'")) return `'${value}'`;
  if (!value.includes('"')) return `"${value}"`;
  return `concat(${value
    .split("'")
    .map((part) => `'${part}'`)
    .join(', "\'", ')})`;
}

async function startBackend(work) {
  const port = await unusedPort();
  const child = spawn(
    backendBinary,
    [
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
      stdio: ["ignore", "ignore", "inherit"],
      env: {
        ...process.env,
        HOME: work,
        USER: "operator",
        USERNAME: "operator",
        LOGNAME: "operator",
        GIT_CONFIG_GLOBAL: "/dev/null",
        GIT_CONFIG_SYSTEM: "/dev/null",
        GIT_CONFIG_NOSYSTEM: "1",
        DOKIME_TEST_HUB_OPERATOR: benchOperator,
      },
    },
  );
  await until("backend listening", () => listening(port), 120000);
  return { child, endpoint: `http://127.0.0.1:${port}` };
}

async function startSession(work, endpoint) {
  const home = join(work, "shell");
  for (const directory of ["config", "data", "state", "cache", "bin"]) {
    await mkdir(join(home, directory), { recursive: true });
  }
  // A no-op `xdg-open` keeps the capture headless when a surface hands a path
  // to the operating system's opener.
  await writeFile(join(home, "bin", "xdg-open"), "#!/bin/sh\nexit 0\n", { mode: 0o755 });

  const port = await unusedPort();
  const webdriver = spawn("WebKitWebDriver", ["--host=127.0.0.1", `--port=${port}`], {
    detached: true,
    env: {
      ...process.env,
      HOME: home,
      TAURI_WEBVIEW_AUTOMATION: "true",
      PATH: `${join(home, "bin")}:${process.env.PATH ?? ""}`,
      XDG_CONFIG_HOME: join(home, "config"),
      XDG_DATA_HOME: join(home, "data"),
      XDG_STATE_HOME: join(home, "state"),
      XDG_CACHE_HOME: join(home, "cache"),
    },
    stdio: ["ignore", "ignore", "inherit"],
  });
  const base = `http://127.0.0.1:${port}`;
  const session = { closed: false, id: null, process: webdriver };

  async function request(method, path, body) {
    const response = await fetch(base + path, {
      method,
      headers: { "content-type": "application/json" },
      ...(body === undefined ? {} : { body: JSON.stringify(body) }),
      signal: AbortSignal.timeout(120000),
    });
    const result = (await response.json()).value;
    if (!response.ok || result?.error) {
      throw new Error(`WebDriver ${path}: ${result?.error ?? response.status}`);
    }
    return result;
  }

  await until("WebKitWebDriver startup", async () => {
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
  session.id = created.sessionId;

  // WebKitWebDriver serializes commands per session; keep one in flight.
  let queue = Promise.resolve();
  session.request = (method, path, body) => {
    const queued = queue.then(() => request(method, `/session/${session.id}${path}`, body));
    queue = queued.catch(() => undefined);
    return queued;
  };
  session.execute = (script, args = []) =>
    session.request("POST", "/execute/sync", { args, script });
  session.location = async () => new URL(await session.request("GET", "/url"));
  session.path = async () => (await session.location()).pathname;
  session.text = () => session.execute("return document.body.innerText");
  session.find = (using, value) => session.request("POST", "/element", { using, value });
  session.findAll = (using, value) => session.request("POST", "/elements", { using, value });
  session.clickElement = (element) =>
    session.execute(
      `const element = arguments[0];
       if (element.disabled) throw new Error('Control is disabled');
       element.scrollIntoView({block: 'center', inline: 'center'});
       element.focus();
       element.click();`,
      [element],
    );
  session.named = (role, name) => {
    const literal = xpathLiteral(name);
    const tag = role === "button" ? "button" : `*[@role=${xpathLiteral(role)}]`;
    return session.find(
      "xpath",
      `//${tag}[(normalize-space(.) = ${literal} or .//span[normalize-space(.) = ${literal}] or @aria-label = ${literal}) and not(@disabled)]`,
    );
  };
  session.clickNamed = async (role, name) =>
    session.clickElement(await until(`${role} ${name}`, () => session.named(role, name)));
  /** Some controls, like Cycle Actions, are icon-only: their name never joins `body.innerText`. */
  session.exists = async (role, name) =>
    session
      .named(role, name)
      .then(() => true)
      .catch(() => false);
  session.rect = (width, height) =>
    session.request("POST", "/window/rect", { height, width, x: 0, y: 0 });
  session.viewport = () => session.execute("return [window.innerWidth, window.innerHeight]");
  session.close = async () => {
    if (session.closed) return;
    try {
      await session.request("DELETE", "/window");
    } finally {
      session.closed = true;
    }
  };
  session.connected = () =>
    until(
      "connected shell",
      () =>
        session.execute("return Boolean(document.querySelector('nav[aria-label=Navigation]'));"),
      180000,
    );
  /**
   * Go to a route by loading it.
   *
   * A load drops the native session and the shell attaches again, which is the
   * price of driving the real deep links rather than poking the router's
   * history from outside it.
   */
  session.goto = async (path) => {
    await session.request("POST", "/url", { url: appOrigin + path });
    await until(
      `load ${path}`,
      async () => (await session.execute("return document.body.innerText.length")) > 20,
      120000,
    );
    await session.connected();
    await until(
      `route ${path}`,
      async () => (await session.path()) === new URL(appOrigin + path).pathname,
      30000,
    );
    await delay(900);
  };

  await until("main native window", async () => {
    const handles = await session.request("GET", "/window/handles");
    if (!handles.length) return false;
    await session.request("POST", "/window", { handle: handles[0] });
    return true;
  });
  return session;
}

/** Size the native window until its WebView reports the exact capture viewport. */
async function sizeViewport(session, variant) {
  let frame = { height: variant.height, width: variant.width };
  for (let attempt = 0; attempt < 8; attempt += 1) {
    await session.rect(frame.width, frame.height);
    await delay(400);
    const [width, height] = await session.viewport();
    if (width === variant.width && height === variant.height) return;
    frame = {
      height: frame.height + (variant.height - height),
      width: frame.width + (variant.width - width),
    };
  }
  throw new Error("The WebView never reported the requested capture viewport.");
}

/* Capture Helpers */

/** Transient notifications overlay the page bar; let them clear before a shot. */
async function settle(session) {
  await until(
    "transient notifications clear",
    async () =>
      session.execute(`
        return [...document.querySelectorAll('[data-sonner-toast]')].every((toast) => {
          if (toast.dataset.visible === 'false') return true;
          const style = getComputedStyle(toast);
          return style.pointerEvents === 'none' || !toast.getClientRects().length;
        });
      `),
    20000,
  ).catch(() => undefined);
  await delay(600);
}

async function shoot(session, name, variant) {
  await settle(session);
  const target = join(screenshots, `${name}.png`);
  const bytes = await session.request("GET", "/screenshot");
  await writeFile(target, Buffer.from(bytes, "base64"));
  // The panel renders at a device pixel ratio of two; publish the logical size.
  await run("magick", [
    target,
    "-strip",
    "-resize",
    `${variant.width}x${variant.height}`,
    "-define",
    "png:compression-level=9",
    target,
  ]);
  console.log(`captured ${target}`);
  return target;
}

/** Fail loudly rather than publish an operator's own name, address or machine. */
async function assertNoPersonalData(session, stage) {
  const hits = await session.execute(
    `
    const haystack = [
      document.body.innerText,
      [...document.querySelectorAll('[aria-label],[title],text,tspan')]
        .map((element) => element.getAttribute('aria-label') ?? element.getAttribute('title') ?? element.textContent)
        .join(' '),
    ].join(' ').toLowerCase();
    return arguments[0].filter((needle) => haystack.includes(needle));
  `,
    [personalDataNeedles()],
  );
  if (hits.length) {
    throw new Error(`${stage} renders personal data: ${hits.join(", ")}`);
  }
}

function personalDataNeedles() {
  const account = (process.env.USER ?? "").toLowerCase();
  return [account, "@", "/home/", "/tmp/dokime"].filter((needle) => needle.length > 1);
}

/** Set one parameter of the open template dialog by its rendered label. */
async function setParameter(session, name, value) {
  await session.execute(
    `const input = document.querySelector('[data-testid="parameter-input-' + arguments[0] + '"]');
     if (!input) throw new Error('No parameter input named ' + arguments[0]);
     const setter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value').set;
     setter.call(input, arguments[1]);
     input.dispatchEvent(new Event('input', {bubbles: true}));
     input.dispatchEvent(new Event('change', {bubbles: true}));`,
    [name, value],
  );
  await delay(400);
}

/** Answer whatever prompt the open cycle is waiting on, through the banner. */
async function answerPendingPrompts(session) {
  const pending = await session.execute(`
    const banner = document.querySelector('section[aria-label="Pending Prompts"]');
    if (!banner) return false;
    const respond = [...banner.querySelectorAll('button')].find(
      (button) => button.innerText.trim() === 'Respond',
    );
    respond?.click();
    return Boolean(respond);
  `);
  if (!pending) return false;
  await delay(700);
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
     }
     const submit = [...panel.querySelectorAll('button')].find((button) =>
       /^(Submit|Confirm|Accept|Continue)$/.test(button.innerText.trim()),
     );
     submit?.click();`,
    [OPERATOR_NOTE],
  );
  await delay(1500);
  return true;
}

/** Every case row has a terminal status and none is in flight. */
async function casesSettled(session) {
  const rows = await session.execute(`
    return [...document.querySelectorAll('nav[aria-label=Navigation] li[data-level=case]')]
      .map((row) => row.innerText)
      .join(' | ');
  `);
  if (/\b(Running|Waiting|Finalizing)\b/.test(rows)) return false;
  return /\b(Passed|Failed|Blocked)\b/.test(rows);
}

async function runUntil(session, description, check, timeout) {
  const deadline = Date.now() + timeout;
  while (Date.now() < deadline) {
    if (await check()) return;
    await answerPendingPrompts(session);
    await delay(1500);
  }
  throw new Error(`Timed out: ${description}`);
}

/* Materialization */

/**
 * Compose one cycle from a suite through the template dialog.
 *
 * The dialog is the only route by which cases enter a cycle, so the capture
 * takes the same one an operator does, and shoots step 1 on the way through.
 */
async function materialize(session, { appendTo, capture, duration, suite }) {
  await session.goto(appendTo ? `/library?appendTo=${appendTo}` : "/library");
  await until("suite rows", async () => (await session.text()).includes(suite), 120000);
  await session.clickNamed("button", suite);
  await until("expanded suite", async () => (await session.text()).includes("Configure"), 30000);
  await delay(900);

  if (capture === "library") {
    await assertNoPersonalData(session, "library");
    await shoot(session, "library", desktop);
  }

  await session.clickNamed("button", "Configure");
  await until("template dialog", () => session.find("css selector", "[role=dialog]"), 30000);
  await delay(900);
  if (duration) await setParameter(session, "duration_s", duration);

  if (capture === "dialog") {
    await assertNoPersonalData(session, "dialog");
    await shoot(session, "dialog", desktop);
  }

  await session.clickNamed("button", "Next");
  await until(
    "summary step",
    async () => (await session.text()).includes("Finish And Add"),
    120000,
  );
  await session.clickNamed("button", "Finish And Add");
  return until(
    "the materialized cycle",
    async () => (await session.location()).searchParams.get("appendTo"),
    120000,
  );
}

/* The Capture Itself */

async function capture(session) {
  await sizeViewport(session, desktop);
  await session.connected();
  await until("cycles home", async () => (await session.path()) === "/cycles", 180000);
  await delay(2500);

  /* One cycle for the workspace and its plots, and a second so the Cycles home
     and the History library both have more than one row to show. */
  const first = await materialize(session, {
    capture: "dialog",
    duration: OBSERVATION_DURATION,
    suite: PROTECTION_SUITE,
  });
  const cycleId = await materialize(session, {
    capture: "library",
    duration: OBSERVATION_DURATION,
    suite: SUITE,
  });

  /* Cycle: the workspace while a case streams telemetry into its plots. */
  await session.goto(`/cycles/${cycleId}`);
  await until("cycle workspace", () => session.exists("button", "Cycle Actions"), 60000);
  await startRun(session);
  await until("a running case", async () => /Running|Waiting/.test(await session.text()), 120000);
  await selectFirstCase(session);
  await widenPlotWindow(session);
  await until(
    "streaming telemetry",
    async () => {
      const points = /([\d,]+) points from/.exec(await session.text());
      return points ? Number(points[1].replace(/,/g, "")) >= 500 : false;
    },
    180000,
  );
  await assertNoPersonalData(session, "cycle");
  await shoot(session, "cycle", desktop);

  /* Touch: the same workspace at the tablet's density. */
  await setDensity(session, touch);
  await until("touch workspace", () => session.exists("button", "Cycle Actions"), 60000);
  await assertNoPersonalData(session, "touch-cycle");
  await shoot(session, "touch-cycle", touch);
  await setDensity(session, desktop);

  /* Let both cycles finish, so Cycles home and History have settled rows. */
  await runUntil(session, "cycle completion", () => casesSettled(session), 600000);
  await session.goto(`/cycles/${first}`);
  await until("the first cycle", () => session.exists("button", "Cycle Actions"), 60000);
  await startRun(session);
  await runUntil(session, "first cycle completion", () => casesSettled(session), 600000);

  /* Cycles: the home, with the groups holding real work. */
  await session.goto("/cycles");
  await until(
    "cycle rows",
    async () => /Completed|Failed|Stopped/.test(await session.text()),
    120000,
  );
  await delay(1500);
  await assertNoPersonalData(session, "cycles");
  await shoot(session, "cycles", desktop);

  /* History: the recorded executions behind the runs, one checkpoint open. */
  await session.goto("/history");
  await openHistoryDetail(session);
  await assertNoPersonalData(session, "history");
  await shoot(session, "history", desktop);

  /* Settings: Appearance, whose rows are the same on every machine. The
     Diagnostics section publishes this installation's own paths, so it is not
     something a published asset can carry. */
  await session.goto("/settings/appearance");
  await until("appearance", async () => (await session.text()).includes("Density"), 60000);
  await delay(1200);
  await assertNoPersonalData(session, "settings");
  await shoot(session, "settings", desktop);
}

/** Density is an Appearance setting, so the capture switches it where an operator would. */
async function setDensity(session, variant) {
  const before = await session.path();
  await session.goto("/settings/appearance");
  await until("appearance", async () => (await session.text()).includes("Density"), 60000);
  await session.clickNamed("radio", variant.density === "touch" ? "Touch" : "Comfortable");
  await until(
    `density ${variant.density}`,
    async () =>
      (await session.execute("return document.documentElement.dataset.density")) ===
      variant.density,
    20000,
  );
  await sizeViewport(session, variant);
  await session.goto(before);
}

/**
 * Start the open cycle.
 *
 * Run Queued is the control for this, and the capture uses it when the header
 * offers it. A cycle whose cases all arrive with a pre-dispatch attempt leaves
 * the header without it, and then the row menus are the only way in.
 */
async function startRun(session) {
  const queued = await session
    .named("button", "Run Queued")
    .then(() => true)
    .catch(() => false);
  if (queued) {
    await session.clickNamed("button", "Run Queued");
    return;
  }
  const labels = await session.execute(`
    return [...document.querySelectorAll('nav[aria-label=Navigation] button[aria-label^="Actions For "]')].map(
      (button) => button.getAttribute('aria-label'),
    );
  `);
  for (const label of labels) {
    await session.clickNamed("button", label);
    await session.clickNamed("menuitem", "Run Case");
    await delay(800);
  }
}

/** Select the first case so the evidence pane shows an attempt and its plots. */
async function selectFirstCase(session) {
  await session.execute(`
    const rows = [
      ...document.querySelectorAll('nav[aria-label=Navigation] li[data-level=case]'),
    ].filter((row) => /Running|Waiting|Passed|Failed|Queued/.test(row.innerText));
    if (!rows.length) throw new Error('No case row to select');
    rows[0].querySelector('button').click();
  `);
  await until(
    "a selected case",
    async () => (await session.location()).searchParams.get("caseId"),
    30000,
  );
  await delay(1500);
}

/**
 * Widen the plot window.
 *
 * The window defaults to two minutes of wall clock while sample stamps follow
 * simulated time, so the trace only fills its panel at the widest setting.
 */
async function widenPlotWindow(session) {
  await session
    .execute(`
      const slider = document.querySelector('[role="slider"]');
      if (!slider) return false;
      slider.focus();
      for (let step = 0; step < 6; step += 1) {
        for (const type of ['keydown', 'keyup']) {
          slider.dispatchEvent(new KeyboardEvent(type, {bubbles: true, code: 'ArrowRight', key: 'ArrowRight'}));
        }
      }
      return true;
    `)
    .catch(() => undefined);
  await delay(1200);
}

async function openHistoryDetail(session) {
  const row = await until(
    "recorded cycle row",
    async () => {
      const rows = await session.findAll("css selector", "tbody tr");
      return rows[0] ?? false;
    },
    120000,
  );
  await session.clickElement(row);
  await until(
    "history detail",
    async () =>
      (await session.path()).startsWith("/history/") &&
      /Inspect checkpoint/i.test(await session.text()),
    120000,
  );
  await session.clickNamed("button", "Inspect checkpoint");
  await until(
    "captured execution",
    async () => /Recorded Steps/i.test(await session.text()),
    60000,
  );
  await session.clickNamed("button", "Records");
  await delay(2000);
}

async function main() {
  for (const [label, path] of [
    ["the backend", backendBinary],
    ["the desktop shell", shellBinary],
    ["the Mini charger plugin bundle", pluginBundle],
  ]) {
    if (!existsSync(path)) {
      throw new Error(`Missing ${label} at ${path}. Run "mise run screenshots:build" first.`);
    }
  }
  if (!process.env.DISPLAY && !process.env.WAYLAND_DISPLAY) {
    throw new Error("Capturing the native WebView needs a running display session.");
  }

  await mkdir(screenshots, { recursive: true });
  const work = await mkdtemp(join(root, "target/screenshots-"));
  let backend;
  let session;
  try {
    backend = await startBackend(work);
    session = await startSession(work, backend.endpoint);
    await capture(session);
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
    await rm(work, { force: true, recursive: true });
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

/* === Entry Point === */

await main();
