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

// Third Party Imports

// Local Imports
import { failureOf, type NotificationFailure } from "@/lib/notification-details";
import { reportsCreate } from "@/lib/host/client";
import { required } from "@/lib/rpc/canonical";
import { commands } from "@/lib/desktop/commands";
import type { ArtifactSource } from "@/lib/contracts/delivery";
import type { SessionIdentity } from "@/lib/rpc/transport";
import { deliverNativeContent } from "./native-delivery";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

export type NativeReportSource = Extract<ArtifactSource, { kind: "cycle_report" | "execution_report" }>;

export interface SavedReport {
  path: string;
  openFailure: NotificationFailure | null;
}

/* === Classes === */

/* === Components === */

/* === Hooks === */

/* === Public Functions === */

/** Save a report without routing its bytes through the WebView, then open the saved file. */
export async function downloadNativeReport(
  session: SessionIdentity,
  source: NativeReportSource,
  filename: string,
): Promise<SavedReport | null> {
  const subject = {
    kind:
      source.kind === "execution_report"
        ? ({ case: "execution", value: source.executionId } as const)
        : ({ case: "cycle", value: source.cycleId } as const),
  };
  // A sealed execution's report was frozen when it sealed; Create returns it (or freezes it once for older executions).
  const prepared = await reportsCreate(session, { subject, documentId: "", requestId: crypto.randomUUID() });
  const name =
    source.kind === "execution_report"
      ? required(prepared.report, "Create Report").identifier
      : required(required(prepared.file, "Create Report").metadata, "Create Report File").id;
  const delivered = await deliverNativeContent(session, { source: { ...source, name }, kind: "html_report", filename, action: "save" });
  if (!delivered) return null;
  try {
    await commands.desktopOpenPath(delivered.path);
    return { path: delivered.path, openFailure: null };
  } catch (error) {
    return { path: delivered.path, openFailure: failureOf(error, "The Saved Report Could Not Be Opened") };
  }
}

/* === Private Functions === */

/* === Entry Point === */
