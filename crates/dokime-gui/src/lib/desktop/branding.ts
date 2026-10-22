/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Branding Pack Status The Desktop Shell Reports
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/** Mirrors `PackSummary` in `src-tauri/src/app/branding.rs`. */
export type BrandingPackSummary = {
  label: string;
  organization: string | null;
  /** Build-time parts the pack carries that only a rebuild applies. */
  ignored: string[];
};

/** Mirrors `BrandingStatus` in `src-tauri/src/app/branding.rs`. */
export type BrandingStatus = {
  active: BrandingPackSummary | null;
  pending: BrandingPackSummary | null;
  removed: boolean;
  error: string | null;
};

/** Mirrors `LoadedPack`: what the shell hands the frontend before it renders. */
export type BootBrandPack = {
  label: string;
  titleTemplate: string;
  organization: { name: string; unit?: string | null; website?: string | null } | null;
  logos: Record<string, string>;
  css: string;
};

export type BootBranding = { pack: BootBrandPack | null; error: string | null };

/* === Classes === */

/* === Public Functions === */

/* === Private Functions === */

/* === Entry Point === */
