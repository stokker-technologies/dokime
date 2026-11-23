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
import { reportsCreate, reportsPublish } from "@/lib/host/client";
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
  const prepared = await reportsCreate(session, { subject, documentId: "", requestId: crypto.randomUUID() });
  const file = required(prepared.file, "Create Report");
  let name = required(file.metadata, "Create Report File").id;
  if (source.kind === "execution_report") {
    const published = await reportsPublish(session, { subject, name, expectedRevision: file.sha256, requestId: crypto.randomUUID() });
    name = required(published.report, "Publish Report").identifier;
  }
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
