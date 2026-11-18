/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Python Drives a Paired Desktop.
 */

/* === Imports === */

// Standard Library Imports
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { mkdir, readFile, readdir, stat, writeFile } from "node:fs/promises";
import { join } from "node:path";

// Third Party Imports

// Local Imports
import { chooseNativeFile, startSession, terminate, until, unusedPort } from "./native-fixture.mjs";
import { call, invoke } from "./native-rpc.mjs";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Public Functions === */

export async function pythonDrivesPairedDesktop({ workspace, work, evidence, profile }) {
  const root = join(work, "python-paired");
  const output = join(evidence, "python-paired");
  await mkdir(output, { recursive: true });
  let desktop;
  let python;
  let transcript = "";
  try {
    // Open the Desktop With Its Owned Local Backend.
    desktop = await startSession(root, undefined, output);
    await desktop.connected();
    const connected = await invoke(desktop, "desktop_session_status");
    assert.equal(connected.ownership, "owned_local");

    // Serve a Same-PC Listener Before Using the Real Export Control.
    const listener = await call(desktop, "harmos.host.v1.access.Listener", "Get");
    await call(desktop, "harmos.host.v1.access.Listener", "Set", {
      value: { enabled: true, endpoint: `127.0.0.1:${await unusedPort()}`, certificateNames: ["localhost", "127.0.0.1"] },
      expectedRevision: listener.revision,
    });
    await desktop.goto("/settings/remote");
    await desktop.clickNamed("button", "Export Remote Access");
    const access = join(root, "python.pair");
    const picker = { window: null };
    await until("Export Remote Access Picker", () => chooseNativeFile(picker, access, "Save", desktop.process));
    await until("Exported Pairing File", async () => (await stat(access)).size > 0);
    const paired = JSON.parse(await readFile(access, "utf8"));
    const status = await call(desktop, "harmos.host.v1.Status", "Get");
    assert.equal(paired.backend_identifier, status.status.identity.hostId);

    // Install This Candidate's Built SDK Through the Same Adapter as Python Stories.
    const wheels = await readdir(join(workspace, "target/build", profile, "any/python"));
    assert.equal(wheels.filter((name) => name.endsWith(".whl")).length, 1, "Prepare the Candidate SDK With mise run build:python");
    const installer = spawn(
      "uv",
      [
        "run",
        "--no-sync",
        "--project",
        "tools/python",
        "python",
        "-m",
        "dokime_tools.test.archive",
        "--profile",
        profile,
        "--install-wheels",
      ],
      {
        cwd: workspace,
        stdio: ["ignore", "inherit", "inherit"],
        windowsHide: true,
        detached: process.platform !== "win32",
        env: { ...process.env, UV_PROJECT_ENVIRONMENT: undefined },
      },
    );
    python = installer;
    let installerError;
    installer.on("error", (error) => {
      installerError = error;
    });
    await until("Working-Tree Python SDK Installation", () => {
      if (installerError) {
        throw installerError;
      }
      if (installer.exitCode === null) {
        return false;
      }
      assert.equal(installer.exitCode, 0);
      return true;
    });

    // Keep the Cycle List Open While the Unmodified Starter Creates Its Work.
    await desktop.goto("/cycles");
    const before = (await call(desktop, "Cycles", "List")).cycles;
    const executable = join(workspace, ".venv", process.platform === "win32" ? "Scripts/python.exe" : "bin/python");
    python = spawn(executable, ["-u", "sdk/python/examples/test/main.py", "--host", access], {
      cwd: workspace,
      stdio: ["pipe", "pipe", "pipe"],
      windowsHide: true,
      detached: process.platform !== "win32",
      env: { ...process.env, UV_PROJECT_ENVIRONMENT: undefined },
    });
    let startupError;
    python.on("error", (error) => {
      startupError = error;
    });
    python.stdout.on("data", (chunk) => {
      transcript += chunk.toString();
    });
    python.stderr.on("data", (chunk) => {
      transcript += chunk.toString();
    });
    const cycle = await until("Python Cycle Registration", async () =>
      (await call(desktop, "Cycles", "List")).cycles.find((row) => !before.some((prior) => prior.cycleId === row.cycleId)),
    );
    await until("Desktop Shows the Python Cycle Without Refresh", () => desktop.find("css selector", `tr[data-row-id="${cycle.cycleId}"]`));
    await desktop.clickCss(`tr[data-row-id="${cycle.cycleId}"]`);
    const executionId = await until("Desktop Shows the Python Execution", () =>
      desktop.execute("return document.querySelector('li[data-execution-id]')?.dataset.executionId"),
    );
    await desktop.clickCss(`li[data-execution-id="${executionId}"] button[data-tree-select]`);
    await until("Desktop Presents the Operator Question", () =>
      desktop.find("css selector", 'ol[aria-label="Pending Prompts"] li[data-prompt-state="open"]'),
    );
    const pending = (await call(desktop, "Prompts", "List", { pendingOnly: true, executionId })).prompts;
    assert.equal(pending.length, 1);

    // Answer Once Through the Starter's Existing Stdin Flow.
    await until("Starter Requests Its Answer", () => {
      if (startupError) {
        throw startupError;
      }
      assert.equal(python.exitCode, null, transcript);
      return transcript.includes("Choose an Action:");
    });
    python.stdin.end(`${pending[0].prompt.definition.actions[0].identifier}\n`);
    const detail = await until("Python Execution Ends With Its Criterion Result", async () => {
      const current = (await call(desktop, "Executions", "Get", { name: executionId })).execution;
      return current.summary.state === "EXECUTION_STATE_SEALED" ? current : null;
    });
    assert.equal(detail.outcome.evaluated.status, "RESULT_STATUS_PASSED");
    const steps = await call(desktop, "Executions", "Steps", { name: executionId });
    const criteria = steps.steps.flatMap((step) => step.criteria);
    assert.equal(criteria.length, 1);
    assert.equal(criteria[0].status, "CRITERION_STATUS_PASSED");
    await until("Desktop Shows the Completed Criterion", () =>
      desktop.find(
        "css selector",
        `section[aria-label="Criteria"] li[data-record-id="${criteria[0].criterionId}"][data-record-status="passed"]`,
      ),
    );
    const remaining = await call(desktop, "Prompts", "List", { pendingOnly: true, executionId });
    assert.equal(remaining.prompts.length, 0);
    await until("Starter Exits Successfully", () => {
      if (python.exitCode === null) {
        return false;
      }
      assert.equal(python.exitCode, 0, transcript);
      return true;
    });
    await desktop.screenshot("python-completed");
  } catch (error) {
    await desktop?.screenshot("python-failure").catch(() => {});
    await writeFile(join(output, "failure.txt"), `${error.stack}\n${await desktop?.text().catch(() => "")}`);
    throw error;
  } finally {
    await writeFile(join(output, "starter.log"), transcript);
    await terminate(python);
    try {
      await desktop?.close();
    } finally {
      await terminate(desktop?.process);
    }
  }
}

/* === Private Functions === */

/* === Entry Point === */
