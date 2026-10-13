/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The support archive, as bytes and a name.
 *
 * `Backend.CollectDiagnostics` streams one zip the operator sends to the
 * developers: a header frame naming the file, chunk frames carrying it at
 * declared offsets, and a completion frame stating the size, exactly as
 * `Reports.DownloadCycleReport` does. This adapter returns the assembled bytes
 * and the suggested filename and nothing else — no view model, so the
 * Diagnostics section decides what to do with them.
 */

/* === Local Imports === */

import { NativeApiUnsupportedError } from "@/lib/api/native-values";
import * as rpc from "@/lib/rpc/generated/client";
import type { SessionIdentity } from "@/lib/rpc/transport";

/* === Types === */

export type DiagnosticsBundle = {
  /** The name the backend suggests for the archive; never a backend path. */
  filename: string;
  bytes: Uint8Array;
};

/* === Public Functions === */

/** Whether this client can ask the backend for a support archive at all. */
export function diagnosticsBundleSupported(): boolean {
  return typeof rpc.backendCollectDiagnostics === "function";
}

export async function collectDiagnosticsBundle(
  session: SessionIdentity,
): Promise<DiagnosticsBundle> {
  if (!diagnosticsBundleSupported()) {
    throw new NativeApiUnsupportedError(
      "Collect diagnostics",
      "Backend.CollectDiagnostics (not available on this backend)",
    );
  }
  const reader = await rpc.backendCollectDiagnostics(session, {});
  let filename = "";
  let expectedOffset = 0n;
  let complete = false;
  const chunks: Uint8Array[] = [];
  let total = 0;

  try {
    for await (const frame of reader) {
      if (complete) throw new Error("Backend.CollectDiagnostics continued after completion.");
      const payload = frame.payload?.HarmosVariant;
      if (!payload) throw new Error("Backend.CollectDiagnostics delivered no payload.");
      switch (payload.case) {
        case "Header": {
          const header = payload.value.field0;
          if (!header) throw new Error("Backend.CollectDiagnostics delivered no header.");
          filename = header.filename;
          break;
        }
        case "Chunk": {
          const chunk = payload.value.field0;
          if (!chunk) throw new Error("Backend.CollectDiagnostics delivered no chunk.");
          // The offsets are declared, so a gap or a replay is a corrupt
          // archive rather than something to stitch over.
          if (chunk.offset !== expectedOffset) {
            throw new Error(
              `Backend.CollectDiagnostics delivered chunk ${chunk.offset} where ${expectedOffset} was due.`,
            );
          }
          chunks.push(chunk.data);
          total += chunk.data.byteLength;
          expectedOffset += BigInt(chunk.data.byteLength);
          break;
        }
        case "Complete":
          if (payload.value.field0?.sizeBytes !== BigInt(total)) {
            throw new Error("Backend.CollectDiagnostics delivered an incomplete archive.");
          }
          complete = true;
          break;
        default:
          break;
      }
    }
  } finally {
    await reader.cancel().catch(() => undefined);
  }

  if (!complete) throw new Error("Backend.CollectDiagnostics ended before the archive was whole.");
  if (!filename) throw new Error("Backend.CollectDiagnostics named no file.");

  const bytes = new Uint8Array(total);
  let written = 0;
  for (const chunk of chunks) {
    bytes.set(chunk, written);
    written += chunk.byteLength;
  }
  return { bytes, filename };
}
