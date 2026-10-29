/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Organization Pack Status The Desktop Shell Reports
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/** Mirrors `PackSummary` in `src-tauri/src/app/organization.rs`. */
export type OrganizationPackSummary = {
  label: string;
  organization: string | null;
};

/** Mirrors `OrganizationStatus` in `src-tauri/src/app/organization.rs`. */
export type OrganizationStatus = {
  active: OrganizationPackSummary | null;
  pending: OrganizationPackSummary | null;
  removed: boolean;
  error: string | null;
};

/** Mirrors `LoadedPack`: what the shell hands the frontend before it renders. */
export type BootOrganizationPack = {
  label: string;
  titleTemplate: string;
  organization: { name: string; unit?: string | null; website?: string | null } | null;
  logos: Record<string, string>;
  css: string;
  updates: OrganizationUpdatePolicy | null;
};

export type OrganizationUpdatePolicy = {
  mode: "disabled" | "manual" | "automatic_check";
  source: string | null;
  sources: Record<
    string,
    | { kind: "direct"; manifest: string; plugins: string | null; connection: string | null }
    | { kind: "github"; repository: string }
    | { kind: "gitlab"; connection: string; project: string }
  >;
  connections: Record<string, { host: string }>;
};

export type BootOrganization = { pack: BootOrganizationPack | null; error: string | null };

/* === Classes === */

/* === Public Functions === */

export function organizationPackError(error: string | null, retained: boolean): string | null {
  if (!error) return null;
  const appearance = retained ? "The last valid pack remains active." : "The bundled organization applies.";
  return `Not applied: ${error}. ${appearance}`;
}

/* === Private Functions === */

/* === Entry Point === */
