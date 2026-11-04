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

import { Trash2 } from "lucide-react";
import type { ReactNode } from "react";

/* === Local Imports === */

import { Badge } from "@/components/ui/badge";
import { FailureConsequences } from "@/components/FailureConsequences";
import { DropdownMenuItem } from "@/components/ui/dropdown-menu";
import { PluginTable, type PluginTableRow } from "@/features/settings/plugins/PluginTable";
import { formatArtifactSize } from "@/lib/artifact-tree";
import type { PluginEntry } from "@/lib/types/view-contracts";

/* === Types === */

type PluginListProps = {
  actions?: ReactNode;
  entries: PluginEntry[];
  builtInRows?: PluginTableRow[];
  onRemove: (entry: PluginEntry) => void;
  /** Opens the plugin's own settings page; null when it contributes none. */
  onOpenSettings: (entry: PluginEntry) => void;
  removingId: string | null;
  onSetEnabled: (entry: PluginEntry, enabled: boolean) => void;
  changingEnabled: boolean;
  /** The settings section a plugin contributed, or null. */
  settingsSectionOf: (entry: PluginEntry) => string | null;
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
  const rows = entries.map((entry): PluginTableRow => {
    const status = statusPresentation(entry);
    const removalPending = entry.state === "removal_pending";
    const configurable = settingsSectionOf(entry) !== null;
    const name = entry.metadata?.identity.name ?? entry.install_id;
    const switchable = entry.state === "installed" || entry.state === "disabled";

    return {
      alert: status.reason ? (
        <div>
          <p role="alert" className="text-sm text-destructive">
            {status.reason}
          </p>
          {entry.last_boot?.state === "failed" ? <FailureConsequences items={entry.last_boot.failure.consequences} /> : null}
        </div>
      ) : null,
      badge: <Badge variant={status.variant}>{status.label}</Badge>,
      enabled: switchable
        ? {
            checked: entry.state !== "disabled",
            disabled: changingEnabled,
            onChange: (next) => onSetEnabled(entry, next),
          }
        : undefined,
      key: entry.install_id,
      name,
      onOpenSettings: configurable ? () => onOpenSettings(entry) : undefined,
      source: entrySource(entry),
      trailing: (
        <DropdownMenuItem
          aria-label={`Remove ${name}`}
          disabled={removalPending || removingId === entry.install_id}
          onSelect={() => onRemove(entry)}
          variant="destructive"
        >
          <Trash2 />
          {removalPending ? "Removed" : removingId === entry.install_id ? "Removing…" : "Remove"}
        </DropdownMenuItem>
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
export function statusPresentation(entry: PluginEntry): StatusPresentation {
  const failure = entry.last_boot?.state === "failed" ? entry.last_boot.failure.message : null;
  if (entry.error) {
    return {
      label: entry.active ? "Active · Bundle Unavailable" : "Unavailable",
      reason: entry.error,
      variant: "destructive",
    };
  }
  if (entry.restart === "restarting") return { label: "Restarting…", reason: null, variant: "outline" };
  if (entry.restart === "pending") {
    return {
      label: entry.active ? "Active · Restart Pending" : "Restart Pending",
      reason: "Applies when the running execution finishes.",
      variant: "warning",
    };
  }
  if (entry.state === "removal_pending") {
    return entry.boot_failed
      ? { label: "Refused", reason: failure, variant: "destructive" }
      : { label: "Removed", reason: null, variant: "outline" };
  }
  if (entry.active && failure) return { label: "Active · Restart Refused", reason: failure, variant: "warning" };
  if (entry.active) return { label: "Active", reason: null, variant: "success" };
  if (entry.boot_failed) return { label: "Failed", reason: failure, variant: "destructive" };
  if (entry.state === "installed") return { label: "Not Running", reason: null, variant: "outline" };
  return { label: "Disabled", reason: null, variant: "outline" };
}

/* === Private Functions === */

/** The bundle's version and size, as the row's one muted column. */
function entrySource(entry: PluginEntry): string {
  if (!entry.metadata) {
    return "Bundle Unavailable";
  }
  return `Installed Bundle · v${entry.metadata.identity.version} · ${formatArtifactSize(entry.metadata.size_bytes)}`;
}
