// Dokime — SPDX-License-Identifier: Apache-2.0
// Native appearance resources. The session owner disposes resources when replaced.

import type { RuntimeAppLogo } from "@/lib/types/view-contracts";
import type { Wire } from "./canonical.js";
import * as rpc from "../host/client";
import { LogoVariant } from "@host/dokime/v1/appearance_pb.js";

import type { SessionIdentity } from "./transport.js";

type AppearanceSnapshot = Wire<Awaited<ReturnType<typeof rpc.appearanceGetAppearance>>>;

export interface AppearanceResource {
  appearance: AppearanceSnapshot;
  logos: RuntimeAppLogo[];
  unavailableReason?: string;
  dispose(): void;
}

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
    const variants = new Set<string>();
    for (const logo of appearance.logos) {
      const variant = logoVariant(logo);
      if (variants.has(variant)) invalid();
      variants.add(variant);
    }
    current(session);
    return {
      appearance,
      logos,
      dispose,
      unavailableReason: appearance.logos.length
        ? "Host logo assets are unavailable until the host supplies appearance content delivery."
        : undefined,
    };
  } catch (error) {
    dispose();
    throw error;
  }
}

function logoVariant(logo: AppearanceSnapshot["logos"][number]): RuntimeAppLogo["variant"] {
  switch (logo.variant) {
    case LogoVariant.TRANSPARENT_ON_LIGHT:
      return "transparent_on_light";
    case LogoVariant.TRANSPARENT_ON_DARK:
      return "transparent_on_dark";
    case LogoVariant.CONTAINED_ON_LIGHT:
      return "contained_on_light";
    case LogoVariant.CONTAINED_ON_DARK:
      return "contained_on_dark";
    default:
      return invalid();
  }
}
