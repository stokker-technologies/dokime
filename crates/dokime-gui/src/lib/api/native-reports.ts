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
import { commands } from "@/lib/desktop/commands";
import type { ArtifactSource } from "@/lib/desktop/delivery";
import type { SessionIdentity } from "@/lib/rpc/transport";
import { deliverNativeContent } from "./native-delivery";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

export type NativeReportSource = Extract<ArtifactSource, { kind: "cycle_report" | "execution_report" }>;

export interface SavedReport {
  path: string;
  openFailure: string | null;
}

/* === Classes === */

/* === Components === */

/* === Hooks === */

/* === Public Functions === */

/** Pull and verify a report into the native cache before opening it. */
export async function openNativeReport(session: SessionIdentity, source: NativeReportSource): Promise<"Current" | "Sealed" | undefined> {
  const delivered = await deliverNativeContent(session, {
    source,
    kind: "html_report",
    filename: source.kind === "cycle_report" ? `cycle-${source.cycleId}.html` : `execution-${source.executionId}.html`,
    action: "open",
  });
  return delivered?.origin === "current" ? "Current" : delivered?.origin === "sealed" ? "Sealed" : undefined;
}

/** Save a report without routing its bytes through the WebView, then open the saved file. */
export async function downloadNativeReport(
  session: SessionIdentity,
  source: NativeReportSource,
  filename: string,
): Promise<SavedReport | null> {
  const delivered = await deliverNativeContent(session, { source, kind: "html_report", filename, action: "save" });
  if (!delivered) return null;
  try {
    await commands.desktopOpenPath(delivered.path);
    return { path: delivered.path, openFailure: null };
  } catch (error) {
    return { path: delivered.path, openFailure: error instanceof Error ? error.message : String(error) };
  }
}

/* === Private Functions === */

/* === Entry Point === */
