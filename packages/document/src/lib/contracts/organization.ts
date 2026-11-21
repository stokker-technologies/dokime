/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Installed Appearance Supplied By The Desktop Shell
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/** Mirrors `LoadedPack` in `src-tauri/src/app/organization.rs`. */
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

/* === Private Functions === */

/* === Entry Point === */
