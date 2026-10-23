/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The settings content column.
 *
 * The sidebar tree lists the sections, so this surface renders exactly one of
 * them and nothing else: no second navigation, no section list, no header that
 * repeats the page bar.
 */

/* === External Imports === */

import { useEffect } from "react";

/* === Local Imports === */

import { Button } from "@/components/ui/button";
import { ReportDefaultsCard } from "@/features/settings/ReportDefaultsCard";
import { SettingsCard } from "@/features/settings/SettingsCard";
import { SettingsGrid } from "@/features/settings/SettingsGrid";
import { StorageSourcesCard } from "@/features/settings/StorageSourcesCard";
import { AboutSection } from "@/features/settings/sections/AboutSection";
import { AccountUpdatesSection } from "@/features/settings/sections/AccountUpdatesSection";
import { AppearanceSection } from "@/features/settings/sections/AppearanceSection";
import { DiagnosticsSection } from "@/features/settings/sections/DiagnosticsSection";
import { GeneralSection } from "@/features/settings/sections/GeneralSection";
import { IdentitySection } from "@/features/settings/sections/IdentitySection";
import { PluginSettingsSection } from "@/features/settings/sections/PluginSettingsSection";
import { PluginsSection } from "@/features/settings/sections/PluginsSection";
import { RemoteSection } from "@/features/settings/sections/RemoteSection";
import { SecretsSection } from "@/features/settings/sections/SecretsSection";
import type { RuntimeSettingsFrame } from "@/features/settings/settings-contracts";
import { DEFAULT_SETTINGS_SECTION, type SettingsSectionId } from "@/features/settings/settings-sections";
import { useSettingsSections } from "@/features/settings/use-settings-sections";
import { usePageChrome } from "@/shell/page-chrome";

/* === Types === */

type SettingsSurfaceProps = {
  close: () => void;
  section?: SettingsSectionId;
};

/* === Components === */

/**
 * The selected settings section. The shell page bar carries the breadcrumb
 * title and the Back chevron, so this surface adds no second header and no
 * close action of its own: an X closes a window, not a panel.
 */
export function SettingsSurface({ close, section = DEFAULT_SETTINGS_SECTION }: SettingsSurfaceProps) {
  const settings = useSettingsSections();
  const frame = settings.frame;
  const meta = settings.sections.find((entry) => entry.id === section);

  usePageChrome({ title: `Settings \u203a ${meta?.label ?? section}` });

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape" && !event.defaultPrevented) {
        close();
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [close]);

  return (
    <div className="flex min-h-full min-w-0 flex-col">
      {/* Every section returns its cards as siblings so they land directly
          in the one settings grid. */}
      <div className="p-[var(--shell-inset)]">
        <SettingsGrid>
          <SectionContent frame={frame} loadError={settings.error} loading={settings.loading} retry={settings.retry} section={section} />
        </SettingsGrid>
      </div>
    </div>
  );
}

function SectionContent({
  frame,
  loadError,
  loading,
  retry,
  section,
}: {
  frame: RuntimeSettingsFrame | null;
  loadError: string | null;
  loading: boolean;
  retry: () => void;
  section: SettingsSectionId;
}) {
  if (section === "general")
    return (
      <>
        <GeneralSection />
        <ReportDefaultsCard />
        <StorageSourcesCard />
      </>
    );
  if (section === "appearance") return <AppearanceSection />;
  if (section === "remote") return <RemoteSection />;
  if (section === "plugins") return <PluginsSection />;
  if (section === "secrets") return <SecretsSection />;

  if (loading) {
    return (
      <SettingsCard title="Settings">
        <div className="px-4 py-3 text-sm text-muted-foreground">Loading settings...</div>
      </SettingsCard>
    );
  }

  if (loadError) {
    return (
      <SettingsCard title="Settings">
        <div className="grid justify-items-start gap-3 px-4 py-3">
          <p className="text-sm text-destructive">{loadError}</p>
          <Button onClick={retry} variant="outline">
            Try Again
          </Button>
        </div>
      </SettingsCard>
    );
  }

  switch (section) {
    case "identity":
      return <IdentitySection frame={frame} />;
    case "account-updates":
      return <AccountUpdatesSection />;
    case "diagnostics":
      return <DiagnosticsSection />;
    case "about":
      return <AboutSection />;
    default:
      return <PluginSettingsSection frame={frame} sectionId={section} />;
  }
}
