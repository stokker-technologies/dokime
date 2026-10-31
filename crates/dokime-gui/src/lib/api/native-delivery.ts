/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Native File-Backed Delivery For Artifacts, Execution Files And Cycle Bundles. */

/* === Local Imports === */

import { commands } from "@/lib/desktop/commands";
import type { ArtifactSource, Delivered, DeliveryAction, DesktopSaveKind } from "@/lib/desktop/delivery";
import { isTauriDesktop } from "@/lib/desktop-open";
import { normalizeDeliveryError } from "@/lib/rpc/errors";
import type { SessionIdentity } from "@/lib/rpc/transport";
import { NativeApiUnsupportedError } from "./native-values";

/* === Types === */

export type NativeArtifactDeliveryRequest = {
  source: ArtifactSource;
  kind: DesktopSaveKind;
  filename: string;
  action: DeliveryAction;
};

/**
 * Stream one verified artifact, execution output, or sealed evidence bundle to a
 * native destination (the save dialog or the OS opener) without routing its
 * bytes through the webview. The native client verifies the server pull;
 * evidence follows the backend's declared archive limit. Inline document
 * byte limits do not apply to this file-backed transfer.
 */
export async function deliverNativeArtifact(session: SessionIdentity, request: NativeArtifactDeliveryRequest): Promise<boolean> {
  return (await deliverNativeContent(session, request)) !== null;
}

export async function deliverNativeContent(session: SessionIdentity, request: NativeArtifactDeliveryRequest): Promise<Delivered | null> {
  if (!isTauriDesktop()) {
    throw new NativeApiUnsupportedError("Deliver native artifact", "desktop_artifact_deliver (native verified content delivery)");
  }
  const filename = request.filename.trim();
  if (!filename) throw new Error("Native artifact delivery requires a filename.");
  assertCurrent(session);
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
export async function prepareNativeMedia(session: SessionIdentity, source: ArtifactSource): Promise<string> {
  if (!isTauriDesktop()) {
    throw new NativeApiUnsupportedError("Prepare native media preview", "desktop_media_prepare (native media preview cache)");
  }
  assertCurrent(session);
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
  await commands.desktopMediaRelease(generation).catch(() => undefined);
}

/* === Private Functions === */

function assertCurrent(session: SessionIdentity): void {
  if (!session.isCurrent()) throw new Error("Native document delivery lost its owning connection.");
}
