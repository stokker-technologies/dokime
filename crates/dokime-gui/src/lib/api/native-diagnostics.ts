/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Bounded Diagnostics Ranges From the Library Status Owner. */

import * as rpc from "@/lib/host/client";
import type { SessionIdentity } from "@/lib/rpc/transport";
import { NATIVE_DOCUMENT_BYTES } from "./native-document-delivery";

const RANGE_BYTES = 262_144n;
export type DiagnosticsBundle = { filename: string; bytes: Uint8Array };
export function diagnosticsBundleSupported(): boolean {
  return true;
}

export async function collectDiagnosticsBundle(session: SessionIdentity): Promise<DiagnosticsBundle> {
  let name = "";
  let offset = 0n;
  let length: bigint | undefined;
  let digest: Uint8Array | undefined;
  const chunks: Uint8Array[] = [];
  do {
    if (!session.isCurrent()) throw new Error("Collect Diagnostics: Owning Connection Changed");
    const range = await rpc.statusDiagnosticsGet(session, { name, offset, maxBytes: RANGE_BYTES });
    if (!session.isCurrent()) throw new Error("Collect Diagnostics: Owning Connection Changed");
    if (!/^[0-9a-f]{64}$/.test(range.name) || range.sha256.byteLength !== 32)
      throw new Error("Collect Diagnostics: Invalid Archive Identity");
    if (length === undefined) {
      name = range.name;
      length = range.totalLength;
      digest = range.sha256;
      if (length > BigInt(NATIVE_DOCUMENT_BYTES)) throw new Error("Collect Diagnostics: Archive Exceeds Native Save Budget");
    } else if (name !== range.name || length !== range.totalLength || digest?.some((byte, index) => byte !== range.sha256[index])) {
      throw new Error("Collect Diagnostics: Archive Changed During Transfer; Collect It Again");
    }
    if (
      BigInt(range.data.byteLength) > RANGE_BYTES ||
      offset + BigInt(range.data.byteLength) > length ||
      (!range.data.byteLength && offset < length)
    ) {
      throw new Error("Collect Diagnostics: Incomplete or Invalid Archive Range");
    }
    chunks.push(range.data);
    offset += BigInt(range.data.byteLength);
  } while (offset < length);
  const bytes = new Uint8Array(Number(length));
  let written = 0;
  for (const chunk of chunks) {
    bytes.set(chunk, written);
    written += chunk.byteLength;
  }
  const actual = new Uint8Array(await crypto.subtle.digest("SHA-256", bytes));
  if (!digest || actual.some((byte, index) => byte !== digest[index]))
    throw new Error("Collect Diagnostics: Archive Digest Verification Failed");
  if (!session.isCurrent()) throw new Error("Collect Diagnostics: Owning Connection Changed");
  return { filename: `diagnostics-${name}.tar`, bytes };
}
