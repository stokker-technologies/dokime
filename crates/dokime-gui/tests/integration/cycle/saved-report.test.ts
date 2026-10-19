/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

// Standard Library Imports
import assert from "node:assert/strict";

// Third Party Imports
import { test, vi } from "vitest";

// Local Imports

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Public Functions === */

/* === Private Functions === */

/* === Entry Point === */

test("Saved Report Browser", async () => {
  const opened: string[] = [];
  let refuseOpen = false;
  const saved: string[] = [];
  vi.resetModules();
  vi.doMock("@/lib/desktop-open", async (original) => ({ ...(await original<object>()), isTauriDesktop: () => true }));
  vi.doMock("@/lib/desktop/commands", () => ({
    commands: {
      desktopSaveDocument: async (_generation: string, document: { filename: string }) => {
        saved.push(document.filename);
        return document.filename === "cancelled.html" ? null : `/home/operator/${document.filename}`;
      },
      desktopOpenPath: async (path: string) => {
        if (refuseOpen) throw new Error("no default browser");
        opened.push(path);
        return null;
      },
    },
  }));
  try {
    const { downloadNativeReport } = await import("@/lib/api/native-reports");
    const session = { generation: "generation-1", isCurrent: () => true } as never;
    const resource = { report: { identifier: "cycle" }, bytes: new TextEncoder().encode("<html></html>"), html: "" } as never;

    // The Saved Path Is The One The Browser Opens
    assert.deepEqual(await downloadNativeReport(resource, "bench.html", session), { path: "/home/operator/bench.html", openFailure: null });
    assert.deepEqual(opened, ["/home/operator/bench.html"]);

    // A Cancelled Save Opens Nothing
    assert.equal(await downloadNativeReport(resource, "cancelled.html", session), null);
    assert.deepEqual(opened, ["/home/operator/bench.html"]);

    // A Failed Open Keeps The Save Successful And Says Why
    refuseOpen = true;
    assert.deepEqual(await downloadNativeReport(resource, "night.html", session), {
      path: "/home/operator/night.html",
      openFailure: "no default browser",
    });
    assert.deepEqual(saved, ["bench.html", "cancelled.html", "night.html"]);
  } finally {
    vi.doUnmock("@/lib/desktop-open");
    vi.doUnmock("@/lib/desktop/commands");
    vi.resetModules();
  }
});
