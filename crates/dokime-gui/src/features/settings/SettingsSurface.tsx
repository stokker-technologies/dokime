/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* A Single Settings Panel With Shared Navigation And Viewer Preferences. */

/* === External Imports === */

import { useNavigate } from "@tanstack/react-router";
import { useNavigationMode } from "@/shell/use-sidebar-state";
import { useId, useMemo, useState } from "react";
import { Switch } from "@/components/ui/switch";
import { useUiStore } from "@/stores/ui-store";

/* === Local Imports === */

import { Button } from "@/components/ui/button";
import { SettingsCard } from "@/features/settings/SettingsCard";
import { SettingsGrid } from "@/features/settings/SettingsGrid";
import { StorageSourcesCard } from "@/features/settings/StorageSourcesCard";
import { AboutSection } from "@/features/settings/sections/AboutSection";
import { DiagnosticsSection } from "@/features/settings/sections/DiagnosticsSection";
import { BootstrapPaths } from "@/features/settings/sections/BootstrapPaths";
import { PluginsSection } from "@/features/settings/sections/PluginsSection";
import { RemoteSection } from "@/features/settings/sections/RemoteSection";
import { SecretsSection } from "@/features/settings/sections/SecretsSection";
import type { RuntimeSettingsFrame } from "@/features/settings/settings-contracts";
import { DEFAULT_SETTINGS_SECTION, SETTINGS_SECTIONS, type SettingsSectionId } from "@/features/settings/settings-sections";
import { useSettingsSections } from "@/features/settings/use-settings-sections";
import { PanelHeader } from "@/shell/PageShell";
import { OrganizationPackRow } from "./sections/OrganizationPackRow";
import { usePageChrome } from "@/shell/page-chrome";
import { ApplicationPreferences } from "./ApplicationPreferences";
import { BackendsTable } from "./BackendsTable";
import { PluginSettingsCards } from "./PluginSettingsCards";

/* === Types === */

type SettingsSurfaceProps = {
  section?: SettingsSectionId;
};

/* === Components === */

/** The Selected Section Of The Shared Settings Panel. */
export function SettingsSurface({ section: selected }: SettingsSurfaceProps) {
  const mode = useNavigationMode();
  const navigate = useNavigate();
  const section = selected ?? DEFAULT_SETTINGS_SECTION;
  const advanced = useUiStore((state) => state.showAdvancedSettings);
  const settings = useSettingsSections();
  const frame = settings.frame;

  const actions = useMemo(() => <SettingsAdvancedSwitch />, []);
  usePageChrome({
    title:
      mode === "phone" && !selected ? "Settings" : `Settings › ${SETTINGS_SECTIONS.find((item) => item.id === section)?.label ?? section}`,
    overflow: actions,
  });

  return (
    <div className="flex min-h-full min-w-0 flex-col tablet:flex-row">
      {mode !== "phone" || !selected ? (
        <nav
          aria-label="Settings Sections"
          className="grid shrink-0 content-start gap-2 border-b p-4 tablet:w-48 tablet:border-r tablet:border-b-0"
        >
          {SETTINGS_SECTIONS.map((item) => (
            <Button
              key={item.id}
              variant={section === item.id && (mode !== "phone" || selected) ? "secondary" : "ghost"}
              className="justify-start"
              onClick={() => void navigate({ to: "/settings/$section", params: { section: item.id } })}
            >
              {item.label}
            </Button>
          ))}
        </nav>
      ) : null}
      {/* Every section returns its cards as siblings so they land directly
          in the one settings grid. */}
      {mode !== "phone" || selected ? (
        <div className="min-w-0 flex-1 p-4">
          <SettingsGrid>
            <SectionContent
              advanced={advanced}
              frame={frame}
              loadError={settings.error}
              loading={settings.loading}
              retry={settings.retry}
              section={section}
            />
          </SettingsGrid>
        </div>
      ) : null}
    </div>
  );
}

function SectionContent({
  advanced,
  frame,
  loadError,
  loading,
  retry,
  section,
}: {
  advanced: boolean;
  frame: RuntimeSettingsFrame | null;
  loadError: string | null;
  loading: boolean;
  retry: () => void;
  section: SettingsSectionId;
}) {
  if (section === "organization")
    return (
      <>
        <SettingsCard title="Organization">
          <OrganizationPackRow />
        </SettingsCard>
        <PluginSettingsCards cards={frame?.cards.filter((card) => card.section_id === "identity") ?? []} />
      </>
    );
  if (section === "backends") return <BackendsTable />;
  if (section === "appearance") return <ApplicationPreferences advanced={advanced} />;
  if (section === "remote") return <RemoteSection advanced={advanced} />;
  if (section === "plugins") return <PluginsSection />;
  if (section === "secrets") return <SecretsSection />;
  if (section === "diagnostics")
    return (
      <>
        <BootstrapPaths />
        <DiagnosticsSection />
        <StorageSourcesCard />
      </>
    );
  if (section === "about") return <AboutSection />;

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

  return (
    <SettingsCard title="Settings">
      <p role="alert" className="p-4">
        This settings section is unavailable.
      </p>
    </SettingsCard>
  );
}

/** Settings Available Before A Backend Connection Has Been Established. */
export function SettingsAdvancedSwitch() {
  const id = useId();
  const advanced = useUiStore((state) => state.showAdvancedSettings);
  return (
    <div className="flex items-center gap-3 text-sm">
      <label htmlFor={id}>Show advanced</label>
      <Switch
        id={id}
        checked={advanced}
        onCheckedChange={(showAdvancedSettings) => useUiStore.getState().setUiState({ showAdvancedSettings })}
      />
    </div>
  );
}

export function DisconnectedSettings({ close }: { close: () => void }) {
  const advanced = useUiStore((state) => state.showAdvancedSettings);
  const [section, setSection] = useState(DEFAULT_SETTINGS_SECTION);
  return (
    <main className="flex h-dvh flex-col bg-background p-2">
      <div className="flex min-h-0 flex-1 flex-col overflow-hidden rounded-lg border bg-card">
        <PanelHeader
          title="Settings"
          onBack={close}
          overflow={
            <div className="px-2 py-1">
              <SettingsAdvancedSwitch />
            </div>
          }
        />
        <div className="grid min-h-0 content-start gap-4 overflow-y-auto p-[var(--shell-inset)]">
          <nav aria-label="Settings Sections" className="flex gap-2">
            {SETTINGS_SECTIONS.filter((item) => ["backends", "appearance"].includes(item.id)).map((item) => (
              <Button key={item.id} variant="outline" aria-pressed={section === item.id} onClick={() => setSection(item.id)}>
                {item.label}
              </Button>
            ))}
          </nav>
          <SettingsGrid>
            {section === "backends" ? <BackendsTable /> : <ApplicationPreferences advanced={advanced} localOnly />}
          </SettingsGrid>
        </div>
      </div>
    </main>
  );
}
