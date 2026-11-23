/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The programs a host, an embedded application or the launch configuration
 * registered, on the same row as an installed plugin.
 *
 * A program's own settings are a page under Settings › Plugins, not an
 * expander nested inside this list: the row states what the program is and
 * where it came from, and its settings button goes to that page.
 */

/* === External Imports === */

/* === Local Imports === */

import type { RuntimeInstalledPluginRecord } from "@/features/settings/installed-plugins/installed-plugin-contracts";
import type { SettingsFrame } from "@generated/host/model";
import type { Wire } from "@/lib/rpc/canonical";

import type { ManagedPlugin } from "@/lib/api/native-plugins";

/* === Types === */

/** One row of programs that no installed plugin entry already lists. */
export type ProgramRow = {
  key: string;
  program: RuntimeInstalledPluginRecord;
  /** The contributed settings page any program of the row owns; null without one. */
  settingsSection: string | null;
};

/* === Public Functions === */

export function bundleSettingsPage(
  entry: ManagedPlugin,
  programs: RuntimeInstalledPluginRecord[],
  frame: Wire<SettingsFrame> | null,
): string | null {
  const owners = new Set(
    programs
      .filter((program) => program.owner?.kind === "bundle" && program.owner.installation === entry.name)
      .map((program) => program.plugin_id),
  );
  const section = frame?.sections.find(
    (candidate) => candidate.owner?.kind.case === "installedPluginId" && owners.has(candidate.owner.kind.value),
  );
  return section?.sectionId ?? entry.name;
}

/** Groups the programs into rows, one per host, bundle package or lone
    program, each carrying the settings page one of its programs contributes.
    A host row opens its own contributed page like any other plugin. */
export function programRows(
  programs: RuntimeInstalledPluginRecord[],
  entries: ManagedPlugin[],
  frame: Wire<SettingsFrame> | null,
): ProgramRow[] {
  const rows: ProgramRow[] = [];
  for (const program of programs) {
    const owner = program.owner;
    // An entry already has its own row, whether a managed bundle or an unmanaged registered plugin.
    const installation = owner?.kind === "bundle" ? owner.installation : program.plugin_id;
    if (installation && entries.some((entry) => entry.name === installation)) {
      continue;
    }
    const key = owner?.kind === "host" ? "host" : owner?.kind === "bundle" ? `bundle:${owner.package}` : programKey(program);
    const settingsSection =
      frame?.sections.find(
        (candidate) => candidate.owner?.kind.case === "installedPluginId" && candidate.owner.kind.value === program.plugin_id,
      )?.sectionId ?? null;
    const existing = rows.find((row) => row.key === key);
    if (existing) {
      existing.settingsSection ??= settingsSection;
    } else {
      rows.push({ key, program, settingsSection });
    }
  }
  return rows;
}

export function programKey(program: RuntimeInstalledPluginRecord): string {
  const owner = program.owner;
  return JSON.stringify([
    owner?.kind ?? null,
    owner?.kind === "bundle" ? owner.package : null,
    owner?.kind === "bundle" ? owner.installation : null,
    owner?.kind === "embedded" ? owner.generation : null,
    program.plugin_id,
  ]);
}

/** Where a program came from, with its version, as the row's muted column. */
export function programSource(program: RuntimeInstalledPluginRecord, brandLabel: string): string {
  const owner = program.owner;
  const origin =
    owner?.kind === "host"
      ? `${brandLabel} Built-in`
      : owner?.kind === "bundle"
        ? (owner.path ?? owner.package)
        : owner?.kind === "embedded"
          ? "Embedded"
          : "Unclassified";
  const version = program.version.trim();
  return version ? `${origin} · v${version}` : origin;
}
