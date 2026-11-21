/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Build Version Labels for Settings › About
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
import type { BuildIdentity } from "@dokime/document/lib/contracts/connection";

/* === Re-Exports === */

/* === Constants === */

const COMMIT = /^[0-9a-f]{7,64}$/;
const DIRTY = "-dirty";
const SHORT_DIGITS = 8;

/* === Types === */

export type BuildLabel = {
  /** The package version. */
  version: string;
  /** The full commit to copy, without the dirty mark; `null` when the build names none. */
  commit: string | null;
  /** The abbreviated commit with its dirty mark, or `unknown`. */
  revision: string;
  channel: string;
  text: string;
};

/* === Classes === */

/* === Public Functions === */

export function buildLabel(build: BuildIdentity): BuildLabel {
  const version = build.productVersion;
  const source = build.sourceRevision ?? "";
  const dirty = source.endsWith(DIRTY);
  const candidate = dirty ? source.slice(0, -DIRTY.length) : source;
  const commit = COMMIT.test(candidate) ? candidate : null;
  const revision = commit ? commit.slice(0, SHORT_DIGITS) + (dirty ? DIRTY : "") : "unknown";
  const channel = build.channel || "local";
  return { version, commit, revision, channel, text: `${version} (${revision}, ${channel})` };
}

/* === Private Functions === */

/* === Entry Point === */
