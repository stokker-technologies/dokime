/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Two Native Desktops Share One Verified Remote Backend And Its Evidence.
 */

/* === Imports === */

// Standard Library Imports
import assert from "node:assert/strict";
import { randomUUID, createHash } from "node:crypto";
import { cp, mkdir, mkdtemp, readFile, realpath, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { parseArgs } from "node:util";

// Third Party Imports

// Local Imports
import {
  startBackend,
  startSession,
  stopBackend,
  until,
  unusedPort,
  chooseNativeFile,
  terminate,
  prepareNativeFixture,
  openNativeReport,
} from "./native-fixture.mjs";
import { call, invoke } from "./native-rpc.mjs";

/* === Re-Exports === */

/* === Constants === */

const { values: options } = parseArgs({ options: { profile: { type: "string", default: "dev" }, junit: { type: "string" } } });
const workspace = resolve(dirname(fileURLToPath(import.meta.url)), "../../..");
const target = `${process.platform === "win32" ? "windows" : "linux"}-${process.arch === "x64" ? "x86_64" : "aarch64"}`;
const sessions = [];
const started = performance.now();
const work = await realpath(await mkdtemp(join(tmpdir(), "dokime-two-gui-")));
const reports = join(workspace, "target/tests", options.profile, target, "system-typescript");
await mkdir(reports, { recursive: true });
const evidence = await mkdtemp(join(reports, "two-gui-remote-"));
let cleanup;
let failure;

/* === Types === */

/* === Classes === */

/* === Public Functions === */

/* === Private Functions === */

// Only The Native Picker Is Driven Outside WebDriver; Product Commands Are Unmodified.
async function pick(desktop, command, args, path, operation = "Open") {
  await desktop.execute(
    `
    const pending = {done: false};
    window.__storyPicker = pending;
    window.__TAURI_INTERNALS__.invoke(arguments[0], arguments[1]).then(
      value => Object.assign(pending, {value, done: true}),
      failure => Object.assign(pending, {failure, done: true}),
    );
  `,
    [command, args],
  );
  const picker = { window: null };
  await until(
    `native ${operation} picker`,
    async () => (await desktop.execute("return window.__storyPicker.done;")) || chooseNativeFile(picker, path, operation, desktop.process),
  );
  const result = await until(`native ${command} completion`, async () => {
    const current = await desktop.execute("return window.__storyPicker;");
    return current.done ? current : null;
  });
  await desktop.execute("delete window.__storyPicker;");
  if (result.failure) throw Object.assign(new Error(JSON.stringify(result.failure)), { detail: result.failure });
  return result.value;
}

async function desktop(name, endpoint) {
  const root = join(work, name);
  const output = join(evidence, name);
  await mkdir(root, { recursive: true });
  await mkdir(output, { recursive: true });
  const instance = await startSession(root, endpoint, output, undefined, {
    startLocal: false,
    downloadsRoot: join(root, "downloads"),
  });
  sessions.push(instance);
  return instance;
}

async function deliver(desktop, source, filename, kind, destination) {
  const status = await invoke(desktop, "desktop_session_status");
  const path = destination ?? join(desktop.downloads, filename);
  await mkdir(dirname(path), { recursive: true });
  const saved = await pick(
    desktop,
    "desktop_artifact_deliver",
    {
      generation: status.connection.generation,
      request: { source, kind, filename, action: "save" },
    },
    path,
    "Save",
  );
  assert.equal(await readFile(saved.path).then((bytes) => bytes.length > 0), true, "Verified delivery writes nonempty evidence");
  return saved;
}

function xml(value) {
  return String(value).replace(
    /[&<>"']/g,
    (character) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&apos;" })[character],
  );
}

/* === Entry Point === */

try {
  process.stdout.write(`Evidence: ${evidence}\n`);
  cleanup = await prepareNativeFixture(work, evidence);
  const dataRoot = join(work, "tampered", "shell", "application", "data");
  const backend = await startBackend(work, evidence, undefined, "backend", true, { dataRoot });
  const a = await desktop("a", backend.endpoint);
  await a.connected();

  // Listener and Pairings Have Independent Library Resource Owners.
  const address = `127.0.0.1:${await unusedPort()}`;
  const listener = await call(a, "harmos.v1.access.Listener", "Get");
  await call(a, "harmos.v1.access.Listener", "Set", {
    value: { enabled: true, endpoint: address, certificateNames: ["localhost", "127.0.0.1"] },
    expectedRevision: listener.revision,
  });
  const b = await desktop("b");
  const imported = [];
  for (const [name, gui] of [
    ["a", a],
    ["b", b],
  ]) {
    const before = await call(a, "harmos.v1.access.Pairings", "List");
    const access = join(work, `client-${name}.pair`);
    assert.equal(await pick(a, "desktop_pairing_create", {}, access, "Save"), access);
    const pairing = JSON.parse(await readFile(access, "utf8"));
    assert.ok(pairing.certificate_pem && pairing.access_key);
    const after = await call(a, "harmos.v1.access.Pairings", "List");
    const admitted = after.pairings.find((item) => !before.pairings.some((prior) => prior.id === item.id));
    assert.ok(admitted, "Each Desktop Receives Its Own Revocable Pairing");
    imported.push({ name: admitted.id, pairing, access });
    await pick(gui, "desktop_backend_import_pairing", { identifier: `story-${name}` }, access);
    await gui.connected();
    const selected = await invoke(gui, "desktop_session_status");
    assert.equal(selected.connection.remote, true);
    assert.equal(selected.connection.backend.backendId, pairing.backend_identifier);
    assert.equal(new URL(selected.connection.endpoint).origin, `https://${address}`);
    const profiles = await invoke(gui, "desktop_backends_list");
    assert.ok(profiles.profiles.some((profile) => profile.identifier === `story-${name}`));
    assert.ok(!profiles.profiles.some((profile) => profile.identifier === `story-${name === "a" ? "b" : "a"}`));
    const storage = await gui.execute("return JSON.stringify([Object.entries(localStorage),Object.entries(sessionStorage)]);");
    assert.ok(!storage.includes(pairing.access_key), "Pairing Credentials Stay in Native Storage");
  }

  // Register Frozen Cases Before Appending Explicit Execution Attempts.
  const catalogA = await call(a, "Suites", "List", {});
  assert.deepEqual((await call(b, "Suites", "List", {})).values, catalogA.values);
  const suite = catalogA.values.find((item) => item.metadata.label === "Thermal");
  assert.ok(suite, "The Shared SDK Fixture Is Admitted");
  const definition = await call(a, "Suites", "Get", { name: suite.metadata.identifier }).then((reply) => reply.value);
  assert.deepEqual(
    (await call(b, "Suites", "Get", { name: suite.metadata.identifier }).then((reply) => reply.value)).templates,
    definition.templates,
  );
  await b.goto("/cycles");
  const title = "Shared Remote Campaign";
  const cycle = await call(a, "Cycles", "Create", { requestId: crypto.randomUUID(), metadata: { label: title } }).then(
    (reply) => reply.cycle,
  );
  await until("Second Desktop Observes New Cycle Without Refresh", async () => (await b.text()).includes(title));
  const plan = await call(a, "Cases", "Create", {
    plan: {
      cycleId: cycle.cycleId,
      selections: [{ suiteId: suite.metadata.identifier, templateId: definition.templates[0].metadata.identifier }],
    },
    requestId: randomUUID(),
  });
  assert.equal(plan.cases.length, 1);
  const executionId = randomUUID();
  await call(a, "Executions", "Create", {
    requestId: crypto.randomUUID(),
    parent: cycle.cycleId,
    executions: [{ executionId, caseId: plan.cases[0].caseId }],
  });
  for (const gui of [a, b]) {
    await gui.goto(`/cycles/${cycle.cycleId}?executionId=${executionId}`, { reload: true });
    await until("Shared Execution Selected", () => gui.find("css selector", `li[data-execution-id="${executionId}"]`));
    await gui.clickCss(`li[data-execution-id="${executionId}"] button[data-tree-select]`);
  }
  await a.clickCss(`li[data-execution-id="${executionId}"] button[data-tree-actions]`);
  await a.clickNamed("menuitem", "Start Execution");
  for (const gui of [a, b])
    await until("Remote Operator Prompt", () => gui.find("css selector", 'ol[aria-label="Pending Prompts"] li[data-prompt-state="open"]'));
  await b.clickNamed("button", "Yes");
  await until(
    "Both Desktops Observe Sealed Result",
    async () => (await a.text()).includes("Passed") && (await b.text()).includes("Passed"),
  );
  const detailA = await call(a, "Executions", "Get", { name: executionId }).then((reply) => reply.execution);
  const detailB = await call(b, "Executions", "Get", { name: executionId }).then((reply) => reply.execution);
  assert.equal(detailA.summary.state, "EXECUTION_STATE_SEALED");
  assert.deepEqual(detailB, detailA);
  assert.equal(detailA.outcome.evaluated.status, "RESULT_STATUS_PASSED");
  const recordsA = await call(a, "dokime.client.v1.evidence.Records", "List", { selection: { scope: { executionId } } });
  const samplesA = await call(a, "dokime.client.v1.evidence.Samples", "List", { scope: { executionId } });
  assert.ok(recordsA.records.some((item) => item.payload));
  assert.ok(samplesA.samples.some((item) => item.raw?.payload));
  assert.deepEqual(await call(b, "dokime.client.v1.evidence.Records", "List", { selection: { scope: { executionId } } }), recordsA);
  assert.deepEqual(await call(b, "dokime.client.v1.evidence.Samples", "List", { scope: { executionId } }), samplesA);

  // Each Native Owner Saves and Reopens the Same Verified Sealed Report.
  const subject = { execution: executionId };
  const preview = await call(a, "Reports", "Create", { subject, requestId: crypto.randomUUID() });
  await call(a, "Reports", "Publish", {
    subject,
    name: preview.file.metadata.id,
    expectedRevision: preview.file.sha256,
    requestId: crypto.randomUUID(),
  });
  const hashes = [],
    archives = [];
  for (const [name, gui] of [
    ["a", a],
    ["b", b],
  ]) {
    const report = await deliver(gui, { kind: "execution_report", executionId }, "execution.html", "html_report");
    assert.equal(report.origin, "sealed");
    const html = await readFile(report.path);
    await mkdir(join(evidence, name), { recursive: true });
    await writeFile(join(evidence, name, "execution.html"), html);
    assert.ok(html.toString("utf8").includes("Thermal"));
    hashes.push(createHash("sha256").update(html).digest("hex"));
    await openNativeReport(report.path);
    const archive = await deliver(gui, { kind: "execution_evidence", executionId }, "execution.evidence", "artifact");
    archives.push(
      createHash("sha256")
        .update(await readFile(archive.path))
        .digest("hex"),
    );
    await cp(gui.downloads, join(evidence, name, "downloads"), { recursive: true });
  }
  assert.equal(hashes[0], hashes[1]);
  assert.equal(archives[0], archives[1]);

  // A Third Native Owner Refuses Identity Substitution and Occupied Storage.
  const c = await desktop("tampered");
  const tampered = join(work, "tampered.pair");
  await writeFile(tampered, JSON.stringify({ ...imported[0].pairing, backend_identifier: randomUUID() }));
  await assert.rejects(
    pick(c, "desktop_backend_import_pairing", { identifier: "tampered" }, tampered),
    (error) => error.detail.kind === "identity_changed",
  );
  assert.equal((await invoke(c, "desktop_session_status")).connection.backend, null);
  await c.clickNamed("button", "Start Local Backend");
  await until("Occupied Storage Recovery", async () => /in use by another host/.test(await c.text()));
  await assert.rejects(
    invoke(c, "desktop_session_select", { selection: { kind: "owned_local" } }),
    (error) => error.detail.kind === "backend_running",
  );
  assert.ok(await c.exists("button", "Start Local Backend"));

  // Revocation Removes Only One Pairing and Preserves the Other Client.
  await call(a, "harmos.v1.access.Pairings", "Delete", { name: imported[1].name });
  await assert.rejects(call(b, "Suites", "List", {}), (error) => error.detail.kind === "grpc" && error.detail.detail.code === 16);
  assert.deepEqual((await call(a, "Suites", "List", {})).values, catalogA.values);
  assert.ok(await b.execute("return Boolean(document.body && window.__TAURI_INTERNALS__);"));

  // Certificate Rotation Refuses the Old Trust Pin and a New Pairing Recovers.
  await call(a, "harmos.v1.access.Certificates", "Create", {});
  await assert.rejects(invoke(a, "desktop_backend_use", { identifier: "story-a" }));
  await invoke(a, "desktop_session_select", {
    selection: {
      kind: "attach",
      request: {
        target: { kind: "local", endpoint: backend.endpoint },
        expectedBackendId: null,
      },
    },
  });
  const replacement = join(work, "replacement.pair");
  await pick(a, "desktop_pairing_create", {}, replacement, "Save");
  await pick(a, "desktop_backend_import_pairing", { identifier: "replacement" }, replacement);
  await a.connected();
  assert.equal(
    (await call(a, "Executions", "Get", { name: executionId }).then((reply) => reply.execution)).summary.state,
    "EXECUTION_STATE_SEALED",
  );
} catch (error) {
  failure = error;
  for (const [index, gui] of sessions.entries()) {
    await writeFile(join(evidence, `failure-${index}.txt`), `${error.stack}\n${await gui.text().catch(() => "WebView unavailable")}`).catch(
      () => {},
    );
    await gui.screenshot(`failure-${index}`).catch(() => {});
  }
} finally {
  const cleanups = [
    ...sessions.toReversed().map((gui) => async () => {
      try {
        await gui.close();
      } finally {
        await terminate(gui.process);
      }
    }),
    stopBackend,
    async () => cleanup?.(),
  ];
  for (const close of cleanups) {
    try {
      await close();
    } catch (error) {
      failure ??= error;
    }
  }
  const seconds = (performance.now() - started) / 1000;
  if (options.junit) {
    await mkdir(dirname(resolve(options.junit)), { recursive: true });
    const outcome = failure ? `<failure message="${xml(failure.message)}">${xml(failure.stack)}</failure>` : "";
    await writeFile(
      options.junit,
      `<?xml version="1.0"?><testsuites><testsuite name="Native Desktop" tests="1" failures="${failure ? 1 : 0}"><testcase classname="dokime.native.${process.platform}" name="Two GUI Remote" time="${seconds}">${outcome}</testcase></testsuite></testsuites>`,
    );
  }
  process.stdout.write(`Two GUI Remote: ${failure ? "FAILED" : "passed"} (${seconds.toFixed(2)}s)\nEvidence: ${evidence}\n`);
  if (failure) {
    process.stderr.write(`${failure.stack}\n`);
    process.exitCode = 1;
  } else await rm(work, { recursive: true, force: true });
}
