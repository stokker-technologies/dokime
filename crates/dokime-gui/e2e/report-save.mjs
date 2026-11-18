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

export async function saveReport(desktop, path, { label }) {
  const started = performance.now();
  // Submit the Same Save Action an Operator Uses.
  await desktop.clickNamed("button", "Execution Actions");
  await desktop.clickNamed("menuitem", "Download Report");

  // Observe the Native Picker Without Holding a WebDriver Script Open.
  const picker = { window: null };
  await until("Report Save Picker", () => chooseNativeFile(picker, path, "Save", desktop.process));

  // The Native Adapter Publishes the Complete Verified File Atomically.
  await until("Saved Report File", async () => (await stat(path)).size > 0);
  const milliseconds = performance.now() - started;
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
          events.push(event);
        }
      }
    } while (pageToken);
    await desktop.dump("report-save-host-timings", events);
  } catch (error) {
    process.stdout.write(`Report Creation Timing Unavailable: ${error.message}\n`);
  }
  saves.push({ label, creationMs, applicationRoot: desktop.applicationRoot, milliseconds, dump: desktop.dump });
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
      const milliseconds = (line, key) => Number(line.match(new RegExp(`\\b${key}=([0-9.]+)`))?.[1] ?? NaN);
      const timing = {
        label: save.label,
        total_ms: save.milliseconds,
        host_creation_ms: save.creationMs,
        transfer_ms: milliseconds(delivery, "request_ms") + milliseconds(delivery, "transfer_ms"),
        write_ms: milliseconds(delivery, "write_ms") + milliseconds(published, "write_publish_ms"),
      };
      process.stdout.write(`Report Save Timing: ${JSON.stringify(timing)}\n`);
      await save.dump("report-save-timings", { timing, delivery, published });
    } catch (error) {
      process.stdout.write(
        `Report Save Timing: ${save.label} total_ms=${save.milliseconds.toFixed(3)}; phase logs unavailable: ${error.message}\n`,
      );
    }
  }
}

/* === Private Functions === */

/* === Entry Point === */
