/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Typed report operations for the original routed UI. */

/* === External Imports === */

/* === Local Imports === */

import type { Row8 as Metadata, Row3 as ReportFrame } from "@generated/dokime/v1/ReportsMessages/DownloadCycleReport/messages_pb";
import { isTauriDesktop } from "@/lib/desktop-open";
import type { Wire } from "@/lib/rpc/canonical";
import * as rpc from "@/lib/rpc/generated/client";
import type { NativeStream, SessionIdentity } from "@/lib/rpc/transport";
import { NATIVE_HTML_DOCUMENT_BYTES, openNativeDocument, saveNativeDocument } from "./native-delivery";
import { NativeApiLimitError, NativeApiUnsupportedError } from "./native-values";

/* === Canonical Types === */

type ReportHtmlFrame = Wire<ReportFrame>;
type ReportMetadata = Wire<Metadata>;

/* === Limits and public types === */

/** Downloaded report bodies remain bounded even when the browser can render larger documents. */
export const NATIVE_REPORT_HTML_BYTES = NATIVE_HTML_DOCUMENT_BYTES;

export interface NativeReportSession extends SessionIdentity {
  /** Keep report streams attached to the selected connection generation. */
  own(dispose: () => void): () => void;
}

/** Which report a stream must deliver. */
type ReportSubject = { cycleId: string; identifier: string } | { executionId: string; identifier: string };

const CYCLE_REPORT = "cycle.summary";
const EXECUTION_REPORT = "execution.report";

export interface NativeReportHtml {
  report: ReportMetadata;
  bytes: Uint8Array;
  html: string;
}

function assertActive(session: SessionIdentity, signal: AbortSignal | undefined, operation: string): void {
  if (signal?.aborted) throw new Error(`${operation} was cancelled.`);
  if (!session.isCurrent()) throw new Error(`${operation} lost its owning connection.`);
}

/* === Verified On-Demand Report Stream === */

/** The report a cycle has right now: sealed, regenerated, or current. */
export function downloadNativeCycleReport(session: NativeReportSession, cycleId: string, signal?: AbortSignal): Promise<NativeReportHtml> {
  if (!cycleId.trim()) throw new Error("Reports.DownloadCycleReport requires a cycle identifier.");
  return streamReport(
    session,
    "Reports.DownloadCycleReport",
    () => rpc.reportsDownloadCycleReport(session, { cycleId, identifier: CYCLE_REPORT }),
    { cycleId, identifier: CYCLE_REPORT },
    signal,
  );
}

/** The report one execution has right now: sealed, regenerated, or current. */
export function downloadNativeExecutionReport(
  session: NativeReportSession,
  executionId: string,
  signal?: AbortSignal,
): Promise<NativeReportHtml> {
  if (!executionId.trim()) {
    throw new Error("Reports.DownloadExecutionReport requires an execution identifier.");
  }
  return streamReport(
    session,
    "Reports.DownloadExecutionReport",
    async () =>
      (await rpc.reportsDownloadExecutionReport(session, {
        executionId,
      })) as unknown as NativeStream<ReportHtmlFrame>,
    { executionId, identifier: EXECUTION_REPORT },
    signal,
  );
}

async function streamReport(
  session: NativeReportSession,
  operation: string,
  open: () => Promise<NativeStream<ReportHtmlFrame>>,
  subject: ReportSubject,
  signal?: AbortSignal,
): Promise<NativeReportHtml> {
  assertActive(session, signal, operation);
  let reader: NativeStream<ReportHtmlFrame> | undefined;
  let release: (() => void) | undefined;
  let cancelPromise: Promise<void> | undefined;
  const cancelReader = async (): Promise<void> => {
    if (!reader) return;
    cancelPromise ??= reader.cancel();
    await cancelPromise;
  };
  const abort = () => void cancelReader().catch(() => undefined);
  signal?.addEventListener("abort", abort, { once: true });
  try {
    reader = await open();
    assertActive(session, signal, operation);
    release = session.own(() => void cancelReader().catch(() => undefined));
    return await assembleReportHtml(reader, operation, subject, () => !signal?.aborted && session.isCurrent());
  } finally {
    signal?.removeEventListener("abort", abort);
    await cancelReader().catch(() => undefined);
    release?.();
  }
}

