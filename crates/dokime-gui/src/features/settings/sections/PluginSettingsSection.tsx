/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Installed-plugin contributed settings section. */

/* === Local Imports === */

import { PluginSettingsCards } from "@/features/settings/PluginSettingsCards";
import { SettingsCard } from "@/features/settings/SettingsCard";
import type { RuntimeSettingsFrame } from "@/features/settings/settings-contracts";

/* === Types === */

type PluginSettingsSectionProps = {
  frame: RuntimeSettingsFrame | null;
  sectionId: string;
};

/* === Components === */

export function PluginSettingsSection({ frame, sectionId }: PluginSettingsSectionProps) {
  const cards = frame?.cards.filter((card) => card.section_id === sectionId) ?? [];

  if (cards.length === 0) {
    return (
      <SettingsCard title="Settings">
        <div className="px-4 py-3 text-sm text-muted-foreground">No settings cards.</div>
      </SettingsCard>
    );
  }

  return <PluginSettingsCards cards={cards} />;
}
