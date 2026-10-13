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

import { useNavigate } from "@tanstack/react-router";

/* === Local Imports === */

import type { RuntimeInstalledPluginRecord } from "@/features/settings/installed-plugins/installed-plugin-contracts";
import { PluginRow } from "@/features/settings/plugins/PluginRow";
import { useSettingsSections } from "@/features/settings/use-settings-sections";

/* === Public Functions === */

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
export function programSource(program: RuntimeInstalledPluginRecord): string {
  const owner = program.owner;
  const origin =
    owner?.kind === "host"
      ? "Dokime Host"
      : owner?.kind === "bundle"
        ? (owner.path ?? owner.package)
        : owner?.kind === "embedded"
          ? "Embedded"
          : "Unclassified";
  const version = program.version.trim();
  return version ? `${origin} · v${version}` : origin;
}

/* === Components === */

export function PluginPrograms({ programs }: { programs: RuntimeInstalledPluginRecord[] }) {
  const navigate = useNavigate();
  const settings = useSettingsSections();

  const sectionOf = (program: RuntimeInstalledPluginRecord): string | null =>
    (settings.frame?.sections ?? []).find(
      (candidate) =>
        candidate.owner.kind === "installed_plugin" &&
        candidate.owner.plugin_id === program.plugin_id,
    )?.section_id ?? null;

  return (
    <ul className="divide-y overflow-hidden rounded-lg border bg-card">
      {programs.map((program) => {
        const pluginId = sectionOf(program);
        return (
          <PluginRow
            key={programKey(program)}
            name={program.label}
            onOpenSettings={
              pluginId
                ? () => void navigate({ params: { pluginId }, to: "/settings/plugins/$pluginId" })
                : undefined
            }
            source={programSource(program)}
          />
        );
      })}
    </ul>
  );
}
