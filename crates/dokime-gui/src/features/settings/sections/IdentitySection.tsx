/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Local Imports === */

import { PluginSettingsCards } from "@/features/settings/PluginSettingsCards";
import { SettingsCard } from "@/features/settings/SettingsCard";
import { SettingsRow } from "@/features/settings/SettingsRow";
import type { RuntimeSettingsFrame } from "@/features/settings/settings-contracts";

/* === Types === */

type IdentitySectionProps = { frame: RuntimeSettingsFrame | null };

/* === Components === */

export function IdentitySection({ frame }: IdentitySectionProps) {
  const cards = frame?.cards.filter((card) => card.section_id === "identity") ?? [];

  return (
    <div className="grid gap-6">
      <SettingsCard title="Identity">
        <SettingsRow
          control={<span className="text-sm text-muted-foreground">Dokime</span>}
          description="Local desktop runtime"
          label="Host"
        />
      </SettingsCard>
      <PluginSettingsCards cards={cards} />
    </div>
  );
}
