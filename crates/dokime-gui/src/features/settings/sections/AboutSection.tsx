/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import { useState } from "react";
import { DesktopUpdateDialog } from "@/components/DesktopUpdateDialog";
import { Button } from "@/components/ui/button";
import { SettingsCard } from "@/features/settings/SettingsCard";
import { SettingsRow } from "@/features/settings/SettingsRow";
import { useBrand } from "@/lib/brand";
import { isTauriDesktop } from "@/lib/desktop-open";

export function AboutSection() {
  const brand = useBrand();
  const [updatesOpen, setUpdatesOpen] = useState(false);

  return (
    <>
      <SettingsCard title="About">
        <SettingsRow label="Application">{brand.label}</SettingsRow>
        <SettingsRow label="Version">
          <span className="font-mono text-sm">v{__APP_VERSION__}</span>
        </SettingsRow>

        <SettingsRow description="Dokime © Stokker Technologies, licensed under the Apache License, Version 2.0." label="License">
          Apache License 2.0
        </SettingsRow>
        <div className="flex justify-end px-4 py-3">
          <Button variant="outline" disabled={!isTauriDesktop()} onClick={() => setUpdatesOpen(true)}>
            Check For Updates
          </Button>
        </div>
      </SettingsCard>
      <DesktopUpdateDialog open={updatesOpen} onOpenChange={setUpdatesOpen} />
    </>
  );
}
