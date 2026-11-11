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
import { execFile } from "node:child_process";
import { randomUUID, createHash } from "node:crypto";
import { cp, mkdir, mkdtemp, readFile, realpath, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { parseArgs, promisify } from "node:util";

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
} from "./native-desktop.mjs";
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
  await invoke(a, "desktop_session_select", {
    selection: { kind: "attach", request: { target: { kind: "local", endpoint: backend.endpoint }, expectedBackendId: null } },
  });
  await a.goto("/settings/remote", { reload: true });
  await a.connected();

  // Export Through The Same Native Adapter As Settings → Remote Access.
  const address = `127.0.0.1:${await unusedPort()}`;
  await call(a, "Remote", "UpdateListener", {
    settings: {
      enabled: true,
      address,
      tlsEnabled: true,
      authenticationEnabled: true,
      certificateNames: ["localhost", "127.0.0.1"],
    },
  });
  const access = join(work, "shared.pair");
  assert.equal(await pick(a, "desktop_pairing_create", {}, access, "Save"), access);
  const pairing = JSON.parse(await readFile(access, "utf8"));
  assert.ok(pairing.certificate_pem && pairing.access_key, "The exported file carries the native trust anchor and credential");
  // Each Desktop Imports A Complete Pairing Into Its Own Folder.
  const b = await desktop("b");
  for (const [name, gui] of [
    ["a", a],
    ["b", b],
  ]) {
    await pick(gui, "desktop_backend_import_pairing", { identifier: `story-${name}` }, access);
    await gui.connected();
    const selected = await invoke(gui, "desktop_session_status");
    assert.equal(selected.connection.remote, true);
    assert.equal(selected.connection.backend.backendId, pairing.backend_identifier);
    assert.equal(new URL(selected.connection.endpoint).origin, `https://${address}`);
    const profiles = await invoke(gui, "desktop_backends_list");
    assert.ok(profiles.profiles.some((profile) => profile.identifier === `story-${name}`));
    assert.ok(
      !profiles.profiles.some((profile) => profile.identifier === `story-${name === "a" ? "b" : "a"}`),
      "Each desktop persists an independent backend registry",
    );
    const browserStorage = await gui.execute("return JSON.stringify([Object.entries(localStorage), Object.entries(sessionStorage)]);");
    assert.ok(!browserStorage.includes(pairing.access_key), "Imported credentials remain native");
  }

  // Interleave Catalog And Mutation Calls Through Both Native IPC Bridges.
  const catalogA = await call(a, "Catalog", "ListSuites");
  const catalogB = await call(b, "Catalog", "ListSuites");
  assert.deepEqual(catalogB.suites, catalogA.suites);
  const suite = catalogA.suites.find((candidate) => candidate.metadata.label === "Charger Start-Up");
  assert.ok(suite, "The real OBC fixture is registered");
  const definitionA = await call(a, "Catalog", "GetSuite", { suiteId: suite.suiteId });
  const definitionB = await call(b, "Catalog", "GetSuite", { suiteId: suite.suiteId });
  assert.deepEqual(definitionB.templates, definitionA.templates);
  const template = definitionA.templates.find((candidate) => candidate.metadata.label === "Start-Up Sequence");
  assert.ok(template);
  await b.goto("/cycles");
  const title = "Shared Remote Campaign";
  const cycle = await call(a, "Execution", "CreateCycle", { metadata: { label: title } });
  await until("GUI B observes the new cycle without navigation or refresh", async () => (await b.text()).includes(title));
  const plan = await call(a, "Planning", "MaterializeTemplate", {
    selection: {
      suiteId: suite.suiteId,
      templateId: template.templateId,
      parameters: [
        { key: "duration_s", value: { values: [{ nodes: [{ kind: { Number: { field0: 0.004 } } }] }] } },
        { key: "sample_interval_s", value: { values: [{ nodes: [{ kind: { Number: { field0: 0.001 } } }] }] } },
        { key: "realtime", value: { values: [{ nodes: [{ kind: { Bool: { field0: false } } }] }] } },
      ],
    },
  });
  assert.equal(plan.cases.length, 1, "One selection materializes one deterministic case");
  const executionId = randomUUID();
  await call(a, "Execution", "AppendExecutions", { cycleId: cycle.cycleId, executions: [{ executionId, caseId: plan.cases[0].caseId }] });
  for (const gui of [a, b]) {
    await gui.goto("/cycles");
    await gui.clickNamed("button", title);
    await until("shared cycle selected", async () => (await gui.location()).pathname === `/cycles/${cycle.cycleId}`);
    await gui.clickCss(`li[data-execution-id="${executionId}"] button[data-tree-select]`);
    assert.equal((await invoke(gui, "desktop_session_status")).connection.remote, true);
  }
  await call(a, "Execution", "StartExecution", { executionId, mode: { Automatic: {} } });
  await until("GUI B watches execution reach its deterministic operator checkpoint", async () =>
    b.execute(`const panel = document.querySelector('[aria-labelledby="active-prompt-title"]');
      return Boolean(panel && panel.innerText.includes('Inspect Powered Charger'));`),
  );
  const scope = { executionId };
  const records = await call(b, "Evidence", "ListRecords", { selection: { scope } });
  assert.ok(records.records.some((record) => record.recordType === "obc.charger.startup.initialized"));
  const samples = await call(b, "Telemetry", "ListSamples", { scope, streams: ["obc.charger.startup.samples"] });
  await b.dump("live-evidence", { records, samples });
  assert.equal(samples.samples.length, 2, "GUI B reads the two pre-inspection samples from GUI A's execution");
  await b.clickNamed("button", "Stop Execution");
  const stopped = await until("GUI A observes GUI B's saved stop without refresh", async () => {
    if (!/Stopped|Interrupted/.test(await a.text())) return null;
    const snapshot = await call(a, "Execution", "GetExecution", { executionId });
    return snapshot.summary.sealed ? snapshot : null;
  });
  assert.ok("Cancelled" in stopped.summary.status || "Interrupted" in stopped.summary.status);
  await call(b, "Execution", "SealCycle", { cycleId: cycle.cycleId });

  // Both Clients Save, Reopen And Compare The Same Sealed Evidence.
  const reports = [];
  const archives = [];
  for (const [name, gui] of [
    ["a", a],
    ["b", b],
  ]) {
    const report = await deliver(gui, { kind: "execution_report", executionId }, "report.html", "html_report");
    assert.equal(report.origin, "sealed");
    const html = await readFile(report.path, "utf8");
    const data = JSON.parse(html.split('<script id="hub-report-data" type="application/json">')[1].split("</script>")[0]);
    assert.deepEqual(
      data.test_executions.map((execution) => execution.test_execution_id),
      [executionId],
    );
    for (const artifact of data.snapshot.artifacts) {
      assert.ok(artifact.href.split("/").every((part) => part && part !== "." && part !== ".." && !/[\\:]/.test(part)));
      const saved = await deliver(
        gui,
        { kind: "artifact", artifactId: artifact.artifact_id },
        artifact.path.split("/").at(-1),
        "artifact",
        join(dirname(report.path), artifact.href),
      );
      assert.equal((await readFile(saved.path)).length, artifact.size_bytes);
    }
    await openNativeReport(report.path);
    reports.push(createHash("sha256").update(html).digest("hex"));
    const archive = await deliver(gui, { kind: "execution_evidence", executionId }, "evidence.zip", "artifact");
    const inspected = await promisify(execFile)(
      "python",
      [
        "-c",
        "import hashlib,json,sys,zipfile\nz=zipfile.ZipFile(sys.argv[1])\nassert z.testzip() is None\nmanifest=json.loads(z.read('manifest.json'))\nfor payload in manifest['payloads']:\n data=z.read(payload['path'])\n assert len(data)==payload['size_bytes']\n assert hashlib.sha256(data).hexdigest()==payload['sha256']\nprint(json.dumps(manifest))",
        archive.path,
      ],
      { windowsHide: true },
    );
    const manifest = JSON.parse(inspected.stdout);
    assert.ok(JSON.stringify(manifest.subject).includes(executionId), "Reopened evidence manifest names the shared execution");
    assert.equal(manifest.complete, true, "The evidence export contains its closed frontier");
    assert.equal(manifest.executions.length, 1, "The execution export contains exactly the shared execution");
    const bundledReport = manifest.payloads.find((payload) => payload.path === `reports/execution-${executionId}.html`);
    assert.equal(bundledReport?.sha256, reports.at(-1), "Evidence reopens the same report downloaded separately");
    archives.push(manifest.executions.map(({ revision, counts }) => ({ revision, counts })));
    await cp(gui.downloads, join(evidence, name, "downloads"), { recursive: true });
    await writeFile(join(evidence, `${name}-records.json`), JSON.stringify(records));
  }
  assert.equal(reports[0], reports[1], "Both clients receive the same sealed report");
  assert.deepEqual(archives[0], archives[1], "Both clients receive the same sealed checkpoints and evidence counts");
  // A Third Desktop Refuses A Tampered Identity Before Ordinary Commands.
  const c = await desktop("tampered");
  const tampered = join(work, "tampered.pair");
  await writeFile(tampered, JSON.stringify({ ...pairing, backend_identifier: randomUUID() }));
  await assert.rejects(
    pick(c, "desktop_backend_import_pairing", { identifier: "tampered" }, tampered),
    (error) => error.detail.kind === "identity_changed",
  );
  const refused = await invoke(c, "desktop_session_status");
  assert.equal(refused.connection.phase, "failed");
  assert.equal(refused.connection.backend, null, "An unverified backend never becomes callable");
  await until("GUI shows identity refusal", async () => /identity.*match|identity.*changed/i.test(await c.text()));

  // Starting A Second Backend Against The Owned Root Returns The Typed Error.
  await c.clickNamed("button", "Start Local Backend");
  await until("GUI preserves controls and shows occupied storage", async () => /in use by another instance/.test(await c.text()));
  let occupied;
  await assert.rejects(invoke(c, "desktop_session_select", { selection: { kind: "owned_local" } }), (error) => {
    occupied = error.detail;
    return occupied.kind === "backend_running";
  });
  assert.equal(await realpath(occupied.root), await realpath(dataRoot));
  assert.ok(await c.exists("button", "Start Local Backend"));
  assert.equal((await call(a, "Execution", "GetExecution", { executionId })).summary.sealed, true);

  // Rotating The Backend Key Invalidates The Shared Pairing For Both Clients.
  await call(a, "Remote", "CreatePairing", { label: "Replacement" });
  for (const gui of [a, b]) {
    await assert.rejects(call(gui, "Catalog", "ListSuites"), (error) => error.detail.kind === "grpc" && error.detail.detail.code === 16);
    await gui.clickNamed("button", "Library");
    await until("GUI renders rotated credential failure", async () =>
      /invalid credentials|unauthenticated|valid access token/i.test(await gui.text()),
    );
    assert.ok(
      await gui.execute("return Boolean(document.body && window.__TAURI_INTERNALS__);"),
      "Key rotation leaves a responsive WebView",
    );
  }
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
