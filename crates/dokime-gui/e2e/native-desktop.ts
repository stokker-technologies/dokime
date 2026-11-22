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
import { cp, mkdir, mkdtemp, readFile, rm, stat, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { parseArgs } from "node:util";

import type { Session } from "./native-fixture.ts";

import {
  answerPendingPrompts,
  chooseNativeFile,
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
} from "./native-fixture.ts";
import { type Reply, call, invoke } from "./native-rpc.ts";
import { printReportSaveTimings, saveReport } from "./report-save.ts";

/* === Re-Exports === */

export * from "./native-fixture.ts";

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
const sessions: Session[] = [];
const results: { name: string; seconds: number; error?: unknown }[] = [];
let cleanup: (() => Promise<void>) | undefined;
let evidence!: string;
let work!: string;
let desktop: Session;
let cycle: Reply<"Cycles", "Get">["cycle"];
let executionId: string;

/* === Public Functions === */

/* === Private Functions === */

async function story(name: string, run: () => Promise<void>, required = false) {
  const started = performance.now();
  try {
    await run();
    results.push({ name, seconds: (performance.now() - started) / 1000 });
    process.stdout.write(`${name}: Passed\n`);
  } catch (error) {
    results.push({ name, seconds: (performance.now() - started) / 1000, error });
    process.stderr.write(`${name}: ${(error as Error).stack ?? error}\n`);
    await desktop?.screenshot(name).catch(() => {});
    await writeFile(join(evidence, `${name}.txt`), `${(error as Error).stack}\n${await desktop?.text().catch(() => "")}`).catch(() => {});
    if (required) {
      throw error;
    }
  }
}

async function close(instance: Session) {
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
  const identity = await call(local, "dokime.host.v1.Status", "Get");
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
    desktop.execute<string>('return document.querySelector("li[data-execution-id]")?.dataset.executionId;'),
  );
  cycle = await call(desktop, "Cycles", "Get", { name: cycleId }).then((reply) => reply.cycle);
  const detail = await call(desktop, "Executions", "Get", { name: executionId }).then((reply) => reply.execution);
  assert.equal(detail.summary.state, "EXECUTION_STATE_OPEN");
  assert.equal(detail.controls.canStart, true);
  await desktop.clickCss(`li[data-execution-id="${executionId}"] button[data-tree-select]`);
}

async function execute() {
  await until("Planned Execution Page Start", () =>
    desktop.find("css selector", '[aria-label="Execution #1 Controls"] button[data-execution-action="Start"]:enabled'),
  );
  await desktop.clickCss('[aria-label="Execution #1 Controls"] button[data-execution-action="Start"]');
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
  await until("Registered Bench Execution Row", () => desktop.find("css selector", 'button[aria-label="Execution Actions"]'));
  await desktop.clickCss('button[aria-label="Execution Actions"]');
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
    desktop.execute<{ exact: Record<string, string>; shown: Record<string, string> }>(`
    const tiles = [...document.querySelectorAll('[data-document-metric]')];
    if (tiles.length < 11 || tiles.some(tile => tile.dataset.documentValuePresent !== 'true')) {
      return null;
    }
    const values = tiles.map(tile => [tile.dataset.documentMetric, tile.querySelector('strong [data-exact-value]')?.dataset.exactValue]);
    // Sealed evidence is complete; the view shows it once the settled run's document refresh lands,
    // until then tiles may hold a capture taken before the samples were saved.
    const captured = values.every(([, value]) => value && value !== '-') && /[1-9]/.test(Object.fromEntries(values).count ?? '');
    return captured ? {exact: Object.fromEntries(values), shown: Object.fromEntries(tiles.map(tile => [tile.dataset.documentMetric, tile.querySelector('strong [data-exact-value]')?.textContent]))} : null;`),
  );
  const base = 1152921504606847975n;
  for (const name of ["mean", "median", "min", "max", "first", "last"]) {
    assert.equal(values.exact[name], String(base));
    assert.equal(values.shown[name], String(base));
  }
  for (const name of ["p05", "p95", "p99"]) {
    assert.equal(values.exact[name], String(BigInt(Number(base))));
  }
  const count = BigInt(values.exact.count);
  assert.ok(count > 0n);
  assert.equal(values.exact.sum, String(base * count));
  assert.equal(values.shown.sum, String(base * count));
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
  const suite = await call(desktop, "Suites", "Get", { name: "thermal.thermal" }).then((reply) => reply.value);
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

