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
      desktopArtifactDeliver: async (_generation: string, document: { filename: string; source: unknown; action: string }) => {
        assert.deepEqual(document.source, { kind: "cycle_report", cycleId: "cycle-1" });
        assert.equal("bytesBase64" in document, false);
        if (document.action === "open") return { path: "/native/cache/report.html", origin: "current" };
        saved.push(document.filename);
        return document.filename === "cancelled.html" ? null : { path: `/home/operator/${document.filename}`, origin: "sealed" };
      },
      desktopOpenPath: async (path: string) => {
        if (refuseOpen) throw new Error("no default browser");
        opened.push(path);
        return null;
      },
    },
  }));
  try {
    const { downloadNativeReport, openNativeReport } = await import("@/lib/api/native-reports");
    const session = { generation: "generation-1", isCurrent: () => true } as never;
    const source = { kind: "cycle_report", cycleId: "cycle-1" } as const;

    // The Saved Path Is The One The Browser Opens
    assert.deepEqual(await downloadNativeReport(session, source, "bench.html"), { path: "/home/operator/bench.html", openFailure: null });
    assert.deepEqual(opened, ["/home/operator/bench.html"]);

    // A Cancelled Save Opens Nothing
    assert.equal(await downloadNativeReport(session, source, "cancelled.html"), null);
    assert.deepEqual(opened, ["/home/operator/bench.html"]);

    // A Failed Open Keeps The Save Successful And Says Why
    refuseOpen = true;
    assert.deepEqual(await downloadNativeReport(session, source, "night.html"), {
      path: "/home/operator/night.html",
      openFailure: "no default browser",
    });
    assert.deepEqual(saved, ["bench.html", "cancelled.html", "night.html"]);

    // Current Report Origin Survives The Native Pull Without An HTML Payload
    assert.equal(await openNativeReport(session, source), "Current");
  } finally {
    vi.doUnmock("@/lib/desktop-open");
    vi.doUnmock("@/lib/desktop/commands");
    vi.resetModules();
  }
});