async function assembleReportHtml(
  reader: NativeStream<ReportHtmlFrame>,
  operation: string,
  subject: ReportSubject,
  active: () => boolean,
): Promise<NativeReportHtml> {
  let report: ReportMetadata | undefined;
  let expectedOffset = 0n;
  let completionSize: bigint | undefined;
  let completionSha: string | undefined;
  let complete = false;
  const chunks: Uint8Array[] = [];
  let total = 0n;
  for await (const frame of reader) {
    if (!active()) throw new Error(`${operation} was cancelled.`);
    if (complete) throw new Error(`${operation} continued after completion.`);
    const payload = frame.payload?.HarmosVariant;
    if (!payload?.case || !payload.value.field0) throw new Error(`${operation} contained an incomplete frame.`);
    switch (payload.case) {
      case "Report":
        if (!payload.value.field0) throw new Error("Transfer frame payload is missing.");
        if (report || !sameSubject(payload.value.field0, subject)) {
          throw new Error(`${operation} metadata identity is invalid.`);
        }
        report = payload.value.field0;
        break;
      case "Chunk": {
        if (!payload.value.field0) throw new Error("Transfer frame payload is missing.");
        if (!report) throw new Error(`${operation} sent data before metadata.`);
        const chunk = payload.value.field0;
        if (chunk.offset !== expectedOffset) {
          throw new Error(`${operation} chunks are not contiguous.`);
        }
        total += BigInt(chunk.data.byteLength);
        if (total > BigInt(NATIVE_REPORT_HTML_BYTES)) {
          throw new NativeApiLimitError(operation, "report is too large");
        }
        chunks.push(chunk.data);
        expectedOffset = total;
        break;
      }
      case "Complete": {
        if (!payload.value.field0) throw new Error("Transfer frame payload is missing.");
        if (!report) throw new Error(`${operation} completed before metadata.`);
        completionSize = payload.value.field0.sizeBytes;
        completionSha = normalizedSha(payload.value.field0.sha256);
        complete = true;
        break;
      }
      default:
        throw new Error(`${operation} contained an unknown frame.`);
    }
  }
  if (!report || !complete || completionSize === undefined || !completionSha) {
    throw new Error(`${operation} ended before metadata and completion.`);
  }
  if (completionSize !== total) throw new Error(`${operation} size is invalid.`);
  const bytes = new Uint8Array(Number(total));
  let offset = 0;
  for (const chunk of chunks) {
    bytes.set(chunk, offset);
    offset += chunk.byteLength;
  }
  const digest = await crypto.subtle.digest("SHA-256", bytes);
  if (!active()) throw new Error(`${operation} was cancelled.`);
  if (hex(new Uint8Array(digest)) !== completionSha) {
    throw new Error(`${operation} digest verification failed.`);
  }
  return { report, bytes, html: new TextDecoder().decode(bytes) };
}

function sameSubject(report: ReportMetadata, subject: ReportSubject): boolean {
  if (report.identifier !== subject.identifier) return false;
  if ("cycleId" in subject) return report.testCycleId === subject.cycleId && !report.testExecutionId;
  return report.testExecutionId === subject.executionId;
}

function normalizedSha(value: string): string {
  const normalized = value.toLowerCase();
  if (!/^[0-9a-f]{64}$/.test(normalized)) throw new Error("Invalid report SHA-256.");
  return normalized;
}

function hex(bytes: Uint8Array): string {
  return Array.from(bytes, (value) => value.toString(16).padStart(2, "0")).join("");
}

/* === Report delivery entrypoints === */

/** Open a verified report through the native cache and OS opener. */
export async function openNativeReport(resource: NativeReportHtml, session?: SessionIdentity, filename?: string): Promise<boolean> {
  if (!session) throw new Error("Reports.OpenReport requires a connected native session.");
  return openNativeDocument(session, {
    kind: "html_report",
    filename: filename?.trim() || `${resource.report.identifier}.html`,
    bytes: resource.bytes,
  });
}

/** A saved report: where it landed, and why the default browser could not show it, if it could not. */
export interface SavedReport {
  path: string;
  /** Opening is a courtesy after the save, so its failure never undoes the save. */
  openFailure: string | null;
}

/** Save a verified report through the native picker, then open the saved file in the default browser. */
export async function downloadNativeReport(
  resource: NativeReportHtml,
  filename = `${resource.report.identifier}.html`,
  session?: SessionIdentity,
): Promise<SavedReport | null> {
  if (!isTauriDesktop()) {
    throw new NativeApiUnsupportedError("Save report", "desktop_save_document (native file picker)");
  }
  if (!session) throw new Error("Reports.SaveReport requires a connected native session.");
  const path = await saveNativeDocument(session, {
    kind: "html_report",
    filename: filename.trim() || `${resource.report.identifier}.html`,
    bytes: resource.bytes,
  });
  if (path === null) return null;
  const { commands } = await import("@/lib/desktop/commands");
  try {
    await commands.desktopOpenPath(path);
    return { path, openFailure: null };
  } catch (error) {
    return { path, openFailure: error instanceof Error ? error.message : String(error) };
  }
}
