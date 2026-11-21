/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
import { useEffect, useState } from "react";

// Local Imports
import { useNativeSessionOptional } from "@/app/native-session/context";
import { DesktopUpdateDialog } from "@/components/DesktopUpdateDialog";
import { copyTraceabilityText } from "@dokime/document/components/traceability/clipboard";
import { Button } from "@dokime/plots/components/ui/button";
import { Tooltip, TooltipContent, TooltipTrigger } from "@dokime/plots/components/ui/tooltip";
import { buildLabel } from "@/features/settings/sections/build-version";
import { SettingsCard } from "@/features/settings/SettingsCard";
import { SettingsRow } from "@/features/settings/SettingsRow";
import { useBrand } from "@dokime/document/lib/brand";
import { isTauriDesktop } from "@dokime/document/lib/surface-environment";
import { commands } from "@/lib/desktop/commands";
import type { BuildIdentity } from "@dokime/document/lib/contracts/connection";

/* === Components === */

export function AboutSection() {
  const brand = useBrand();
  const backend = useNativeSessionOptional()?.snapshot.connection.backend ?? null;
  const desktop = useDesktopBuild();
  const build = desktop ?? { productVersion: __APP_VERSION__, sourceRevision: null, channel: null };
  const versionMatches = backend?.productVersion === build.productVersion;
  const revisionMatches = (backend?.sourceRevision ?? "unknown") === (build.sourceRevision ?? "unknown");
  const channelMatches = (backend?.channel ?? "") === (build.channel ?? "");
  const backendDiffers = backend !== null && !(versionMatches && revisionMatches && channelMatches);
  const [updatesOpen, setUpdatesOpen] = useState(false);

  return (
    <>
      <SettingsCard title="About">
        <SettingsRow label="Application">{brand.label}</SettingsRow>
        {desktop ? (
          <SettingsRow description="The build of this desktop application." label="Desktop Version">
            <BuildVersion build={desktop} />
          </SettingsRow>
        ) : (
          <SettingsRow label="Version">
            <BuildVersion build={build} />
          </SettingsRow>
        )}
        {backend && backendDiffers ? (
          <SettingsRow description="The build of the connected backend, which may run elsewhere." label="Backend Version">
            <BuildVersion build={backend} />
          </SettingsRow>
        ) : null}

        <SettingsRow description="Dokime © Stokker Technologies, licensed under the Apache License, Version 2.0." label="License">
          Apache License 2.0
        </SettingsRow>
        <div className="flex justify-end px-4 py-3">
          <Button data-testid="desktop-updates-open" variant="outline" disabled={!isTauriDesktop()} onClick={() => setUpdatesOpen(true)}>
            Check for Updates
          </Button>
        </div>
      </SettingsCard>
      <DesktopUpdateDialog open={updatesOpen} onOpenChange={setUpdatesOpen} build={build} />
    </>
  );
}

function BuildVersion({ build }: { build: BuildIdentity }) {
  const label = buildLabel(build);
  const commit = label.commit;
  return (
    <span className="font-mono text-sm">
      {`${label.version} (`}
      {commit ? (
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              variant="link"
              size="inline"
              aria-label={`Copy Commit ${commit} to Clipboard`}
              onClick={() => void copyTraceabilityText(commit)}
              type="button"
            >
              {label.revision}
            </Button>
          </TooltipTrigger>
          <TooltipContent className="break-all" side="top">{`Copy: ${commit}`}</TooltipContent>
        </Tooltip>
      ) : (
        label.revision
      )}
      , <span className="text-muted-foreground">{label.channel}</span>)
    </span>
  );
}

/* === Hooks === */

function useDesktopBuild(): BuildIdentity | null {
  const [build, setBuild] = useState<BuildIdentity | null>(null);
  useEffect(() => {
    if (!isTauriDesktop()) return;
    let current = true;
    // Without the native build, the row keeps the bundled frontend version.
    void commands.desktopBuild().then(
      (value) => current && setBuild(value),
      () => undefined,
    );
    return () => {
      current = false;
    };
  }, []);
  return build;
}
