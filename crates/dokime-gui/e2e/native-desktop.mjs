/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/** Native Desktop Journeys Through the Execution Host. */

/* === Imports === */

import assert from "node:assert/strict";
import { randomUUID } from "node:crypto";
import { mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { parseArgs } from "node:util";

import {
  answerPendingPrompts,
  chooseNativeFile,
  openNativeReport,
  prepareNativeFixture,
  setDensity,
  sizeViewport,
  startBackend,
  startSession,
  stopBackend,
  terminate,
  until,
} from "./native-fixture.mjs";
import { call, invoke } from "./native-rpc.mjs";

/* === Re-Exports === */

export * from "./native-fixture.mjs";

/* === Constants === */

const { values: options } = parseArgs({
  options: {
    profile: { type: "string", default: "dev" },
    junit: { type: "string" },
    workflow: { type: "string", default: "execution" },
    release: { type: "boolean" },
    scene: { type: "string" },
  },
});
const workspace = resolve(dirname(fileURLToPath(import.meta.url)), "../../..");
const target = (process.platform === "win32" ? "windows" : "linux") + "-" + (process.arch === "x64" ? "x86_64" : "aarch64");
const reports = join(workspace, "target/tests", options.profile, target, "system-typescript");
const sessions = [];
const results = [];
let cleanup;
let evidence;
let work;
let desktop;
let cycle;
let executionId;

/* === Public Functions === */

/* === Private Functions === */

async function story(name, run) {
  const started = performance.now();
  try {
    await run();
    results.push({ name, seconds: (performance.now() - started) / 1000 });
    process.stdout.write(name + ": Passed\n");
  } catch (error) {
    results.push({ name, seconds: (performance.now() - started) / 1000, error });
    await desktop?.screenshot(name).catch(() => {});
    await writeFile(join(evidence, name + ".txt"), error.stack + "\n" + (await desktop?.text().catch(() => ""))).catch(() => {});
    throw error;
  }
}

async function close(instance) {
  try {
    await instance.close();
  } finally {
    await terminate(instance.process);
  }
}

async function owned() {
  const root = join(work, "owned");
  await mkdir(root, { recursive: true });
  const local = await startSession(root, undefined, evidence, undefined, { startLocal: false });
  sessions.push(local);
  desktop = local;
  await until("Local Host Startup Control", () => local.exists("button", "Start Local Backend"));
  await local.clickNamed("button", "Start Local Backend");
  await local.connected();
  const status = await invoke(local, "desktop_session_status");
  assert.equal(status.ownership, "owned_local");
  assert.equal(status.connection.phase, "connected");
  const identity = await call(local, "harmos.v1.Status", "Get");
  assert.ok(identity.status.identity.hostId);
  const saved = await call(local, "Execution", "CreateCycle", { metadata: { label: "Preserved Local Cycle" } });
  await close(local);
  const restarted = await startSession(root, undefined, evidence);
  sessions.push(restarted);
  desktop = restarted;
  await restarted.connected();
  const restored = await call(restarted, "Execution", "GetCycle", { cycleId: saved.cycleId });
  assert.equal(restored.metadata.label, "Preserved Local Cycle");
  await close(restarted);
}

async function library() {
  await desktop.goto("/library");
  await until("SDK Suite in Library", async () => (await desktop.text()).includes("Thermal"));
  await desktop.clickNamed("button", "New Cycle");
  await until("New Cycle Title", () => desktop.find("css selector", "#new-cycle-title"));
  await desktop.type(await desktop.find("css selector", "#new-cycle-title"), "Desktop Thermal Journey");
  await desktop.clickNamed("button", "Create Cycle");
  const cycleId = await until("Created Active Cycle", async () => (await desktop.location()).searchParams.get("appendTo"));
  await until("New Cycle Dialog Closed", () => desktop.execute("return !document.querySelector('[role=dialog]');"));
  await desktop.clickCss('details:has(table[aria-label="Thermal"]) > summary');
  await desktop.clickNamed("button", "Thermal Cycle");
  await until("Template Parameters", () => desktop.find("css selector", '[data-testid="parameter-input-limit"]'));
  const input = await desktop.find("css selector", '[data-testid="parameter-input-limit"]');
  await desktop.type(input, "90");
  await until("Edited Parameter", () =>
    desktop.execute('return document.querySelector("[data-testid=parameter-row-limit]")?.dataset.edited === "true";'),
  );
  await desktop.clickNamed("button", "Next");
  await until("Template Summary", () => desktop.find("css selector", '[data-testid="template-add"]'));
  assert.ok((await desktop.text()).includes("Target Cycle"));
  await desktop.clickCss('[data-testid="template-add"]');
  await until("Template Added", () => desktop.execute("return !document.querySelector('[role=dialog]');"));
  await desktop.clickNamed("button", "Open Cycle");
  await until("Cycle Workspace", async () => (await desktop.location()).pathname === "/cycles/" + cycleId);
  executionId = await until("Registered Execution Row", () =>
    desktop.execute('return document.querySelector("li[data-execution-id]")?.dataset.executionId;'),
  );
  cycle = await call(desktop, "Execution", "GetCycle", { cycleId });
  const detail = await call(desktop, "Execution", "GetExecution", { executionId });
  assert.equal(detail.summary.state, "EXECUTION_STATE_OPEN");
  await desktop.clickCss('li[data-execution-id="' + executionId + '"] button[data-tree-select]');
}

async function execute() {
  await desktop.clickCss('li[data-execution-id="' + executionId + '"] button[data-tree-actions]');
  await desktop.clickNamed("menuitem", "Start Execution");
  await until("Live Operator Prompt", () => desktop.find("css selector", 'ol[aria-label="Pending Prompts"] li[data-prompt-state="open"]'));
  assert.ok((await desktop.text()).includes("Door"));
  await answerPendingPrompts(desktop);
  const detail = await until("Sealed Execution", async () => {
    const current = await call(desktop, "Execution", "GetExecution", { executionId });
    return current.summary.state === "EXECUTION_STATE_SEALED" ? current : null;
  });
  assert.equal(detail.outcome.evaluated.status, "RESULT_STATUS_PASSED");
  assert.ok(detail.seal);
  await until("Passed Result in WebView", async () => (await desktop.text()).includes("Passed"));
  const records = await call(desktop, "Evidence", "ListRecords", { selection: { scope: { executionId } } });
  const samples = await call(desktop, "Telemetry", "ListSamples", { scope: { executionId } });
  assert.ok(records.records.some((record) => record.payload));
  assert.ok(samples.samples.some((sample) => sample.payload));
  assert.ok(await desktop.execute('return Boolean(document.querySelector("[data-execution-section=Records]"));'));
}

async function reportsAndHistory() {
  const status = await invoke(desktop, "desktop_session_status");
  const path = join(evidence, "thermal-execution.html");
  await desktop.execute(
    `const pending = {done:false}; window.__storyPicker=pending;
    window.__TAURI_INTERNALS__.invoke("desktop_artifact_deliver",arguments[0]).then(
      value=>Object.assign(pending,{done:true,value}), failure=>Object.assign(pending,{done:true,failure}));`,
    [
      {
        generation: status.connection.generation,
        request: {
          source: { kind: "execution_report", executionId },
          kind: "html_report",
          filename: "thermal-execution.html",
          action: "save",
        },
      },
    ],
  );
  const picker = { window: null };
  await until(
    "Save Execution Report",
    async () => (await desktop.execute("return window.__storyPicker.done")) || chooseNativeFile(picker, path, "Save", desktop.process),
  );
  const saved = await until("Verified Execution Report", async () => {
    const current = await desktop.execute("return window.__storyPicker");
    return current.done ? current : null;
  });
  assert.equal(saved.failure, undefined, JSON.stringify(saved.failure));
  assert.equal(saved.value.origin, "sealed");
  const html = await readFile(saved.value.path, "utf8");
  assert.ok(html.includes("Thermal"), "The Saved Report Contains Frozen Execution Content");
  await openNativeReport(saved.value.path);
  await desktop.goto("/history");
  await until("History Lists Sealed Execution", async () => (await desktop.text()).includes("Thermal"));
  await desktop.goto("/cycles/" + cycle.cycleId + "?executionId=" + executionId, { reload: true });
  await until("Saved Execution Reopens", async () => (await desktop.text()).includes("Passed"));
}

async function cancellation() {
  const suites = await call(desktop, "Catalog", "ListSuites");
  const suite = await call(desktop, "Catalog", "GetSuite", { suiteId: suites.suites[0].suiteId });
  const plan = await call(desktop, "Planning", "MaterializeTemplate", {
    selection: { suiteId: suite.metadata.identifier, templateId: suite.templates[0].metadata.identifier },
  });
  const pending = await call(desktop, "Execution", "CreateCycle", {
    origins: plan.cases.map((item) => item.origin),
    continueAutomatically: true,
  });
  const executionId = pending.executionIds[0];
  await desktop.goto("/cycles/" + pending.cycleId + "?executionId=" + executionId, { reload: true });
  await until("Cancellation Prompt", async () => (await call(desktop, "Prompts", "ListPending", { executionId })).prompts.length);
  await call(desktop, "Execution", "CancelExecution", { executionId });
  const detail = await until("Cancelled Execution Sealed", async () => {
    const current = await call(desktop, "Execution", "GetExecution", { executionId });
    return current.summary.state === "EXECUTION_STATE_SEALED" ? current : null;
  });
  assert.ok(detail.outcome.terminated);
  await until("WebView Shows Termination", async () => /Cancel|Terminat/.test(await desktop.text()));
}

async function settings() {
  for (const [section, label] of [
    ["appearance", "Density"],
    ["plugins", "Plugins"],
    ["secrets", "Secret"],
    ["remote", "Remote"],
    ["diagnostics", "Diagnostics"],
    ["about", "Dokime"],
    ["backends", "Backends"],
  ]) {
    await desktop.goto("/settings/" + section);
    await until("Settings " + section, async () => (await desktop.text()).includes(label));
    assert.equal(await desktop.execute('return Boolean(document.querySelector("[data-error-boundary]"));'), false);
  }
  await desktop.goto("/settings/plugins");
  assert.ok((await desktop.text()).includes("Thermal"));
  await desktop.goto("/settings/diagnostics");
  assert.ok(/recovery|preserv|unavailable|not.*support/i.test(await desktop.text()));
}

async function appearance() {
  await setDensity(desktop, { density: "touch", width: 1920, height: 1200 });
  await desktop.goto("/library");
  assert.equal(await desktop.execute("return document.documentElement.dataset.density"), "touch");
  await setDensity(desktop, { density: "comfortable", width: 1600, height: 1000 });
}

function xml(value) {
  return String(value).replace(
    /[&<>"']/g,
    (character) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&apos;" })[character],
  );
}

/* === Entry Point === */

try {
  await mkdir(reports, { recursive: true });
  evidence = await mkdtemp(join(reports, "native-desktop-"));
  work = await mkdtemp(join(tmpdir(), "dokime-native-desktop-"));
  cleanup = await prepareNativeFixture(work, evidence);
  process.stdout.write("Evidence: " + evidence + "\n");
  await story("Owned Startup and Restart", owned);
  if (options.workflow !== "ownership") {
    const backend = await startBackend(work, evidence);
    desktop = await startSession(join(work, "attached"), backend.endpoint, evidence);
    sessions.push(desktop);
    await story("Attach to Host", async () => {
      await desktop.connected();
      await sizeViewport(desktop, { width: 1600, height: 1000 });
      assert.equal((await invoke(desktop, "desktop_session_status")).ownership, "attached");
      const identity = await call(desktop, "harmos.v1.Status", "Get");
      assert.ok(identity.status.identity.hostId);
    });
    if (options.workflow === "execution") {
      await story("Catalog and Registered Cases", library);
      await story("Execution Progress Prompts and Evidence", execute);
      await story("Documents Reports and History", reportsAndHistory);
      await story("Execution Cancellation", cancellation);
    }
    if (["configuration", "remote", "execution"].includes(options.workflow))
      await story("Settings Plugins Secrets and Diagnostics", settings);
    if (["appearance", "execution"].includes(options.workflow)) await story("Appearance and Touch", appearance);
  }
} catch (error) {
  process.stderr.write(error.stack + "\n");
  process.exitCode = 1;
  if (!results.some((result) => result.error)) results.push({ name: "Fixture Setup", seconds: 0, error });
} finally {
  for (const instance of sessions.toReversed()) {
    if (!instance.closed)
      await close(instance).catch((error) => {
        results.push({ name: "Close Desktop", seconds: 0, error });
      });
  }
  for (const operation of [stopBackend, async () => cleanup?.()]) {
    try {
      await operation();
    } catch (error) {
      results.push({ name: "Fixture Cleanup", seconds: 0, error });
    }
  }
  const failures = results.filter((result) => result.error).length;
  if (options.junit) {
    await mkdir(dirname(resolve(options.junit)), { recursive: true });
    await writeFile(
      options.junit,
      '<?xml version="1.0"?><testsuites><testsuite name="Native Desktop" tests="' +
        results.length +
        '" failures="' +
        failures +
        '">' +
        results
          .map(
            (result) =>
              '<testcase classname="dokime.native.' +
              process.platform +
              '" name="' +
              xml(result.name) +
              '" time="' +
              result.seconds +
              '">' +
              (result.error ? '<failure message="' + xml(result.error.message) + '">' + xml(result.error.stack) + "</failure>" : "") +
              "</testcase>",
          )
          .join("") +
        "</testsuite></testsuites>",
    );
  }
  if (failures) process.exitCode = 1;
  else if (work) await rm(work, { recursive: true, force: true });
  process.stdout.write("Native Desktop: " + (results.length - failures) + " Passed; " + failures + " Failed\n");
}
