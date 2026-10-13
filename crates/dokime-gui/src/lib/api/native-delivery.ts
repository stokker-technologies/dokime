/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Shared native delivery for verified bounded documents, and for the large
   (up to 2 GiB) artifact/execution-file/cycle-bundle content that no longer
   fits the base64 JSON IPC hop those documents use. */

/* === Local Imports === */

import type {
  ArtifactSource,
  DeliveryAction,
  DesktopSaveDocument,
  DesktopSaveKind,
} from "@/lib/desktop/delivery";
import { isTauriDesktop } from "@/lib/desktop-open";
import { normalizeDeliveryError } from "@/lib/rpc/errors";
import type { SessionIdentity } from "@/lib/rpc/transport";
import { NativeApiUnsupportedError } from "./native-values";

/* === Limits and public types === */

/** Native HTML reports are bounded before crossing the Tauri IPC boundary. */
export const NATIVE_HTML_DOCUMENT_BYTES = 32 * 1024 * 1024;

export type NativeDeliveryRequest = {
  kind: DesktopSaveKind;
  filename: string;
  bytes: Uint8Array;
};

/**
 * Open verified bytes through the main-window native cache and OS opener.
 *
 * The desktop command owns filename sanitization, encoded-size preflight,
 * private filesystem publication, and generation checks. This adapter repeats
 * the byte bound to stop an oversized request before base64 encoding and IPC
 * allocation.
 */
export async function openNativeDocument(
  session: SessionIdentity,
  request: NativeDeliveryRequest,
): Promise<boolean> {
  if (!isTauriDesktop()) {
    throw new NativeApiUnsupportedError(
      "Open native document",
      "desktop_open_document (native document cache)",
    );
  }
  const document = prepareNativeDocument(session, request);
  const { commands } = await import("@/lib/desktop/commands");
  await commands.desktopOpenDocument(session.generation, document);
  assertCurrent(session);
  return true;
}

/**
 * Save verified bytes through the main-window native picker.
 *
 * The desktop command owns extension, encoded-size preflight, private filesystem
 * publication, and generation checks. This adapter repeats the byte bound to stop
 * an oversized request before base64 encoding and IPC allocation.
 */
export async function saveNativeDocument(
  session: SessionIdentity,
  request: NativeDeliveryRequest,
): Promise<boolean> {
  if (!isTauriDesktop()) {
    throw new NativeApiUnsupportedError(
      "Save native document",
      "desktop_save_document (native file picker)",
    );
  }
  const document = prepareNativeDocument(session, request);
  const { commands } = await import("@/lib/desktop/commands");
  const saved = await commands.desktopSaveDocument(session.generation, document);
  assertCurrent(session);
  return saved;
}

/**
 * Save the support archive through the main-window native picker, and say
 * where it landed.
 *
 * The archive is assembled in the webview from `Backend.CollectDiagnostics`
 * frames, so it has no artifact, execution-file or cycle id for the verified
 * server pull to address; it crosses inline under the same 32 MiB bound as an
 * HTML report. Its own command, rather than `desktop_save_document`, because
 * this is the one save whose *path* the caller needs: the row offers the
 * folder the operator chose, and only the shell knows what that was.
 */
export async function saveNativeDiagnosticsBundle(
  session: SessionIdentity,
  request: Omit<NativeDeliveryRequest, "kind">,
): Promise<string> {
  if (!isTauriDesktop()) {
    throw new NativeApiUnsupportedError(
      "Save diagnostics bundle",
      "desktop_save_diagnostics_bundle (native file picker)",
    );
  }
  /* The shell requires the extension rather than appending it, so a filename
     the backend named something else is refused here, with the reason, rather
     than by an IPC error. */
  if (!request.filename.trim().toLowerCase().endsWith(".zip")) {
    throw new Error(
      `The diagnostics bundle must be saved as a .zip archive; the backend named it "${request.filename}".`,
    );
  }
  const document = prepareNativeDocument(session, { ...request, kind: "diagnostics_bundle" });
  const { commands } = await import("@/lib/desktop/commands");
  const path = await commands.desktopSaveDiagnosticsBundle(session.generation, document);
  assertCurrent(session);
  return path;
}