async function cycleSeal() {
  // Create a Cycle Whose Executions Have Never Started.
  const suite = await call(desktop, "Suites", "Get", { name: "thermal.thermal" }).then((reply) => reply.value);
  const planned = await call(desktop, "Cases", "Create", {
    requestId: crypto.randomUUID(),
    plan: { selections: [{ suiteId: suite.metadata.identifier, templateId: suite.templates[0].metadata.identifier }] },
    validateOnly: true,
  });
  const cycle = await call(desktop, "Cycles", "Create", {
    requestId: crypto.randomUUID(),
    origins: [planned.cases[0].origin, planned.cases[0].origin],
    continueAutomatically: false,
  }).then((reply) => reply.cycle);
  await desktop.goto(`/cycles/${cycle.cycleId}`, { reload: true });

  // Seal From the Cycle's Own Menu and Await Its Completion.
  await desktop.clickNamed("button", "Cycle Actions");
  await desktop.clickNamed("menuitem", "Seal Cycle");
  await until("Seal Confirmation", () => desktop.find("css selector", "[role=dialog]"));
  await desktop.execute(
    `const button = [...document.querySelectorAll('[role=dialog] button')].find(button => button.textContent.trim() === 'Seal Cycle');
     if (!button || button.disabled) {
       throw new Error('Seal Confirmation Unavailable');
     }
     button.click();`,
  );
  const retained = await until("Sealed Cycle With Retained Files", async () => {
    const current = await call(desktop, "Cycles", "Get", { name: cycle.cycleId }).then((reply) => reply.cycle);
    if (current.phase === "CYCLE_PHASE_SEAL_FAILED") {
      throw new Error(current.sealFailure);
    }
    return current.sealed ? current : null;
  });
  assert.ok(retained.reportFile);
  assert.ok(retained.evidenceFile);
  const executions = await call(desktop, "Executions", "List", { parent: cycle.cycleId });
  assert.equal(executions.executions.length, 2);
  for (const execution of executions.executions) {
    assert.equal(execution.outcome.notEvaluated.termination, "TERMINATION_NOT_RUN");
  }
  await until("Not-Run Rows Render", () =>
    desktop.execute("return document.querySelectorAll('[data-execution-status=not_run]').length === 2;"),
  );

  // Download Both Retained Files Twice Through the Native Save Picker.
  for (const item of [
    { menu: "Download Report", extension: "html" },
    { menu: "Export Evidence", extension: "zip" },
  ]) {
    const digests = [];
    for (const attempt of [1, 2]) {
      const path = join(work, `sealed-cycle-${attempt}.${item.extension}`);
      await desktop.clickNamed("button", "Cycle Actions");
      await desktop.clickNamed("menuitem", item.menu);
      const picker = { window: null };
      await Promise.all([
        until("Cycle Save Picker", () => chooseNativeFile(picker, path, "Save", desktop.process)),
        until("Retained Cycle File Saved", async () => (await stat(path)).size > 0),
      ]);
      const bytes = await readFile(path);
      digests.push(createHash("sha256").update(bytes).digest("hex"));
      if (item.extension === "zip") {
        assert.ok(bytes.includes(Buffer.from("report.html")));
      }
    }
    assert.equal(digests[0], digests[1]);
  }
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
    ["engine", "Streams"],
  ]) {
    await desktop.goto(`/settings/${section}`);
    await until(`Settings ${section}`, async () => (await desktop.text()).includes(label));
    assert.equal(await desktop.execute('return Boolean(document.querySelector("[data-error-boundary]"));'), false);
    if (["plugins", "secrets", "backends"].includes(section)) {
      const table = await desktop.execute<{
        heading: boolean;
        filter: boolean;
        footer: boolean;
        columns: number;
        failureWidth: boolean;
      } | null>(`const frame = document.querySelector('[data-slot="data-table"]');
        if (!frame) return null;
        return {
          heading: Boolean(frame.querySelector('h2')),
          filter: Boolean(frame.querySelector('input')),
          footer: Boolean(frame.querySelector('footer [data-testid="table-page-size"]')),
          columns: frame.querySelectorAll('thead th').length,
          failureWidth: [...frame.querySelectorAll('tbody td[colspan]')].every(cell => cell.colSpan === frame.querySelectorAll('thead th').length),
        };`);
      assert.ok(table);
      assert.ok(table.heading && table.filter && table.footer && table.columns > 0 && table.failureWidth);
    }
  }
  await desktop.goto("/settings/plugins");
  const inventory = await call(desktop, "dokime.host.v1.Plugins", "List");
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
  const archive = await call(desktop, "dokime.host.v1.status.Diagnostics", "Get", { maxBytes: "1024" });
  assert.ok(BigInt(archive.totalLength) > 0n);
  const chunks = [Buffer.from(archive.data, "base64")];
  let length = chunks[0].length;
  while (BigInt(length) < BigInt(archive.totalLength)) {
    const page = await call(desktop, "dokime.host.v1.status.Diagnostics", "Get", {
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

function xml(value: unknown) {
  return String(value).replace(
    /[&<>"']/g,
    (character) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&apos;" })[character]!,
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
      const identity = await call(desktop, "dokime.host.v1.Status", "Get");
      assert.ok(identity.status.identity.hostId);
    });
    if (options.workflow === "execution") {
      await story("Catalog and Registered Cases", library, true);
      await story("Execution Progress Prompts and Evidence", execute);
      await story("Exact Bench Statistics", exactBench);
      await story("Documents Reports and History", reportsAndHistory);
      await story("Cycle Seal and Retained Downloads", cycleSeal);
      await story("Execution Cancellation", cancellation);
    }
    if (["configuration", "remote", "execution"].includes(options.workflow))
      await story("Settings Plugins Secrets and Diagnostics", settings);
    if (["appearance", "execution"].includes(options.workflow)) {
      await story("Appearance and Touch", appearance);
    }
  }
} catch (error) {
  process.stderr.write(`${(error as Error).stack}\n`);
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
  if (process.env.DOKIME_KEEP_EVIDENCE === "1") {
    const roots = [...new Set(sessions.map((instance) => instance.applicationRoot))];
    for (const [index, root] of roots.entries()) {
      try {
        await cp(join(root, "logs"), join(evidence, `desktop-${index}-logs`), { recursive: true });
      } catch (error) {
        results.push({ name: "Retain Desktop Diagnostics", seconds: 0, error });
      }
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
              (result.error
                ? `<failure message="${xml((result.error as Error).message)}">${xml((result.error as Error).stack)}</failure>`
                : "") +
              "</testcase>",
          )
          .join("") +
        "</testsuite></testsuites>",
    );
  }
  if (failures) {
    process.exitCode = 1;
  } else if (work) {
    await rm(work, { recursive: true, force: true });
  }
  process.stdout.write(`Native Desktop: ${results.length - failures} Passed; ${failures} Failed\n`);
}
