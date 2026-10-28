/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Organization Pack Row States
 */

import { type OrganizationPackSummary, type OrganizationStatus, organizationPackError } from "@/lib/desktop/organization";

export type OrganizationPackState = "none" | "active" | "pending" | "removed" | "invalid";

export type OrganizationPackPresentation = {
  state: OrganizationPackState;
  /** The pack named in the row, or the bundled organization. */
  value: string;
  /** What happens next, in one sentence. */
  detail: string;
  /** The host's reason a pack did not apply; shown as the field error. */
  error: string | null;
  restart: boolean;
  remove: boolean;
};

export function organizationPackPresentation(status: OrganizationStatus): OrganizationPackPresentation {
  const base = { error: organizationPackError(status.error, status.active !== null) };
  if (status.pending) {
    return {
      ...base,
      state: "pending",
      value: packName(status.pending),
      detail: "Applies on next start.",
      restart: true,
      remove: true,
    };
  }
  if (status.removed) {
    return {
      ...base,
      state: "removed",
      value: "Bundled Organization",
      detail: "Bundled organization applies on next start.",
      restart: true,
      remove: false,
    };
  }
  if (status.active) {
    return {
      ...base,
      state: "active",
      value: packName(status.active),
      detail: "Active since this start.",
      restart: false,
      remove: true,
    };
  }
  if (status.error) {
    return {
      ...base,
      state: "invalid",
      value: "Bundled Organization",
      detail: "Remove the pack or choose another.",
      restart: false,
      remove: true,
    };
  }
  return {
    ...base,
    state: "none",
    value: "Bundled Organization",
    detail: "Choose a zipped organization folder to rebrand this desktop.",
    restart: false,
    remove: false,
  };
}

function packName(pack: OrganizationPackSummary): string {
  return pack.organization ? `${pack.label} · ${pack.organization}` : pack.label;
}
