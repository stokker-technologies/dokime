/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Local Imports === */

import { SettingsCard } from "@/features/settings/SettingsCard";
import { SettingsRow } from "@/features/settings/SettingsRow";
import { useBrand } from "@/lib/brand";

/* === Components === */

export function AboutSection() {
  const brand = useBrand();

  return (
    <>
      <SettingsCard title="About">
        <SettingsRow label="Application">{brand.label}</SettingsRow>
        <SettingsRow label="Version">
          <span className="font-mono text-sm">v{__APP_VERSION__}</span>
        </SettingsRow>
        <SettingsRow label="Engine">Rust runtime · Tauri desktop shell</SettingsRow>
      </SettingsCard>

      <SettingsCard title="Legal">
        <SettingsRow description="© Stokker Technologies. All rights reserved." label="License" />
      </SettingsCard>
    </>
  );
}
