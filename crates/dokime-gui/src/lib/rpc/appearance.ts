// Dokime — SPDX-License-Identifier: Apache-2.0
import { type AppearanceResource, logoVariant } from "@dokime/document/lib/native-appearance";
// Native appearance resources. The session owner disposes resources when replaced.

import type { RuntimeAppLogo } from "@dokime/plots/lib/types/appearance";
import * as rpc from "../host/client";

import type { SessionIdentity } from "./transport.js";

import { readFileReference } from "@/lib/api/native-content";
import { required } from "@dokime/plots/lib/rpc/canonical";
function current(session: SessionIdentity) {
  if (!session.isCurrent()) throw { kind: "session", detail: { kind: "superseded" } };
}
function invalid(): never {
  throw new Error("Read Host Appearance: Invalid Logo Declaration; Refresh Appearance or Update Its Provider");
}
/** Read the canonical host appearance. Provider logos remain unavailable until
 * the host serves their content; the shell displays that reason independently
 * of its installed appearance assets.
 */
export async function loadAppearance(session: SessionIdentity): Promise<AppearanceResource> {
  const appearance = required((await rpc.backendAppearance(session, {})).appearance, "Read Appearance");
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
    const variants = new Set<string>();
    for (const logo of appearance.logos) {
      const variant = logoVariant(logo);
      if (variants.has(variant)) invalid();
      variants.add(variant);
      const asset = logo.asset;
      if (asset?.file) {
        const bytes = await readFileReference(session, asset.file);
        current(session);
        const url = URL.createObjectURL(new Blob([new Uint8Array(bytes)], { type: asset.contentType }));
        const reference = required(asset.reference, "Read Appearance Asset Identity");
        logos.push({ variant, url, plugin_id: reference.pluginId, asset_id: reference.assetId });
      }
    }
    current(session);
    return {
      appearance,
      logos,
      dispose,
      unavailableReason:
        appearance.logos.length !== logos.length
          ? "Host logo assets are unavailable until the host supplies appearance content delivery."
          : undefined,
    };
  } catch (error) {
    dispose();
    throw error;
  }
}
