/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/** Capture One Fixed Product Journey In Both Themes. */

/* === Imports === */

// Standard Library Imports
import assert from "node:assert/strict";
import { execFile } from "node:child_process";
import { randomUUID } from "node:crypto";
import { existsSync } from "node:fs";
import { copyFile, cp, mkdir, mkdtemp, readdir, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { parseArgs, promisify } from "node:util";

// Third Party Imports

// Local Imports
import {
  answerPendingPrompts,
  chooseNativeFile,
  prepareNativeFixture,
  setDensity,
  settle,
  sizeViewport,
  startBackend,
  startSession,
  stopBackend,
  terminate,
  until,
} from "./native-fixture.mjs";
import { call, invoke } from "./native-rpc.mjs";

/* === Re-Exports === */

/* === Constants === */

const { values: options } = parseArgs({ options: { scene: { type: "string" } } });
const scenes = ["cycle", "plots", "report", "history", "library", "cycles", "touch"];
assert.ok(!options.scene || scenes.includes(options.scene), "Unknown screenshot scene");
const workspace = resolve(dirname(fileURLToPath(import.meta.url)), "../../..");
const folder = join(workspace, "docs/screenshots");
const work = await mkdtemp(join(tmpdir(), "dokime-screenshots-"));
await mkdir(join(workspace, "target/screenshots"), { recursive: true });
const evidence = await mkdtemp(join(workspace, "target/screenshots/run-"));
const entries = [];
const desktop = { width: 1600, height: 1000, density: "comfortable" };
const touch = { width: 1920, height: 1200, density: "touch" };
const environment = {
  USER: "dokime-bench",
  USERNAME: "dokime-bench",
  NAME: "Bench Operator",
  GIT_CONFIG_NOSYSTEM: "1",
  GIT_CONFIG_GLOBAL: join(work, "empty-gitconfig"),
  GIT_CONFIG_COUNT: "0",
  DOKIME_TEST_HUB_OPERATOR: "Bench Operator",
  RUST_LOG: "info,dokime_runtime_next=debug",
};
let session;
let backend;
let cleanup;
let failure;
const cleanupErrors = [];

/* === Types === */

/* === Classes === */

/* === Public Functions === */

/* === Private Functions === */

async function capture(scene, ready, prepare) {
  if (options.scene && options.scene !== scene) return;
  const viewport = scene === "touch" ? touch : desktop;
  for (const theme of ["light", "dark"]) {
    // Set Appearance Through The Product Controls.
    if (process.platform === "win32") {
      await session.request("POST", "/ms/cdp/execute", { cmd: "Emulation.clearDeviceMetricsOverride", params: {} });
    }
    await setDensity(session, viewport);
    if (process.platform === "win32") {
      await session.request("POST", "/ms/cdp/execute", {
        cmd: "Emulation.setDeviceMetricsOverride",
        params: { width: viewport.width, height: viewport.height, deviceScaleFactor: 1, mobile: false },
      });
    }
    await session.clickNamed("combobox", "Theme Mode");
    await session.clickNamed("option", theme === "light" ? "Light" : "Dark");
    await until(`${theme} theme`, () =>
      session.execute("return document.documentElement.classList.contains('dark') === arguments[0]", [theme === "dark"]),
    );
    await prepare(theme);
    await until(`${scene} ready`, ready);
    if (["cycle", "plots", "touch"].includes(scene)) {
      await until("populated trace statistics", () =>
        session.execute(`
        const count = document.querySelector('table[aria-label="Trace Statistics"] tbody tr td:nth-child(4)')?.innerText;
        return Number(count?.replaceAll(',', '')) >= 32;
      `),
      );
    }
    if (scene === "plots") {
      await until("zoomed trace statistics settled", () =>
        session.execute(`
          const table = document.querySelector('table[aria-label="Trace Statistics"]');
          const grid = table?.closest('[data-slot=data-table]');
          return grid && !grid.classList.contains('opacity-50') && getComputedStyle(grid).opacity === '1';
        `),
      );
    }
    if (["cycle", "touch"].includes(scene)) {
      const window = await session.find("css selector", '[role="slider"][aria-label$="Window"]');
      await session.execute("arguments[0].focus()", [window]);
      await session.key("End");
      await session.clickNamed("button", "Refresh");
    }
    if (["cycle", "touch"].includes(scene)) {
      await until("complete evidence range refreshed", () =>
        session.execute(`
        const section = document.querySelector('[data-execution-section=Report]');
        const refresh = [...section.querySelectorAll('button')].find(node => node.innerText === 'Refresh');
        const loading = [...section.querySelectorAll('[data-testid=plot-placeholder]')].some(node => { const r = node.getBoundingClientRect(); return r.top < innerHeight && r.bottom > 0; });
        return refresh?.getAttribute('aria-busy') === 'false' && !loading;
      `),
      );
    }
    await settle(session);
    await session.executeAsync(
      "const done = arguments[arguments.length - 1]; document.fonts.ready.then(() => requestAnimationFrame(() => requestAnimationFrame(done)));",
    );
    await until("transient notifications dismissed", () =>
      session.execute("return document.querySelectorAll('[data-sonner-toast]').length === 0"),
    );
    if (["plots", "report"].includes(scene)) {
      // Show The Same Loaded Signal Maps In The Interface And Saved HTML.
      await until("loaded signal maps", () =>
        session.execute(`
        const doc = document.querySelector('#screenshot-report')?.contentDocument ?? document;
        const maps = doc.querySelector('[data-section-id="obc.charger.startup.sequence.maps"]');
        maps?.scrollIntoView({block:'start'});
        return !!maps?.querySelector('[data-testid="contour-plot"]') &&
          !!maps.querySelector('[data-testid="grid-chart-plot"]') &&
          Number(maps.querySelector('[aria-label="Electrical Trajectory Instant"][role="slider"]')?.getAttribute('aria-valuemax')) > 0;
      `),
      );
      const gridInstant = await session.execute(`
        const doc = document.querySelector('#screenshot-report')?.contentDocument ?? document;
        return doc.querySelector('[aria-label="Channel Survey Instant"][role="slider"]').getAttribute('aria-valuenow');
      `);
      await session.execute(`
        const doc = document.querySelector('#screenshot-report')?.contentDocument ?? document;
        const slider = doc.querySelector('[aria-label="Electrical Trajectory Instant"][role="slider"]');
        slider.focus();
        slider.dispatchEvent(new KeyboardEvent('keydown', {key:'Home',code:'Home',bubbles:true}));
      `);
      await until("trajectory selection", () =>
        session.execute(`
        const doc = document.querySelector('#screenshot-report')?.contentDocument ?? document;
        return doc.querySelector('[aria-label="Electrical Trajectory Instant"][role="slider"]').getAttribute('aria-valuenow') === '0';
      `),
      );
      assert.equal(
        await session.execute(`
        const doc = document.querySelector('#screenshot-report')?.contentDocument ?? document;
        return doc.querySelector('[aria-label="Channel Survey Instant"][role="slider"]').getAttribute('aria-valuenow');
      `),
        gridInstant,
        "Trajectory selection leaves the grid unchanged",
      );
      await session.execute(`
        const doc = document.querySelector('#screenshot-report')?.contentDocument ?? document;
        const controls = doc.querySelector('[aria-label="Electrical Trajectory Playback"]');
        [...controls.querySelectorAll('button')].find(button => button.innerText === 'Replay').click();
      `);
      await until("trajectory replay", () =>
        session.execute(`
        const doc = document.querySelector('#screenshot-report')?.contentDocument ?? document;
        const controls = doc.querySelector('[aria-label="Electrical Trajectory Playback"]');
        return ![...controls.querySelectorAll('button')].find(button => button.innerText === 'Pause').disabled;
      `),
      );
      await session.execute(`
        const doc = document.querySelector('#screenshot-report')?.contentDocument ?? document;
        const controls = doc.querySelector('[aria-label="Electrical Trajectory Playback"]');
        [...controls.querySelectorAll('button')].find(button => button.innerText === 'Pause').click();
      `);
      await until("trajectory pause", () =>
        session.execute(`
        const doc = document.querySelector('#screenshot-report')?.contentDocument ?? document;
        const controls = doc.querySelector('[aria-label="Electrical Trajectory Playback"]');
        return [...controls.querySelectorAll('button')].find(button => button.innerText === 'Pause').disabled;
      `),
      );
      assert.equal(
        await session.execute(`
        const doc = document.querySelector('#screenshot-report')?.contentDocument ?? document;
        return doc.querySelector('[aria-label="Channel Survey Instant"][role="slider"]').getAttribute('aria-valuenow');
      `),
        gridInstant,
        "Trajectory replay leaves the grid unchanged",
      );
      const trajectoryInstant = await session.execute(`
        const doc = document.querySelector('#screenshot-report')?.contentDocument ?? document;
        return doc.querySelector('[aria-label="Electrical Trajectory Instant"][role="slider"]').getAttribute('aria-valuenow');
      `);
      await session.execute(`
        const doc = document.querySelector('#screenshot-report')?.contentDocument ?? document;
        const slider = doc.querySelector('[aria-label="Channel Survey Instant"][role="slider"]');
        if (slider.getAttribute('aria-disabled') !== 'true') {
          slider.focus();
          slider.dispatchEvent(new KeyboardEvent('keydown', {key:'Home',code:'Home',bubbles:true}));
        }
      `);
      await until("grid selection", () =>
        session.execute(`
        const doc = document.querySelector('#screenshot-report')?.contentDocument ?? document;
        return doc.querySelector('[aria-label="Channel Survey Instant"][role="slider"]').getAttribute('aria-valuenow') === '0';
      `),
      );
      await session.execute(`
        const doc = document.querySelector('#screenshot-report')?.contentDocument ?? document;
        const controls = doc.querySelector('[aria-label="Channel Survey Playback"]');
        const replay = [...controls.querySelectorAll('button')].find(button => button.innerText === 'Replay');
        if (!replay.disabled) replay.click();
      `);
      await session.execute(`
        const doc = document.querySelector('#screenshot-report')?.contentDocument ?? document;
        const controls = doc.querySelector('[aria-label="Channel Survey Playback"]');
        const pause = [...controls.querySelectorAll('button')].find(button => button.innerText === 'Pause');
        if (!pause.disabled) pause.click();
      `);
      assert.equal(
        await session.execute(`
        const doc = document.querySelector('#screenshot-report')?.contentDocument ?? document;
        return doc.querySelector('[aria-label="Electrical Trajectory Instant"][role="slider"]').getAttribute('aria-valuenow');
      `),
        trajectoryInstant,
        "Grid selection and replay leave the trajectory unchanged",
      );
      await session.execute(`
        const doc = document.querySelector('#screenshot-report')?.contentDocument ?? document;
        const controls = doc.querySelector('[aria-label="Electrical Trajectory Playback"]');
        [...controls.querySelectorAll('button')].find(button => button.innerText === 'Latest').click();
        doc.querySelector('[data-section-id="obc.charger.startup.sequence.maps"]').scrollIntoView({block:'start'});
      `);
    }
    if (["cycle", "touch"].includes(scene)) {
      await session.execute("document.querySelector('[data-execution-section=Report]').scrollIntoView({block:'start'});");
      await session.executeAsync("const done = arguments[arguments.length - 1]; requestAnimationFrame(() => requestAnimationFrame(done));");
    }
    const errors = await session.execute(`
      const doc = document.querySelector('#screenshot-report')?.contentDocument ?? document;
      return [...doc.querySelectorAll('[role="alert"]')].filter(node => node.getClientRects().length).map(node => node.innerText);
    `);
    assert.deepEqual(errors, [], `${scene} must contain no error panels`);
    const emptyBars = await session.execute(`
      const doc = document.querySelector('#screenshot-report')?.contentDocument ?? document;
      return [...doc.querySelectorAll('button[aria-label*=" to "] > span')].filter(node => {
        const bounds = node.getBoundingClientRect();
        return parseFloat(node.style.height) > 0 && bounds.width === 0;
      }).length;
    `);
    assert.equal(emptyBars, 0, `${scene} has histogram samples with invisible bars`);
    // Capture Only The Webview And Verify Its Native Pixel Dimensions.
    const image = `${scene}-${theme}.png`;
    const path = await session.screenshot(image.slice(0, -4));
    const bytes = await readFile(path);
    assert.equal(bytes.readUInt32BE(16), viewport.width);
    assert.equal(bytes.readUInt32BE(20), viewport.height);
    await session.execute("document.querySelector('#screenshot-report')?.remove()");
    entries.push({
      image,
      scene,
      theme,
      viewport: { width: viewport.width, height: viewport.height },
      density: viewport.density,
      revision,
      captured_at: new Date().toISOString(),
    });
    process.stdout.write(`Captured ${image}\n`);
  }
}

/* === Entry Point === */

// Record The Source Revision And Start Owned Services On Fresh Storage.
const { stdout } = await promisify(execFile)("git", ["rev-parse", "HEAD"], { cwd: workspace, windowsHide: true });
const revision = stdout.trim();
try {
  await mkdir(folder, { recursive: true });
  await mkdir(evidence, { recursive: true });
  await writeFile(join(work, "empty-gitconfig"), "");
  const chrome = join(process.env.ProgramFiles ?? "", "Google/Chrome/Application/chrome.exe");
  if (process.platform === "win32" && !process.env.DOKIME_E2E_CHROME && existsSync(chrome)) process.env.DOKIME_E2E_CHROME = chrome;
  cleanup = await prepareNativeFixture(work, evidence);
  backend = await startBackend(work, evidence, undefined, "backend", true, { environment });
  session = await startSession(work, backend.endpoint, evidence);
  await session.connected();
  await sizeViewport(session, desktop);

  // Materialize A Real Bench Case With Finite Paced Measurements.
  const catalog = await call(session, "Catalog", "ListSuites");
  const suite = catalog.suites.find((row) => row.metadata.label === "Charger Start-Up");
  assert.ok(suite, "The bench plugin registers its startup suite");
  const definition = await call(session, "Catalog", "GetSuite", { suiteId: suite.suiteId });
  await writeFile(join(evidence, "suite.json"), JSON.stringify(definition, null, 2));
  const template = definition.templates.find((row) => row.metadata.label === "Start-Up Sequence");

  // The Library Has No Execution Prerequisite.
  if (options.scene === "library") {
    await capture(
      "library",
      () => session.execute("return document.querySelectorAll('[role=dialog] [aria-label=\"Procedure Steps\"] li').length > 0"),
      async () => {
        await session.goto("/library");
        await session.clickCss('details:has(table[aria-label="Charger Start-Up"]) > summary');
        await session.clickCss('table[aria-label="Charger Start-Up"] tbody tr');
        await until("template parameters", () => session.find("css selector", "[data-testid=parameter-editor-grid]"));
      },
    );
  } else {
    const overrides = {
      duration_s: ["Number", 1],
      sample_interval_s: ["Number", 0.001],
      realtime: ["Bool", true],
      continuous: ["Bool", false],
    };
    const parameters = Object.entries(overrides).map(([name, [kind, value]]) => {
      const parameter = template.parameters.find(
        (row) => row.name === name || row.metadata?.identifier === name || row.parameterId?.endsWith(`.${name}`),
      );
      assert.ok(parameter, `The bench declares ${name}`);
      return { key: name, value: { values: [{ nodes: [{ kind: { [kind]: { field0: value } } }] }] } };
    });
    const plan = await call(session, "Planning", "MaterializeTemplate", {
      selection: { suiteId: suite.suiteId, templateId: template.templateId, parameters },
    });
    const cycle = await call(session, "Execution", "CreateCycle", {
      metadata: { label: "AC Characterization" },
      continueAutomatically: false,
    });
    const executionId = randomUUID();
    await call(session, "Execution", "AppendExecutions", {
      cycleId: cycle.cycleId,
      executions: [{ executionId, caseId: plan.cases[0].caseId }],
    });
    const route = `/cycles/${cycle.cycleId}?executionId=${executionId}`;
    await session.goto(route);
    await session.clickNamed("button", "Start");
    // Keep Inspection Pending Until Both Running Captures Are Saved.
    await until("live measurement plots", async () => {
      await session.execute("document.querySelector('[data-execution-section=Report]')?.scrollIntoView({block:'start'});");
      return session.execute("return document.querySelectorAll('.uplot canvas').length >= 1 && /Running/.test(document.body.innerText)");
    });

    // Capture Running Evidence Before The Case Finishes.
    await capture(
      "cycle",
      () => session.execute("return document.querySelectorAll('.uplot canvas').length >= 1 && /Running/.test(document.body.innerText)"),
      async () => {
        await session.goto(route);
        await until("live report plots", async () => {
          await session.execute("document.querySelector('[data-execution-section=Report]')?.scrollIntoView({block:'start'});");
          return session.find("css selector", ".uplot canvas");
        });
        await session.execute("document.querySelector('[data-execution-section=Report]').scrollIntoView({block:'start'});");
      },
    );

    // A Selected Running Scene Needs No Finished Execution.
    if (options.scene === "touch") {
      await capture(
        "touch",
        () => session.execute("return document.querySelectorAll('.uplot canvas').length >= 1 && /Running/.test(document.body.innerText)"),
        async () => {
          await session.goto(route);
          await until("touch live plots", async () => {
            await session.execute("document.querySelector('[data-execution-section=Report]')?.scrollIntoView({block:'start'});");
            return session.find("css selector", ".uplot canvas");
          });
        },
      );
    }
    if (["cycle", "touch"].includes(options.scene)) {
      await call(session, "Execution", "StopExecution", { executionId });
    } else {
      // Finish And Seal The Measured Execution Through The Real Backend.
      await session.goto(route);
      const finished = await until(
        "finished measured execution",
        async () => {
          await answerPendingPrompts(session);
          const result = await call(session, "Execution", "GetExecution", { executionId });
          await writeFile(join(evidence, "execution.json"), JSON.stringify(result, null, 2));
          return result.summary?.sealed ||
            result.summary?.status?.Failed ||
            result.summary?.status?.Cancelled ||
            result.summary?.status?.Blocked
            ? result
            : null;
        },
        undefined,
        500,
      );
      assert.ok(finished.summary.status.Passed, `Measured case did not pass: ${JSON.stringify(finished.summary.status)}`);
      await call(session, "Execution", "SealCycle", { cycleId: cycle.cycleId });

      // Show Finished Plots And Hold A Zoom On The First Trace.
      await capture(
        "plots",
        () =>
          session.execute(
            "return document.querySelectorAll('.uplot canvas').length >= 1 && !!document.querySelector('table[aria-label=\"Trace Statistics\"]')",
          ),
        async () => {
          await session.goto(route);
          await until("finished report section", () => session.find("css selector", "[data-execution-section=Report]"));
          await session.execute("document.querySelector('[data-execution-section=Report]').scrollIntoView({block:'start'});");
          const window = await session.find("css selector", '[role="slider"][aria-label$="Window"]');
          await session.execute("arguments[0].focus()", [window]);
          await session.key("End");
          await session.clickNamed("button", "Refresh");
          await until("finished plot statistics", () => session.find("css selector", 'table[aria-label="Trace Statistics"]'));
          await session.execute("document.querySelector('.uplot').scrollIntoView({block:'start'});");
          const bounds = await session.execute(
            "const r = document.querySelector('.u-over').getBoundingClientRect(); return {x:r.x,y:r.y,width:r.width,height:r.height}",
          );
          await session.request("POST", "/actions", {
            actions: [
              {
                type: "pointer",
                id: "zoom",
                parameters: { pointerType: "mouse" },
                actions: [
                  {
                    type: "pointerMove",
                    duration: 0,
                    x: Math.round(bounds.x + bounds.width * 0.2),
                    y: Math.round(bounds.y + bounds.height * 0.25),
                  },
                  { type: "pointerDown", button: 0 },
                  {
                    type: "pointerMove",
                    duration: 350,
                    x: Math.round(bounds.x + bounds.width * 0.65),
                    y: Math.round(bounds.y + bounds.height * 0.8),
                  },
                  { type: "pointerUp", button: 0 },
                ],
              },
            ],
          });
          await session.request("DELETE", "/actions");
          await session.execute("document.querySelector('.uplot').closest('.plot-card').scrollIntoView({block:'start'});");
        },
      );

      // Generate The Saved Report And Open Its Actual HTML In The Webview.
      const reportPath = join(work, "report.html");
      let delivered = false;
      const delivery = invoke(session, "desktop_artifact_deliver", {
        generation: (await invoke(session, "desktop_session_status")).connection.generation,
        request: { source: { kind: "execution_report", executionId }, kind: "html_report", filename: "report.html", action: "save" },
      }).finally(() => {
        delivered = true;
      });
      const picker = { window: null };
      await until("report save picker", () => delivered || chooseNativeFile(picker, reportPath, "Save"), undefined, 25);
      const saved = await delivery;
      const html = await readFile(saved.path, "utf8");
      assert.ok(html.includes('id="hub-report-data"'), "The real saved report contains its captured document");
      await capture(
        "report",
        () =>
          session.execute(
            "const doc = document.querySelector('#screenshot-report')?.contentDocument; if (!doc) return false; const row = doc.querySelector('li[data-level=case] button[data-tree-select]'); if (!doc.querySelector('[data-execution-section=Report]')) row?.click(); doc.querySelector('[data-execution-section=Report]')?.scrollIntoView({block:'start'}); return doc.querySelectorAll('.uplot canvas').length > 0",
          ),
        async (theme) => {
          await session.goto(route);
          await session.execute(
            `const frame = document.createElement('iframe'); frame.id = 'screenshot-report'; frame.style = 'position:fixed;inset:0;width:100vw;height:100vh;border:0;z-index:9999;background:var(--background)'; frame.style.colorScheme = arguments[1]; frame.srcdoc = arguments[0]; document.body.append(frame);`,
            [html, theme],
          );
        },
      );
      await session.execute("document.querySelector('#screenshot-report')?.remove()");

      // Inspect The Recorded Execution Checkpoint.
      await capture(
        "history",
        () =>
          session.execute(
            "return document.body.innerText.includes('Recorded Steps') && !!document.querySelector('table[aria-label=\"History Executions\"]')",
          ),
        async () => {
          await session.goto(`/history/${cycle.cycleId}`);
          await session.clickCss('table[aria-label="History Executions"] button[aria-label="Inspect"]');
          await until("recorded checkpoint", () => session.execute("return document.body.innerText.includes('Recorded Steps')"));
        },
      );

      // Show The Current Library Template Presentation.
      await capture(
        "library",
        () => session.execute("return document.querySelectorAll('[role=dialog] [aria-label=\"Procedure Steps\"] li').length > 0"),
        async () => {
          await session.goto("/library");
          await session.clickCss('details:has(table[aria-label="Charger Start-Up"]) > summary');
          await session.clickCss('table[aria-label="Charger Start-Up"] tbody tr');
          await until("template parameters", () => session.find("css selector", "[data-testid=parameter-editor-grid]"));
        },
      );
      await session.key("Escape");

      // Retain An Active Cycle Beside The Completed One On The Home Surface.
      const active = await call(session, "Execution", "CreateCycle", {
        metadata: { label: "Continuous Observation" },
        continueAutomatically: false,
      });
      const activeId = randomUUID();
      await call(session, "Execution", "AppendExecutions", {
        cycleId: active.cycleId,
        executions: [{ executionId: activeId, caseId: plan.cases[0].caseId }],
      });
      await session.goto(`/cycles/${active.cycleId}?executionId=${activeId}`);
      await session.clickNamed("button", "Start");
      // The Operator Answers After Capturing The Active And Touch Views.
      await until("active observation", async () => {
        await session.execute("document.querySelector('[data-execution-section=Report]')?.scrollIntoView({block:'start'});");
        return session.execute("return document.querySelectorAll('.uplot canvas').length >= 1 && /Running/.test(document.body.innerText)");
      });
      await capture(
        "cycles",
        () =>
          session.execute(
            "return document.body.innerText.includes('AC Characterization') && document.body.innerText.includes('Continuous Observation') && !document.body.innerText.includes('Loading cycles')",
          ),
        () => session.goto("/cycles"),
      );
      await capture(
        "touch",
        () => session.execute("return document.querySelectorAll('.uplot canvas').length >= 1 && /Running/.test(document.body.innerText)"),
        async () => {
          await session.goto(`/cycles/${active.cycleId}?executionId=${activeId}`);
          await until("touch live plots", async () => {
            await session.execute("document.querySelector('[data-execution-section=Report]')?.scrollIntoView({block:'start'});");
            return session.find("css selector", ".uplot canvas");
          });
        },
      );

      // Let The Active Case Finish Normally Before Shutting Down Its Backend.
      await session.goto(`/cycles/${active.cycleId}?executionId=${activeId}`);
      const observed = await until(
        "active observation completed",
        async () => {
          await answerPendingPrompts(session);
          const result = await call(session, "Execution", "GetExecution", { executionId: activeId });
          return result.summary?.sealed ||
            result.summary?.status?.Failed ||
            result.summary?.status?.Cancelled ||
            result.summary?.status?.Blocked
            ? result
            : null;
        },
        undefined,
        500,
      );
      assert.ok(observed.summary.status.Passed, `Observation did not pass: ${JSON.stringify(observed.summary.status)}`);
      await call(session, "Execution", "SealCycle", { cycleId: active.cycleId });
    }
  }
} catch (error) {
  failure = error;
  const view = session ? await session.text().catch(() => "Webview unavailable") : "Webview not started";
  await writeFile(join(evidence, "failure.txt"), `${error.stack ?? String(error)}\n\n${view}`).catch((captureError) =>
    cleanupErrors.push(captureError),
  );
  if (session) {
    await session.screenshot("failure").catch(() => {});
  }
} finally {
  // Stop Only Processes Started By This Journey; Preserve Failure Evidence.
  for (const [label, stop] of [
    [
      "desktop",
      async () => {
        if (session) await session.close();
      },
    ],
    ["backend", stopBackend],
    [
      "backend process",
      async () => {
        if (backend) await terminate(backend.child);
      },
    ],
    [
      "native fixture",
      async () => {
        if (cleanup) await cleanup();
      },
    ],
  ]) {
    try {
      await stop();
    } catch (error) {
      cleanupErrors.push(new Error(`${label}: ${error.message}`, { cause: error }));
    }
  }
  const logs = join(work, "shell/application/logs");
  for (const name of await readdir(logs).catch(() => [])) {
    if (!name.endsWith(".log")) continue;
    await copyFile(join(logs, name), join(evidence, `desktop-${name}`)).catch((error) => cleanupErrors.push(error));
  }
  try {
    if (process.env.DOKIME_KEEP_EVIDENCE === "1") await cp(work, join(evidence, "fixture"), { recursive: true });
    await rm(work, { recursive: true, force: true, maxRetries: 5, retryDelay: 200 });
  } catch (error) {
    cleanupErrors.push(error);
  }
  process.stdout.write(`Screenshot diagnostics: ${evidence}\n`);
}
// Preserve Ready Scenes Even When A Later Product Operation Fails.
if (entries.length) {
  const previous = JSON.parse(await readFile(join(folder, "manifest.json"), "utf8").catch(() => "[]"));
  const replaced = new Set(entries.map((entry) => entry.image));
  const manifest = [...previous.filter((entry) => !replaced.has(entry.image)), ...entries];
  manifest.sort(
    (a, b) => scenes.indexOf(a.scene) - scenes.indexOf(b.scene) || ["light", "dark"].indexOf(a.theme) - ["light", "dark"].indexOf(b.theme),
  );
  for (const entry of entries) await copyFile(join(evidence, entry.image), join(folder, entry.image));
  await writeFile(join(folder, "manifest.json"), `${JSON.stringify(manifest, null, 2)}\n`);
}
if (failure || cleanupErrors.length)
  throw new AggregateError([...(failure ? [failure] : []), ...cleanupErrors], "Screenshot journey failed");
