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

import { isTauriDesktop } from "@/lib/desktop-open";
import * as rpc from "@/lib/rpc/generated/client";
import type { ExecutionCheckpoint } from "@/lib/rpc/generated/dokime/v1/checkpoint_pb";
import type {
  RenderedReport,
  ReportHtmlFrame,
  ReportMetadata,
} from "@/lib/rpc/generated/dokime/v1/reports_pb";
import type { NativeStream, SessionIdentity } from "@/lib/rpc/transport";
import {
  NATIVE_HTML_DOCUMENT_BYTES,
  openNativeDocument,
  saveNativeDocument,
} from "./native-delivery";
import { NativeApiLimitError, NativeApiUnsupportedError } from "./native-values";

/* === Limits and public types === */

/** Downloaded report bodies remain bounded even when the browser can render larger documents. */
export const NATIVE_REPORT_HTML_BYTES = NATIVE_HTML_DOCUMENT_BYTES;
export const NATIVE_RENDERED_REPORT_BYTES = 4 * 1024 * 1024;

export interface NativeReportSession extends SessionIdentity {
  /** Keep report streams attached to the selected connection generation. */
  own(dispose: () => void): () => void;
}

export interface NativeReportHtml {
  report: ReportMetadata;
  bytes: Uint8Array;
  html: string;
}

function assertActive(
  session: SessionIdentity,
  signal: AbortSignal | undefined,
  operation: string,
): void {
  if (signal?.aborted) throw new Error(`${operation} was cancelled.`);
  if (!session.isCurrent()) throw new Error(`${operation} lost its owning connection.`);
}

export async function renderNativeCheckpointReport(
  session: SessionIdentity,
  options: {
    context?: ExecutionCheckpoint;
    checkpoints: ExecutionCheckpoint[];
    identifier: string;
  },
): Promise<RenderedReport> {
  if (!options.identifier.trim() || !options.checkpoints.length) {
    throw new Error("Reports.RenderCheckpointReport requires an identifier and checkpoint.");
  }
  const reply = await rpc.reportsRenderCheckpointReport(session, options);
  const bytes = new TextEncoder().encode(reply.html);
  if (bytes.byteLength > NATIVE_RENDERED_REPORT_BYTES) {
    throw new NativeApiLimitError(
      "Reports.RenderCheckpointReport",
      `HTML exceeds ${NATIVE_RENDERED_REPORT_BYTES} bytes`,
    );
  }
  return reply;
}

/* === Verified On-Demand Report Stream === */

export async function downloadNativeCycleReport(
  session: NativeReportSession,
  cycleId: string,
  identifier: string,
  signal?: AbortSignal,
): Promise<NativeReportHtml> {
  if (!cycleId.trim() || !identifier.trim()) {
    throw new Error("Reports.DownloadCycleReport requires cycle and report identifiers.");
  }
  assertActive(session, signal, "Reports.DownloadCycleReport");
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
    reader = await rpc.reportsDownloadCycleReport(session, { cycleId, identifier });
    assertActive(session, signal, "Reports.DownloadCycleReport");
    release = session.own(() => void cancelReader().catch(() => undefined));
    return await assembleReportHtml(
      reader,
      { cycleId, identifier },
      () => !signal?.aborted && session.isCurrent(),
    );
  } finally {
    signal?.removeEventListener("abort", abort);
    await cancelReader().catch(() => undefined);
    release?.();
  }
}

async function assembleReportHtml(
  reader: NativeStream<ReportHtmlFrame>,
  request: { cycleId: string; identifier: string },
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
    if (!active()) throw new Error("Reports.DownloadCycleReport was cancelled.");
    if (complete) throw new Error("Reports.DownloadCycleReport continued after completion.");
    switch (frame.payload.case) {
      case "report":
        if (
          report ||
          frame.payload.value.testCycleId !== request.cycleId ||
          frame.payload.value.identifier !== request.identifier
        ) {
          throw new Error("Reports.DownloadCycleReport metadata identity is invalid.");
        }
        report = frame.payload.value;
        break;
      case "chunk": {
        if (!report) throw new Error("Reports.DownloadCycleReport sent data before metadata.");
        const chunk = frame.payload.value;
        if (chunk.offset !== expectedOffset) {
          throw new Error("Reports.DownloadCycleReport chunks are not contiguous.");
        }
        total += BigInt(chunk.data.byteLength);
        if (total > BigInt(NATIVE_REPORT_HTML_BYTES)) {
          throw new NativeApiLimitError("Reports.DownloadCycleReport", "report is too large");
        }
        chunks.push(chunk.data);
        expectedOffset = total;
        break;
      }
      case "complete": {
        if (!report) throw new Error("Reports.DownloadCycleReport completed before metadata.");
        completionSize = frame.payload.value.sizeBytes;
        completionSha = normalizedSha(frame.payload.value.sha256);
        complete = true;
        break;
      }
      default:
        throw new Error("Reports.DownloadCycleReport contained an unknown frame.");
    }
  }
  if (!report || !complete || completionSize === undefined || !completionSha) {
    throw new Error("Reports.DownloadCycleReport ended before metadata and completion.");
  }
  if (completionSize !== total) throw new Error("Reports.DownloadCycleReport size is invalid.");
  const bytes = new Uint8Array(Number(total));
  let offset = 0;
  for (const chunk of chunks) {
    bytes.set(chunk, offset);
    offset += chunk.byteLength;
  }
  const digest = await crypto.subtle.digest("SHA-256", bytes);
  if (!active()) throw new Error("Reports.DownloadCycleReport was cancelled.");
  if (hex(new Uint8Array(digest)) !== completionSha) {
    throw new Error("Reports.DownloadCycleReport digest verification failed.");
  }
  return { report, bytes, html: new TextDecoder().decode(bytes) };
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
export async function openNativeReport(
  resource: NativeReportHtml,
  session?: SessionIdentity,
  filename?: string,
): Promise<boolean> {
  if (!session) throw new Error("Reports.OpenReport requires a connected native session.");
  return openNativeDocument(session, {
    kind: "html_report",
    filename: filename?.trim() || `${resource.report.identifier}.html`,
    bytes: resource.bytes,
  });
}

/** Save a verified report through the native picker. */
export async function downloadNativeReport(
  resource: NativeReportHtml,
  filename = `${resource.report.identifier}.html`,
  session?: SessionIdentity,
): Promise<boolean> {
  if (!isTauriDesktop()) {
    throw new NativeApiUnsupportedError(
      "Save report",
      "desktop_save_document (native file picker)",
    );
  }
  if (!session) throw new Error("Reports.SaveReport requires a connected native session.");
  return saveNativeDocument(session, {
    kind: "html_report",
    filename: filename.trim() || `${resource.report.identifier}.html`,
    bytes: resource.bytes,
  });
}
