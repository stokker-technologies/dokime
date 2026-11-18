/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Submit and Observe the Desktop Report Save.
 */

/* === Imports === */

// Standard Library Imports
import { readFile, readdir, stat } from "node:fs/promises";
import { join } from "node:path";

// Third Party Imports

// Local Imports
import { chooseNativeFile, until } from "./native-fixture.mjs";
import { call } from "./native-rpc.mjs";

/* === Re-Exports === */

/* === Constants === */

const saves = [];

/* === Types === */

/* === Classes === */

/* === Public Functions === */

export async function saveReport(desktop, path, { label, executionId }) {
  const started = performance.now();
  // Submit the Same Save Action an Operator Uses.
  await desktop.clickNamed("button", "Execution Actions");
  const menuOpened = performance.now();
  await desktop.clickNamed("menuitem", "Download Report");
  const submitted = performance.now();

  // Observe the Picker and Published File Concurrently.
  const picker = { window: null };
  const [pickerCompleted, fileCompleted] = await Promise.all([
    until("Report Save Picker", () => chooseNativeFile(picker, path, "Save", desktop.process)).then(() => performance.now()),
    until("Saved Report File", async () => (await stat(path)).size > 0).then(() => performance.now()),
  ]);
  const milliseconds = fileCompleted - started;
  const observation = {
    menu_ms: menuOpened - started,
    submit_ms: submitted - menuOpened,
    save_wait_ms: fileCompleted - submitted,
    picker_observation_ms: pickerCompleted - submitted,
    picker_observation_tail_ms: Math.max(0, pickerCompleted - fileCompleted),
  };
  let creationMs = null;
  const events = [];
  try {
    let at;
    let pageToken = "";
    do {
      const logs = await call(desktop, "harmos.host.v1.status.Logs", "List", {
        ...(at ? { at } : {}),
        page: { pageSize: 4096, maxBytes: "3145728", pageToken },
      });
      at = logs.resolvedAt;
      pageToken = logs.nextPageToken;
      for (const event of logs.events) {
        if (event.message === "Report Created") {
          creationMs = Number(event.fields.creation_ms);
        }
        if (["Report Created", "Report Published", "Report Stage", "Report Plot"].includes(event.message)) {
          events.push(event);
        }
      }
    } while (pageToken);
    const firstStage = events.findLastIndex((event) => event.message === "Report Stage" && event.fields.phase === "resolve_cut");
    if (firstStage > 0) events.splice(0, firstStage);
    await desktop.dump("report-save-host-timings", events);
  } catch (error) {
    process.stdout.write(`Report Creation Timing Unavailable: ${error.message}\n`);
  }

  // Inventory the Complete Source After the Timed Save, Without Warming Its Reads.
  const streams = {};
  let at;
  let pageToken = "";
  do {
    const page = await call(desktop, "dokime.host.v1.evidence.Samples", "List", {
      scope: { executionId },
      ...(at ? { at } : {}),
      page: { pageSize: 4096, maxBytes: "3145728", pageToken },
    });
    for (const sample of page.samples) streams[sample.raw.stream] = (streams[sample.raw.stream] ?? 0) + 1;
    at = page.resolvedAt;
    pageToken = page.nextPageToken;
  } while (pageToken);
  const records = await call(desktop, "dokime.host.v1.evidence.Records", "List", { selection: { scope: { executionId } } });
  const inventory = { streams, samples: Object.values(streams).reduce((sum, count) => sum + count, 0), records: records.count };
  saves.push({
    label,
    creationMs,
    observation,
    inventory,
    events,
    applicationRoot: desktop.applicationRoot,
    milliseconds,
    dump: desktop.dump,
  });
  return { path };
}

/** Print Measurements After the Owned Processes Flush Their Logs. */
export async function printReportSaveTimings() {
  for (const save of saves) {
    try {
      const directory = join(save.applicationRoot, "logs");
      const paths = (await readdir(directory)).filter((name) => name.endsWith(".log")).sort();
      const logs = (await Promise.all(paths.map((name) => readFile(join(directory, name), "utf8")))).join("\n");
      const delivery =
        logs
          .split("\n")
          .filter((line) => line.includes("Content Delivered") && line.includes("file=prepared-report/"))
          .at(-1) ?? "";
      const published =
        logs
          .split("\n")
          .filter((line) => line.includes("Verified File Saved"))
          .at(-1) ?? "";
      const picker =
        logs
          .split("\n")
          .filter((line) => line.includes("Save Destination Selected"))
          .at(-1) ?? "";
      const milliseconds = (line, key) => Number(line.match(new RegExp(`\\b${key}=([0-9.]+)`))?.[1] ?? NaN);
      const calls = logs
        .split("\n")
        .filter((line) => line.includes("Report RPC Completed"))
        .map((line) => ({
          operation: line.match(/operation=([^\s]+)/)?.[1],
          admission_ms: milliseconds(line, "admission_ms"),
          request_ms: milliseconds(line, "request_ms"),
        }));
      const timing = {
        label: save.label,
        total_ms: save.milliseconds,
        host_creation_ms: save.creationMs,
        host_publication_ms: Number(save.events.findLast((event) => event.message === "Report Published")?.fields.publication_ms ?? 0),
        native_picker_ms: milliseconds(picker, "picker_ms"),
        transfer_ms: milliseconds(delivery, "request_ms") + milliseconds(delivery, "transfer_ms"),
        write_ms: milliseconds(delivery, "write_ms") + milliseconds(published, "write_publish_ms"),
      };
      timing.remainder_ms = timing.total_ms - timing.host_creation_ms - timing.transfer_ms - timing.write_ms;
      timing.observation_ms = timing.remainder_ms - timing.host_publication_ms - timing.native_picker_ms;
      timing.native_admission_ms = calls.length ? calls.reduce((sum, call) => sum + call.admission_ms, 0) : null;
      timing.native_rpc_overhead_ms = calls.length
        ? calls.reduce((sum, call) => sum + call.request_ms, 0) - timing.host_creation_ms - timing.host_publication_ms
        : null;
      process.stdout.write(`Report Save Timing: ${JSON.stringify(timing)}\n`);
      process.stdout.write(`Report Save Observation: ${JSON.stringify({ label: save.label, ...save.observation })}\n`);
      process.stdout.write(`Report RPC Timing: ${JSON.stringify({ label: save.label, calls })}\n`);
      process.stdout.write(`Report Source Inventory: ${JSON.stringify({ label: save.label, ...save.inventory })}\n`);
      for (const event of save.events) process.stdout.write(`Report Host Timing: ${JSON.stringify(event)}\n`);
      await save.dump("report-save-timings", {
        timing,
        observation: save.observation,
        calls,
        inventory: save.inventory,
        events: save.events,
        picker,
        delivery,
        published,
      });
    } catch (error) {
      process.stdout.write(
        `Report Save Timing: ${save.label} total_ms=${save.milliseconds.toFixed(3)}; phase logs unavailable: ${error.message}\n`,
      );
    }
  }
}

/* === Private Functions === */

/* === Entry Point === */
