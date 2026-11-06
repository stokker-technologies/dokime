/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Build Version Labels For Settings › About
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
import type { BuildIdentity } from "@/lib/desktop/connection";

/* === Re-Exports === */

/* === Constants === */

const COMMIT = /^[0-9a-f]{7,64}$/;
const DIRTY = "-dirty";
const SHORT_DIGITS = 8;

/* === Types === */

export type BuildLabel = {
  /** `v0.1.2`. */
  version: string;
  /** The full commit to copy, without the dirty mark; `null` when the build names none. */
  commit: string | null;
  /** The abbreviated commit with its dirty mark, or `unknown`. */
  revision: string;
  channel: string | null;
};

/* === Classes === */

/* === Public Functions === */

export function buildLabel(build: BuildIdentity): BuildLabel {
  const version = `v${build.productVersion}`;
  const source = build.sourceRevision ?? "";
  const dirty = source.endsWith(DIRTY);
  const commit = dirty ? source.slice(0, -DIRTY.length) : source;
  if (!COMMIT.test(commit)) return { version, commit: null, revision: "unknown", channel: build.channel };
  return { version, commit, revision: commit.slice(0, SHORT_DIGITS) + (dirty ? DIRTY : ""), channel: build.channel };
}

/** `v0.1.2 · 766330b7 · nightly`, the text the About row shows. */
export function buildText(label: BuildLabel): string {
  return [label.version, label.revision, label.channel].filter(Boolean).join(" · ");
}

/* === Private Functions === */

/* === Entry Point === */
