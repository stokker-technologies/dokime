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
const target = `${process.platform === "win32" ? "windows" : "linux"}-${process.arch === "x64" ? "x86_64" : "aarch64"}`;
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

async function story(name, run, required = false) {
  const started = performance.now();
  try {
    await run();
    results.push({ name, seconds: (performance.now() - started) / 1000 });
    process.stdout.write(`${name}: Passed\n`);
  } catch (error) {
    results.push({ name, seconds: (performance.now() - started) / 1000, error });
    process.stderr.write(`${name}: ${error.stack ?? error}\n`);
    await desktop?.screenshot(name).catch(() => {});
    await writeFile(join(evidence, `${name}.txt`), `${error.stack}\n${await desktop?.text().catch(() => "")}`).catch(() => {});
    if (required) throw error;
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
  const saved = await call(local, "Cycles", "Create", {
    requestId: crypto.randomUUID(),
    metadata: { label: "Preserved Local Cycle" },
  }).then((reply) => reply.cycle);
  await close(local);
  const restarted = await startSession(root, undefined, evidence);
  sessions.push(restarted);
  desktop = restarted;
  await restarted.connected();
  const restored = await call(restarted, "Cycles", "Get", { name: saved.cycleId }).then((reply) => reply.cycle);
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
  await until("Cycle Workspace", async () => (await desktop.location()).pathname === `/cycles/${cycleId}`);
  executionId = await until("Registered Execution Row", () =>
    desktop.execute('return document.querySelector("li[data-execution-id]")?.dataset.executionId;'),
  );
  cycle = await call(desktop, "Cycles", "Get", { name: cycleId }).then((reply) => reply.cycle);
  const detail = await call(desktop, "Executions", "Get", { name: executionId }).then((reply) => reply.execution);
  assert.equal(detail.summary.state, "EXECUTION_STATE_OPEN");
  await desktop.clickCss(`li[data-execution-id="${executionId}"] button[data-tree-select]`);
}

async function execute() {
  await desktop.clickCss(`li[data-execution-id="${executionId}"] button[data-tree-actions]`);
  await desktop.clickNamed("menuitem", "Start Execution");
  await until("Live Operator Prompt", () => desktop.find("css selector", 'ol[aria-label="Pending Prompts"] li[data-prompt-state="open"]'));
  assert.ok((await desktop.text()).includes("Door"));
  await answerPendingPrompts(desktop);
  const detail = await until("Sealed Execution", async () => {
    const current = await call(desktop, "Executions", "Get", { name: executionId }).then((reply) => reply.execution);
    return current.summary.state === "EXECUTION_STATE_SEALED" ? current : null;
  });
  assert.equal(detail.outcome.evaluated.status, "RESULT_STATUS_PASSED");
  assert.ok(detail.seal);
  await until("Passed Result in WebView", async () => (await desktop.text()).includes("Passed"));
  const records = await call(desktop, "dokime.client.v1.evidence.Records", "List", { selection: { scope: { executionId } } });
  const samples = await call(desktop, "dokime.client.v1.evidence.Samples", "List", { scope: { executionId } });
  assert.ok(records.records.some((record) => record.payload));
  assert.ok(samples.samples.some((sample) => sample.raw?.payload));
  await until("Captured Record Rows in the Authored Document", () =>
    desktop.execute('return Boolean(document.querySelector("[data-testid=record-rows] li[data-window-row]"));'),
  );
  const values = await until("Exact Summary Tiles", () =>
    desktop.execute(`
    const tiles = [...document.querySelectorAll('[data-document-metric]')];
    return tiles.length >= 11 ? Object.fromEntries(tiles.map(tile => [tile.dataset.documentMetric, tile.querySelector('strong')?.textContent])) : null;`),
  );
  const base = 1152921504606847975n;
  for (const name of ["mean", "median", "min", "max", "first", "last", "p05", "p95", "p99"]) assert.equal(values[name], String(base));
  const count = BigInt(values.count.replace(/[^0-9]/g, ""));
  assert.ok(count > 0n);
  assert.equal(values.sum, String(base * count));
}

async function reportsAndHistory() {
  const subject = { execution: executionId };
  const preview = await call(desktop, "Reports", "Create", { subject, requestId: crypto.randomUUID() });
  await call(desktop, "Reports", "Publish", {
    subject,
    name: preview.file.metadata.id,
    expectedRevision: preview.file.sha256,
    requestId: crypto.randomUUID(),
  });
  const status = await invoke(desktop, "desktop_session_status");
  const path = join(work, "thermal-execution.html");
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
  await writeFile(join(evidence, "thermal-execution.html"), html);
  assert.ok(html.includes(executionId), "The Saved Report Identifies the Frozen Execution");
  await openNativeReport(saved.value.path);
  await desktop.goto("/history");
  await until("History Lists Sealed Execution", async () => (await desktop.text()).includes("Thermal"));
  await desktop.goto(`/cycles/${cycle.cycleId}?executionId=${executionId}`, { reload: true });
  await until("Saved Execution Reopens", async () => (await desktop.text()).includes("Passed"));
}

async function cancellation() {
  const suites = await call(desktop, "Suites", "List", {});
  const suite = await call(desktop, "Suites", "Get", { name: suites.values[0].metadata.identifier }).then((reply) => reply.value);
  const plan = await call(desktop, "Cases", "Create", {
    requestId: crypto.randomUUID(),
    plan: { selections: [{ suiteId: suite.metadata.identifier, templateId: suite.templates[0].metadata.identifier }] },
    validateOnly: true,
  });
  const pending = await call(desktop, "Cycles", "Create", {
    requestId: crypto.randomUUID(),
    origins: plan.cases.map((item) => item.origin),
    continueAutomatically: true,
  }).then((reply) => reply.cycle);
  const executionId = (await call(desktop, "Executions", "List", { parent: pending.cycleId })).executions[0].summary.executionId;
  await desktop.goto(`/cycles/${pending.cycleId}?executionId=${executionId}`, { reload: true });
  await until(
    "Cancellation Prompt",
    async () => (await call(desktop, "Prompts", "List", { pendingOnly: true, executionId })).prompts.length,
  );
  await call(desktop, "Executions", "Cancel", { requestId: crypto.randomUUID(), name: executionId });
  const detail = await until("Cancelled Execution Sealed", async () => {
    const current = await call(desktop, "Executions", "Get", { name: executionId }).then((reply) => reply.execution);
    return current.summary.state === "EXECUTION_STATE_SEALED" ? current : null;
  });
  assert.equal(detail.outcome.notEvaluated.termination, "TERMINATION_CANCELLED");
  await until("WebView Shows Stopped Execution", async () => (await desktop.text()).includes("Stopped"));
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
    await desktop.goto(`/settings/${section}`);
    await until(`Settings ${section}`, async () => (await desktop.text()).includes(label));
    assert.equal(await desktop.execute('return Boolean(document.querySelector("[data-error-boundary]"));'), false);
  }
  await desktop.goto("/settings/plugins");
  const installed = await call(desktop, "Plugins", "ListInstallations", { pageSize: 100 });
  const admitted = await call(desktop, "Plugins", "ListAdmittedPlugins", { pageSize: 100 });
  const rowIds = installed.installations.map((entry) => entry.installId);
  for (const entry of admitted.plugins) {
    const plugin = entry.plugin;
    if (plugin.remote) rowIds.push(JSON.stringify(["remote", null, null, null, plugin.entry]));
    else if (plugin.managed && !rowIds.includes(plugin.entry)) rowIds.push(`bundle:${plugin.managed.bundle}`);
  }
  assert.ok(rowIds.length > 0);
  await until("Admitted and Installed Plugin Rows", () =>
    desktop.execute(
      'return arguments[0].every(id => [...document.querySelectorAll("[data-row-id]")].some(row => row.dataset.rowId === id));',
      [rowIds],
    ),
  );
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
  process.stdout.write(`Evidence: ${evidence}\n`);
  await story("Owned Startup and Restart", owned, true);
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
      await story("Catalog and Registered Cases", library, true);
      await story("Execution Progress Prompts and Evidence", execute);
      await story("Documents Reports and History", reportsAndHistory);
      await story("Execution Cancellation", cancellation);
    }
    if (["configuration", "remote", "execution"].includes(options.workflow))
      await story("Settings Plugins Secrets and Diagnostics", settings);
    if (["appearance", "execution"].includes(options.workflow)) await story("Appearance and Touch", appearance);
  }
} catch (error) {
  process.stderr.write(`${error.stack}\n`);
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
              (result.error ? `<failure message="${xml(result.error.message)}">${xml(result.error.stack)}</failure>` : "") +
              "</testcase>",
          )
          .join("") +
        "</testsuite></testsuites>",
    );
  }
  if (failures) process.exitCode = 1;
  else if (work) await rm(work, { recursive: true, force: true });
  process.stdout.write(`Native Desktop: ${results.length - failures} Passed; ${failures} Failed\n`);
}
