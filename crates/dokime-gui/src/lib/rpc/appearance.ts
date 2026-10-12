// Dokime — SPDX-License-Identifier: Apache-2.0
// Native appearance resources. The session owner disposes resources when replaced.
import * as rpc from "./generated/client.js";
import type {
  AppearanceAssetMetadata,
  AppearanceSnapshot,
} from "./generated/dokime/v1/appearance_pb.js";
import type { SessionIdentity } from "./transport.js";

const maxAssetBytes = 8n * 1024n * 1024n;

export interface AppearanceResource {
  appearance: AppearanceSnapshot;
  logoUrl: string | null;
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
  let logoUrl: string | null = null;
  if (appearance.logo) {
    const expected = appearance.logo.asset;
    if (!expected?.reference || expected.sizeBytes > maxAssetBytes || expected.sizeBytes < 0n)
      invalid();
    if (
      !expected.contentType.startsWith("image/") ||
      !/^[a-f0-9]{64}$/.test(expected.reference.sha256)
    )
      invalid();
    const bytes = new Uint8Array(new ArrayBuffer(Number(expected.sizeBytes)));
    const download = await rpc.contentDownloadAppearanceAsset(session, {
      reference: expected.reference,
    });
    let offset = 0;
    let header = false;
    let complete = false;
    try {
      for await (const frame of download) {
        if (complete) invalid();
        switch (frame.payload.case) {
          case "metadata":
            if (header || !sameMetadata(frame.payload.value, expected)) invalid();
            header = true;
            break;
          case "chunk": {
            const chunk = frame.payload.value;
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
          case "complete":
            if (
              !header ||
              offset !== bytes.length ||
              frame.payload.value.sizeBytes !== expected.sizeBytes ||
              frame.payload.value.sha256 !== expected.reference.sha256
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
    logoUrl = URL.createObjectURL(new Blob([bytes], { type: expected.contentType }));
  }
  current(session);
  let disposed = false;
  return {
    appearance,
    logoUrl,
    dispose() {
      if (!disposed && logoUrl) URL.revokeObjectURL(logoUrl);
      disposed = true;
    },
  };
}
