// Dokime — SPDX-License-Identifier: Apache-2.0
// Native appearance resources. The session owner disposes resources when replaced.

import type { Wire } from "./canonical.js";
import * as rpc from "./generated/client.js";
import type { RuntimeAppLogo } from "@/lib/types/view-contracts";

type AppearanceAssetMetadata = NonNullable<AppearanceSnapshot["logos"][number]["asset"]>;

import type { SessionIdentity } from "./transport.js";

/* === Canonical Types === */

type AppearanceSnapshot = Wire<Awaited<ReturnType<typeof rpc.appearanceGetAppearance>>>;

const maxAssetBytes = 8n * 1024n * 1024n;

export interface AppearanceResource {
  appearance: AppearanceSnapshot;
  logos: RuntimeAppLogo[];
  dispose(): void;
}

function current(session: SessionIdentity) {
  if (!session.isCurrent()) throw { kind: "session", detail: { kind: "superseded" } };
}
function invalid(): never {
  throw new Error("Invalid appearance asset transfer");
}
function sameMetadata(left: AppearanceAssetMetadata, right: AppearanceAssetMetadata) {
  return (
    left.sizeBytes === right.sizeBytes &&
    left.contentType === right.contentType &&
    left.reference?.pluginId === right.reference?.pluginId &&
    left.reference?.assetId === right.reference?.assetId &&
    left.reference?.sha256 === right.reference?.sha256
  );
}

/** Uses generated protobuf calls; never resolves backend paths or HTTP URLs.
 * The caller must refresh appearance after a missing/changed asset response.
 * Returned blob URLs belong to this resource and must be disposed on unmount or
 * session replacement. Production wiring waits for the desktop session owner.
 */
export async function loadAppearance(session: SessionIdentity): Promise<AppearanceResource> {
  const appearance = await rpc.appearanceGetAppearance(session, {});
  const logos: RuntimeAppLogo[] = [];
  let disposed = false;
  const dispose = () => {
    if (disposed) return;
    for (const logo of logos) URL.revokeObjectURL(logo.url);
    disposed = true;
  };
  try {
    current(session);
    if (appearance.logos.length > 4) invalid();
    for (const logo of appearance.logos) {
      const variant = logoVariant(logo);
      if (logos.some((existing) => existing.variant === variant)) invalid();
      const url = await downloadLogo(session, logo.asset);
      logos.push({
        variant,
        plugin_id: logo.asset?.reference?.pluginId ?? "",
        asset_id: logo.asset?.reference?.assetId ?? "",
        url,
        alt: logo.alt ?? null,
      });
    }
    current(session);
    return { appearance, logos, dispose };
  } catch (error) {
    dispose();
    throw error;
  }
}

function logoVariant(logo: AppearanceSnapshot["logos"][number]): RuntimeAppLogo["variant"] {
  switch (logo.variant?.HarmosVariant.case) {
    case "TransparentOnLight":
      return "transparent_on_light";
    case "TransparentOnDark":
      return "transparent_on_dark";
    case "ContainedOnLight":
      return "contained_on_light";
    case "ContainedOnDark":
      return "contained_on_dark";
    default:
      return invalid();
  }
}

async function downloadLogo(session: SessionIdentity, expected: AppearanceAssetMetadata | undefined): Promise<string> {
  if (!expected?.reference || expected.sizeBytes > maxAssetBytes || expected.sizeBytes < 0n) invalid();
  if (!expected.contentType.startsWith("image/") || !/^[a-f0-9]{64}$/.test(expected.reference.sha256)) invalid();
  const bytes = new Uint8Array(new ArrayBuffer(Number(expected.sizeBytes)));
  const download = await rpc.contentDownloadAppearanceAsset(session, {
    reference: {
      pluginId: expected.reference.pluginId,
      assetId: expected.reference.assetId,
      sha256: expected.reference.sha256,
    },
  });
  let offset = 0;
  let header = false;
  let complete = false;
  try {
    for await (const frame of download) {
      if (complete) invalid();
      const payload = frame.payload?.HarmosVariant;
      if (!payload?.case || !payload.value.field0) invalid();
      switch (payload.case) {
        case "Metadata":
          if (!payload.value.field0) throw new Error("Transfer frame payload is missing.");
          if (header || !sameMetadata(payload.value.field0, expected)) invalid();
          header = true;
          break;
        case "Chunk": {
          if (!payload.value.field0) throw new Error("Transfer frame payload is missing.");
          const chunk = payload.value.field0;
          if (
            !header ||
            chunk.offset !== BigInt(offset) ||
            chunk.data.length === 0 ||
            chunk.data.length > 65536 ||
            offset + chunk.data.length > bytes.length
          )
            invalid();
          bytes.set(chunk.data, offset);
          offset += chunk.data.length;
          break;
        }
        case "Complete":
          if (!payload.value.field0) throw new Error("Transfer frame payload is missing.");
          if (
            !header ||
            offset !== bytes.length ||
            payload.value.field0.sizeBytes !== expected.sizeBytes ||
            payload.value.field0.sha256 !== expected.reference.sha256
          )
            invalid();
          complete = true;
          break;
        default:
          invalid();
      }
    }
    // EOF here also requires an OK final gRPC status, not just completion data.
    if (!complete) invalid();
    const digest = new Uint8Array(await crypto.subtle.digest("SHA-256", bytes.buffer));
    const sha256 = Array.from(digest, (value) => value.toString(16).padStart(2, "0")).join("");
    if (sha256 !== expected.reference.sha256) invalid();
  } finally {
    await download.cancel();
  }
  current(session);
  return URL.createObjectURL(new Blob([bytes], { type: expected.contentType }));
}
