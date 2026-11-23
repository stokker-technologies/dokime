// Dokime — SPDX-License-Identifier: Apache-2.0
import { useQueryClient } from "@tanstack/react-query";
import { useLocation, useNavigate } from "@tanstack/react-router";
import { useNativeSession, useNativeSessionController } from "@/app/native-session/context";
import { SvelteMount, useSvelteSlots } from "@/components/SvelteMount";
import { useBrand } from "@/lib/brand-context";
import { useNavigationMode } from "@/shell/use-navigation-mode";
import DisconnectedSettingsView from "./DisconnectedSettings.svelte";
import { PluginsSection } from "./sections/PluginsSection";
import AdvancedSwitch from "./SettingsAdvancedSwitch.svelte";
import type { SettingsSectionId } from "./settings-sections";
import SettingsView from "./SettingsSurface.svelte";
import { useSettingsSections } from "./use-settings-sections";

export function SettingsSurface({ section }: { section?: SettingsSectionId }) {
  const mode = useNavigationMode();
  const navigate = useNavigate();
  const hash = useLocation({ select: (location) => location.hash });
  const settings = useSettingsSections();
  const owner = useNativeSessionController();
  const session = useNativeSession();
  const { label } = useBrand();
  const legacySection = section === "plugins" ? <PluginsSection /> : null;
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
          hash,
          clearHash: () => void navigate({ hash: "", replace: true }),
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
  const queryClient = useQueryClient();
  return <SvelteMount component={DisconnectedSettingsView} props={{ owner, close, queryClient }} />;
}