export type NativeArtifactDeliveryRequest = {
  source: ArtifactSource;
  kind: DesktopSaveKind;
  filename: string;
  action: DeliveryAction;
};

/**
 * Stream one verified artifact, execution output, or cycle bundle to a
 * native destination (the save dialog or the OS opener) without routing its
 * bytes through the webview. Unlike `saveNativeDocument`/`openNativeDocument`,
 * there is no client-side byte bound to preflight: `content::fetch_verified`
 * pulls the content straight from the server, scaling to the full 2 GiB
 * artifact/cycle-bundle bound instead of the 32 MiB base64 JSON cap.
 */
export async function deliverNativeArtifact(
  session: SessionIdentity,
  request: NativeArtifactDeliveryRequest,
): Promise<boolean> {
  if (!isTauriDesktop()) {
    throw new NativeApiUnsupportedError(
      "Deliver native artifact",
      "desktop_artifact_deliver (native verified content delivery)",
    );
  }
  const filename = request.filename.trim();
  if (!filename) throw new Error("Native artifact delivery requires a filename.");
  assertCurrent(session);
  const { commands } = await import("@/lib/desktop/commands");
  const delivered = await commands
    .desktopArtifactDeliver(session.generation, {
      source: request.source,
      kind: request.kind,
      filename,
      action: request.action,
    })
    .catch((error: unknown) => {
      throw normalizeDeliveryError(error);
    });
  assertCurrent(session);
  return delivered;
}

/**
 * Materialize one verified artifact or execution file in the private media
 * preview cache and return its path (`"<generation>/<key>"`, not a URL: the
 * caller turns it into one with `convertFileSrc(path, "dokime-media")`).
 */
export async function prepareNativeMedia(
  session: SessionIdentity,
  source: ArtifactSource,
): Promise<string> {
  if (!isTauriDesktop()) {
    throw new NativeApiUnsupportedError(
      "Prepare native media preview",
      "desktop_media_prepare (native media preview cache)",
    );
  }
  assertCurrent(session);
  const { commands } = await import("@/lib/desktop/commands");
  const path = await commands.desktopMediaPrepare(session.generation, source);
  assertCurrent(session);
  return path;
}

/**
 * Release one generation's media preview cache. Best-effort: called on
 * disconnect and generation change, when the connection this cache belonged
 * to is already gone.
 */
export async function releaseNativeMedia(generation: string): Promise<void> {
  if (!isTauriDesktop()) return;
  const { commands } = await import("@/lib/desktop/commands");
  await commands.desktopMediaRelease(generation).catch(() => undefined);
}

/* === Private Functions === */

function prepareNativeDocument(
  session: SessionIdentity,
  request: NativeDeliveryRequest,
): DesktopSaveDocument {
  const filename = request.filename.trim();
  if (!filename) throw new Error("Native document delivery requires a filename.");
  if (request.bytes.byteLength > NATIVE_HTML_DOCUMENT_BYTES) {
    throw new Error(`Native document delivery exceeds ${NATIVE_HTML_DOCUMENT_BYTES} bytes.`);
  }
  assertCurrent(session);
  return {
    kind: request.kind,
    filename,
    bytesBase64: encodeNativeBytes(request.bytes),
  };
}

function encodeNativeBytes(bytes: Uint8Array): string {
  let binary = "";
  const chunkSize = 0x8000;
  for (let offset = 0; offset < bytes.byteLength; offset += chunkSize) {
    binary += String.fromCharCode(...bytes.subarray(offset, offset + chunkSize));
  }
  return btoa(binary);
}

function assertCurrent(session: SessionIdentity): void {
  if (!session.isCurrent()) throw new Error("Native document delivery lost its owning connection.");
}
