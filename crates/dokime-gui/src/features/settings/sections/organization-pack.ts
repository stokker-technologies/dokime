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

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
import { organizationPackError, type OrganizationPackSummary, type OrganizationStatus } from "@/lib/desktop/organization";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

export type OrganizationPackState = "none" | "active" | "pending" | "removed" | "invalid";

export type OrganizationPackPresentation = {
  state: OrganizationPackState;
  /** The pack named in the row, or the compiled brand. */
  value: string;
  /** What happens next, in one sentence. */
  detail: string;
  /** The host's reason a pack did not apply; shown as the field error. */
  error: string | null;
  /** Build-time parts the pack carries that this build keeps compiled. */
  ignored: string | null;
  restart: boolean;
  remove: boolean;
};

/* === Classes === */

/* === Public Functions === */

export function organizationPackPresentation(status: OrganizationStatus): OrganizationPackPresentation {
  const base = { error: organizationPackError(status.error, status.active !== null), ignored: null };
  if (status.pending) {
    return {
      ...base,
      state: "pending",
      value: packName(status.pending),
      detail: "Applies on next start.",
      ignored: ignoredParts(status.pending),
      restart: true,
      remove: true,
    };
  }
  if (status.removed) {
    return {
      ...base,
      state: "removed",
      value: "Compiled Brand",
      detail: "Compiled brand applies on next start.",
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
      ignored: ignoredParts(status.active),
      restart: false,
      remove: true,
    };
  }
  if (status.error) {
    return {
      ...base,
      state: "invalid",
      value: "Compiled Brand",
      detail: "Remove the pack or choose another.",
      restart: false,
      remove: true,
    };
  }
  return {
    ...base,
    state: "none",
    value: "Compiled Brand",
    detail: "Choose a zipped organization folder to rebrand this desktop.",
    restart: false,
    remove: false,
  };
}

/* === Private Functions === */

function packName(pack: OrganizationPackSummary): string {
  return pack.organization ? `${pack.label} · ${pack.organization}` : pack.label;
}

function ignoredParts(pack: OrganizationPackSummary): string | null {
  return pack.ignored.length ? `Kept from this build: ${pack.ignored.join(", ")}.` : null;
}

/* === Entry Point === */
