// Dokime — SPDX-License-Identifier: Apache-2.0
import { useNativeSession } from "@/app/native-session/context";
import { SvelteMount } from "@/components/SvelteMount";
import { useBrand } from "@/lib/brand-context";
import PluginSettingsView from "./PluginSettingsSurface.svelte";

export function PluginSettingsSurface({ pluginId }: { pluginId: string }) {
  const session = useNativeSession();
  const { label } = useBrand();
  return <SvelteMount component={PluginSettingsView} props={{ session, pluginId, label }} />;
}
