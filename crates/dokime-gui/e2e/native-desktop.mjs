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
import { createHash } from "node:crypto";
import { mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { parseArgs } from "node:util";

import {
  answerPendingPrompts,
  obcExecutable,
  openNativeReport,
  pluginExecutable,
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
import { printReportSaveTimings, saveReport } from "./report-save.mjs";

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
    if (required) {
      throw error;
    }
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
  const identity = await call(local, "harmos.host.v1.Status", "Get");
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
  const records = await call(desktop, "dokime.host.v1.evidence.Records", "List", { selection: { scope: { executionId } } });
  const samples = await call(desktop, "dokime.host.v1.evidence.Samples", "List", { scope: { executionId } });
  assert.ok(records.records.some((record) => record.payload));
  assert.ok(samples.samples.some((sample) => sample.raw?.payload));
  await until("Captured Record Rows in the Authored Document", () =>
    desktop.execute('return Boolean(document.querySelector("[data-testid=record-rows] li[data-window-row]"));'),
  );
  await until("Captured Plot Data in the Authored Document", () =>
    desktop.execute(`
      const plot = document.querySelector('[data-plot-viewport]');
      plot?.scrollIntoView({block: 'center'});
      return Boolean(plot?.querySelector('.plot-card canvas'));`),
  );
}

async function exactBench() {
  // Run the OBC Bench Template, Whose Meter Register Exceeds Every Float's Integer Precision.
  const plan = await call(desktop, "Cases", "Create", {
    requestId: crypto.randomUUID(),
    plan: { selections: [{ suiteId: "obc.benchmark", templateId: "obc.benchmark.exact_values" }] },
    validateOnly: false,
  });
  const bench = await call(desktop, "Cycles", "Create", {
    requestId: crypto.randomUUID(),
    metadata: { label: "Desktop Bench Journey" },
    continueAutomatically: false,
  }).then((reply) => reply.cycle);
  const benchExecution = crypto.randomUUID();
  await call(desktop, "Executions", "Create", {
    requestId: crypto.randomUUID(),
    parent: bench.cycleId,
    executions: [{ executionId: benchExecution, caseId: plan.cases[0].caseId }],
  });
  await desktop.goto(`/cycles/${bench.cycleId}?executionId=${benchExecution}`, { reload: true });
  await until("Registered Bench Execution Row", () =>
    desktop.find("css selector", `li[data-execution-id="${benchExecution}"] button[data-tree-actions]`),
  );
  await desktop.clickCss(`li[data-execution-id="${benchExecution}"] button[data-tree-actions]`);
  await desktop.clickNamed("menuitem", "Start Execution");
  const prompt = await until("Bench Confirmation Prompt", async () => {
    const pending = await call(desktop, "Prompts", "List", { pendingOnly: true, executionId: benchExecution });
    return pending.prompts[0]?.prompt.promptId;
  });
  await call(desktop, "dokime.host.v1.prompts.Answers", "Create", {
    requestId: crypto.randomUUID(),
    parent: prompt,
    action: "yes",
    accepted: true,
  });
  const detail = await until("Sealed Bench Execution", async () => {
    const current = await call(desktop, "Executions", "Get", { name: benchExecution }).then((reply) => reply.execution);
    return current.summary.state === "EXECUTION_STATE_SEALED" ? current : null;
  });
  assert.equal(detail.outcome.evaluated.status, "RESULT_STATUS_PASSED");
  const values = await until("Exact Summary Tiles", () =>
    desktop.execute(`
    const tiles = [...document.querySelectorAll('[data-document-metric]')];
    if (tiles.length < 11 || tiles.some(tile => tile.dataset.documentValuePresent !== 'true')) {
      return null;
    }
    const values = tiles.map(tile => [tile.dataset.documentMetric, tile.querySelector('strong [data-exact-value]')?.dataset.exactValue]);
    // Sealed evidence is complete; the view shows it once the settled run's document refresh lands,
    // until then tiles may hold a capture taken before the samples were saved.
    const captured = values.every(([, value]) => value && value !== '-') && /[1-9]/.test(Object.fromEntries(values).count ?? '');
    return captured ? Object.fromEntries(values) : null;`),
  );
  const base = 1152921504606847975n;
  for (const name of ["mean", "median", "min", "max", "first", "last"]) {
    assert.equal(values[name], String(base));
  }
  for (const name of ["p05", "p95", "p99"]) {
    assert.equal(values[name], String(BigInt(Number(base))));
  }
  const count = BigInt(values.count);
  assert.ok(count > 0n);
  assert.equal(values.sum, String(base * count));
}

async function reportsAndHistory() {
  const before = await call(desktop, "Executions", "Get", { name: executionId }).then((reply) => reply.execution);
  assert.equal(before.report.state, "REPORT_STATE_PENDING");
  await desktop.goto(`/cycles/${cycle.cycleId}?executionId=${executionId}`, { reload: true });
  const path = join(work, "thermal-execution.html");
  const saved = await saveReport(desktop, path, { label: "Thermal", executionId });
  const after = await call(desktop, "Executions", "Get", { name: executionId }).then((reply) => reply.execution);
  assert.equal(after.report.state, "REPORT_STATE_READY");
  const html = await readFile(saved.path, "utf8");
  await writeFile(join(evidence, "thermal-execution.html"), html);
  const reader = await openNativeReport(saved.path);
  assert.ok(
    reader.headings.some((heading) => heading.includes("Thermal")),
    "The Reader Sees the Authored Subject Name",
  );
  assert.match(reader.outcome, /Passed/);
  assert.ok(
    reader.headings.some((heading) => heading.includes("Report")),
    "The Reader Sees a Document Section",
  );
  assert.ok(reader.points > 0, "The Reader Sees a Plot with Data");
  await desktop.goto("/history");
  await until("History Lists Sealed Execution", async () => (await desktop.text()).includes("Thermal"));
  await desktop.goto(`/cycles/${cycle.cycleId}?executionId=${executionId}`, { reload: true });
  await until("Saved Execution Reopens", async () => (await desktop.text()).includes("Passed"));
}

async function cancellation() {
  const suite = await call(desktop, "Suites", "Get", { name: "thermal" }).then((reply) => reply.value);
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
  const inventory = await call(desktop, "harmos.host.v1.Plugins", "List");
  const rowIds = inventory.entries.map((entry) => entry.name);
  assert.ok(rowIds.length > 0);
  await until("Admitted and Installed Plugin Rows", () =>
    desktop.execute(
      'return arguments[0].every(id => [...document.querySelectorAll("[data-row-id]")].some(row => row.dataset.rowId === id));',
      [rowIds],
    ),
  );
  await desktop.goto("/settings/diagnostics");
  await until("Diagnostics Download Is Available", () =>
    desktop.execute(
      `const button = [...document.querySelectorAll("button")].find(node => node.textContent.trim() === "Download Bundle");
       return Boolean(button && !button.disabled);`,
    ),
  );
  assert.equal(
    await desktop.execute(
      'return [...document.querySelectorAll("button")].some(node => node.textContent.trim() === "Reset Local Storage");',
    ),
    false,
  );
  const archive = await call(desktop, "harmos.host.v1.status.Diagnostics", "Get", { maxBytes: "1024" });
  assert.ok(BigInt(archive.totalLength) > 0n);
  const chunks = [Buffer.from(archive.data, "base64")];
  let length = chunks[0].length;
  while (BigInt(length) < BigInt(archive.totalLength)) {
    const page = await call(desktop, "harmos.host.v1.status.Diagnostics", "Get", {
      name: archive.name,
      offset: String(length),
      maxBytes: "1024",
    });
    assert.equal(page.name, archive.name);
    assert.equal(page.totalLength, archive.totalLength);
    assert.equal(page.sha256, archive.sha256);
    const data = Buffer.from(page.data, "base64");
    assert.ok(data.length > 0 && data.length <= 1024);
    chunks.push(data);
    length += data.length;
  }
  assert.equal(BigInt(length), BigInt(archive.totalLength));
  assert.equal(createHash("sha256").update(Buffer.concat(chunks)).digest("base64"), archive.sha256);
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
    const backend = await startBackend(work, evidence, undefined, "backend", true, { plugins: [pluginExecutable, obcExecutable] });
    desktop = await startSession(join(work, "attached"), backend.endpoint, evidence);
    sessions.push(desktop);
    await story("Attach to Host", async () => {
      await desktop.connected();
      await sizeViewport(desktop, { width: 1600, height: 1000 });
      assert.equal((await invoke(desktop, "desktop_session_status")).ownership, "attached");
      const identity = await call(desktop, "harmos.host.v1.Status", "Get");
      assert.ok(identity.status.identity.hostId);
    });
    if (options.workflow === "execution") {
      await story("Catalog and Registered Cases", library, true);
      await story("Execution Progress Prompts and Evidence", execute);
      await story("Exact Bench Statistics", exactBench);
      await story("Documents Reports and History", reportsAndHistory);
      await story("Execution Cancellation", cancellation);
    }
    if (["configuration", "remote", "execution"].includes(options.workflow))
      await story("Settings Plugins Secrets and Diagnostics", settings);
    if (["appearance", "execution"].includes(options.workflow)) {
      await story("Appearance and Touch", appearance);
    }
  }
} catch (error) {
  process.stderr.write(`${error.stack}\n`);
  process.exitCode = 1;
  if (!results.some((result) => result.error)) {
    results.push({ name: "Fixture Setup", seconds: 0, error });
  }
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
  await printReportSaveTimings();
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
  if (failures) {
    process.exitCode = 1;
  } else if (work) await rm(work, { recursive: true, force: true });
  process.stdout.write(`Native Desktop: ${results.length - failures} Passed; ${failures} Failed\n`);
}
