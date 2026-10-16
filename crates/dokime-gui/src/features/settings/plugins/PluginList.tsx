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

import { PackagePlus, Trash2 } from "lucide-react";

/* === Local Imports === */

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Empty,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from "@/components/ui/empty";
import { PluginTable, type PluginTableRow } from "@/features/settings/plugins/PluginTable";
import { formatArtifactSize } from "@/lib/artifact-tree";
import { useBrand } from "@/lib/brand";
import type { PluginEntry } from "@/lib/types/view-contracts";

/* === Types === */

type PluginListProps = {
  entries: PluginEntry[];
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
  entries,
  onRemove,
  onOpenSettings,
  removingId,
  onSetEnabled,
  changingEnabled,
  settingsSectionOf,
}: PluginListProps) {
  const brand = useBrand();
  if (entries.length === 0) {
    return (
      <Empty className="min-h-44 border">
        <EmptyHeader>
          <EmptyMedia variant="icon">
            <PackagePlus />
          </EmptyMedia>
          <EmptyTitle>No Installed Plugins</EmptyTitle>
          <EmptyDescription>
            Drop a .dokime bundle above to inspect it and review its declared identity before
            install.
          </EmptyDescription>
        </EmptyHeader>
      </Empty>
    );
  }

  const rows = entries.map((entry): PluginTableRow => {
    const status = statusPresentation(entry, brand.label);
    const removalPending = entry.state === "removal_pending" && entry.consented_at !== null;
    const configurable = settingsSectionOf(entry) !== null;
    const name = entry.metadata?.identity.name ?? entry.install_id;
    const switchable = entry.state === "installed" || entry.state === "disabled";

    return {
      alert: status.reason ? (
        <p className="whitespace-pre-line text-xs text-destructive" role="alert">
          {status.reason}
        </p>
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
        <Button
          aria-label={`Remove ${name}`}
          disabled={removalPending || removingId === entry.install_id}
          onClick={() => onRemove(entry)}
          size="sm"
          variant="destructive"
        >
          <Trash2 />
          {removalPending
            ? "Removal queued"
            : removingId === entry.install_id
              ? "Removing…"
              : entry.consented_at === null
                ? "Discard"
                : "Remove"}
        </Button>
      ),
    };
  });

  return <PluginTable label="Installed Plugins" rows={rows} />;
}

/* === Public Functions === */

export function statusPresentation(entry: PluginEntry, brandLabel: string): StatusPresentation {
  if (entry.error) {
    return {
      label: entry.active ? "Active · Bundle unavailable" : "Unavailable",
      reason: entry.error,
      variant: "destructive",
    };
  }
  if (entry.state === "removal_pending") {
    return {
      label: entry.consented_at === null ? "Cleanup pending" : "Removal pending",
      reason: entry.active
        ? `Active until ${brandLabel} restarts.`
        : entry.last_boot?.state === "failed"
          ? entry.last_boot.error
          : null,
      variant: "warning",
    };
  }
  if (entry.state === "staged") {
    return { label: "Awaiting consent", reason: null, variant: "outline" };
  }
  if (entry.restart_required) {
    return {
      label: entry.active ? "Active · Pending restart" : "Pending restart",
      reason: null,
      variant: "warning",
    };
  }
  if (entry.active) return { label: "Active", reason: null, variant: "success" };
  if (entry.boot_error)
    return { label: "Failed", reason: entry.boot_error, variant: "destructive" };
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
