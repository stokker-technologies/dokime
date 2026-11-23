// Dokime — SPDX-License-Identifier: Apache-2.0
import { useNavigate } from "@tanstack/react-router";
import { useNativeSession, useNativeSessionController } from "@/app/native-session/context";
import { SvelteMount, useSvelteSlots } from "@/components/SvelteMount";
import { useBrand } from "@/lib/brand-context";
import { useNavigationMode } from "@/shell/use-navigation-mode";
import { useUiStore } from "@/stores/ui-store";
import { ApplicationPreferences } from "./ApplicationPreferences";
import DisconnectedSettingsView from "./DisconnectedSettings.svelte";
import { PluginsSection } from "./sections/PluginsSection";
import { RemoteSection } from "./sections/RemoteSection";
import { SecretsSection } from "./sections/SecretsSection";
import AdvancedSwitch from "./SettingsAdvancedSwitch.svelte";
import type { SettingsSectionId } from "./settings-sections";
import SettingsView from "./SettingsSurface.svelte";
import { useSettingsSections } from "./use-settings-sections";

export function SettingsSurface({ section }: { section?: SettingsSectionId }) {
  const mode = useNavigationMode();
  const navigate = useNavigate();
  const advanced = useUiStore((state) => state.showAdvancedSettings);
  const settings = useSettingsSections();
  const owner = useNativeSessionController();
  const session = useNativeSession();
  const { label } = useBrand();
  const legacySection =
    section === "appearance" ? (
      <ApplicationPreferences advanced={advanced} />
    ) : section === "remote" ? (
      <RemoteSection advanced={advanced} />
    ) : section === "plugins" ? (
      <PluginsSection />
    ) : section === "secrets" ? (
      <SecretsSection />
    ) : null;
  const { rendered, portals } = useSvelteSlots({ legacySection });
  return (
    <>
      <SvelteMount
        component={SettingsView}
        props={{
          owner,
          session,
          mode,
          selected: section,
          label,
          ...settings,
          ...rendered,
          choose: (section) => void navigate({ to: "/settings/$section", params: { section } }),
        }}
      />
      {portals}
    </>
  );
}

export function SettingsAdvancedSwitch() {
  return <SvelteMount component={AdvancedSwitch} props={{}} />;
}

export function DisconnectedSettings({ close }: { close: () => void }) {
  const owner = useNativeSessionController();
  const advanced = useUiStore((state) => state.showAdvancedSettings);
  const { rendered, portals } = useSvelteSlots({ appearance: <ApplicationPreferences advanced={advanced} localOnly /> });
  return (
    <>
      <SvelteMount component={DisconnectedSettingsView} props={{ owner, close, ...rendered }} />
      {portals}
    </>
  );
}
