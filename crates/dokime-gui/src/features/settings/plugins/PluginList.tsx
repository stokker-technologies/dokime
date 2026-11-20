/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The installed plugin bundles, one aligned row each.
 *
 * Name, source, enabled, settings, actions: the same columns the host and launch
 * programs use, so the whole page is one list. The bundle's identity digest
 * and its programs belong on the plugin's own page, not in a second block
 * nested inside this row.
 */

/* === External Imports === */

import { useNativeSessionOptional } from "@/app/native-session/context";
import { pluginStartFailure } from "@/features/settings/plugins/plugin-queries";
import { ErrorCard } from "@/components/ErrorCard";

import { RotateCw, Trash2 } from "lucide-react";
import type { ReactNode } from "react";

/* === Local Imports === */

import { Badge } from "@/components/ui/badge";

import { Button } from "@/components/ui/button";
import { useReloadPluginMutation } from "@/features/settings/plugins/plugin-queries";
import { PluginTable, type PluginTableRow } from "@/features/settings/plugins/PluginTable";
import { PluginState } from "@host/model";
import type { ManagedPlugin } from "@/lib/api/native-plugins";

/* === Types === */

type PluginListProps = {
  actions?: ReactNode;
  entries: ManagedPlugin[];
  builtInRows?: PluginTableRow[];
  onRemove: (entry: ManagedPlugin) => void;
  /** Opens the plugin's own settings page; null when it contributes none. */
  onOpenSettings: (entry: ManagedPlugin) => void;
  removingId: string | null;
  onSetEnabled: (entry: ManagedPlugin, enabled: boolean) => void;
  changingEnabled: boolean;
  /** The settings section a plugin contributed, or null. */
  settingsSectionOf: (entry: ManagedPlugin) => string | null;
};

type StatusPresentation = {
  label: string;
  reason: string | null;
  variant: "destructive" | "outline" | "success" | "warning";
};

/* === Components === */

export function PluginList({
  actions,
  entries,
  builtInRows = [],
  onRemove,
  onOpenSettings,
  removingId,
  onSetEnabled,
  changingEnabled,
  settingsSectionOf,
}: PluginListProps) {
  const reload = useReloadPluginMutation();
  const generation = useNativeSessionOptional()?.identity.generation ?? "";
  const rows = entries.map((entry): PluginTableRow => {
    const status = statusPresentation(entry);

    const configurable = settingsSectionOf(entry) !== null;
    const name = entry.description?.id || entry.name;
    const switchable = entry.value !== undefined;

    return {
      alert: status.reason ? (
        <ErrorCard
          failure={pluginStartFailure(entry, generation)}
          onRetry={entry.value?.enabled ? () => reload.mutate(entry.name) : undefined}
          retryLabel="Restart Plugin"
          retryDisabled={reload.isPending}
        />
      ) : null,
      badge: <Badge variant={status.variant}>{status.label}</Badge>,
      enabled: switchable
        ? {
            checked: entry.value?.enabled ?? false,
            disabled: changingEnabled,
            onChange: (next) => onSetEnabled(entry, next),
          }
        : undefined,
      key: entry.name,
      name,
      onOpenSettings: configurable ? () => onOpenSettings(entry) : undefined,
      source: entrySource(entry),
      trailing: (
        <>
          <Button
            aria-label={`Reload ${name}`}
            title="Reload"
            size="icon-sm"
            variant="ghost"
            disabled={!entry.value?.enabled || entry.state === PluginState.STOPPING || reload.isPending}
            onClick={() => reload.mutate(entry.name)}
          >
            <RotateCw />
          </Button>
          <Button
            aria-label={`Delete ${name}`}
            title="Delete"
            size="icon-sm"
            variant="ghost"
            disabled={removingId === entry.name}
            onClick={() => onRemove(entry)}
          >
            <Trash2 />
          </Button>
        </>
      ),
    };
  });

  return <PluginTable actions={actions} label="Plugins" rows={[...builtInRows, ...rows]} />;
}

/* === Public Functions === */

/**
 * The running backend applies every change live: an installation shows what it
 * is doing now, and a refused change names what keeps running.
 */
export function statusPresentation(entry: ManagedPlugin): StatusPresentation {
  if (entry.failure) return { label: "Unavailable", reason: entry.failure.message, variant: "destructive" };
  if (!entry.value?.enabled) return { label: "Disabled", reason: null, variant: "outline" };
  switch (entry.state) {
    case PluginState.READY:
      return { label: "Active", reason: null, variant: "success" };
    case PluginState.CONNECTING:
      return { label: "Connecting…", reason: null, variant: "outline" };
    case PluginState.STOPPING:
      return { label: "Stopping…", reason: null, variant: "outline" };
    case PluginState.STOPPED:
      return { label: "Stopped", reason: null, variant: "outline" };
    default:
      return { label: "Unavailable", reason: null, variant: "warning" };
  }
}

/* === Private Functions === */

function entrySource(entry: ManagedPlugin): string {
  const placement = entry.placement?.kind;
  const source = placement?.case === "remote" ? placement.value.url : placement?.case === "managed" ? "Managed Bundle" : "Host Guest";
  const version = entry.description?.version;
  return version ? `${source} · v${version}` : source;
}
