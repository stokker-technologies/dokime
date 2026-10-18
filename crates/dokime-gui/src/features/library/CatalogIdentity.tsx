/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The Library rows' second line: the readable identifier the CLI and Python
 * client accept, and the plugin that registered the suite. Internal UUIDs are
 * never rendered here. */

/* === External Imports === */

import { Clipboard } from "lucide-react";
import { toast } from "sonner";

/* === Local Imports === */

import { TAG_COLUMN_WIDTH, TagChip } from "@/components/suite-picker/SuiteTags";
import type { RuntimeInstalledPluginRecord } from "@/features/settings/installed-plugins/installed-plugin-contracts";
import { useBrand } from "@/lib/brand";
import { cn } from "@/lib/utils";

/* === Types === */

type CatalogIdentifierProps = {
  className?: string;
  /** What the identifier names, for the copy button's accessible name. */
  subject: string;
  value: string;
};

type SuiteProvenanceProps = {
  className?: string;
  plugin: RuntimeInstalledPluginRecord | undefined;
  source: string;
};

type OwnerDisplay = {
  label: string;
  title: string;
};

/* === Components === */

/** A monospace identifier with a copy affordance. */
export function CatalogIdentifier({ className, subject, value }: CatalogIdentifierProps) {
  return (
    <span className={cn("flex min-w-0 items-center gap-1 font-mono text-[11px] leading-4 text-muted-foreground", className)}>
      <span className="min-w-0 truncate" title={value}>
        {value}
      </span>
      <button
        aria-label={`Copy ${subject} identifier ${value}`}
        className="grid size-4 shrink-0 place-items-center rounded-sm hover:bg-muted hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
        onClick={(event) => {
          event.stopPropagation();
          void copyIdentifier(value);
        }}
        title="Copy identifier"
        type="button"
      >
        <Clipboard aria-hidden className="size-3" />
      </button>
    </span>
  );
}

/**
 * The registering plugin, how it reached this host and its version, in the
 * tag column's chip face so it sits on the same vertical as the tags above.
 */
export function SuiteProvenance({ className, plugin, source }: SuiteProvenanceProps) {
  const brand = useBrand();
  const owner = plugin ? ownerDisplay(plugin.owner, brand.label) : null;
  const pluginLabel = plugin?.label && plugin.label !== plugin.plugin_id ? `${plugin.label} · ${plugin.plugin_id}` : source;

  return (
    <div className={cn("flex min-w-0 flex-nowrap items-center gap-1 overflow-hidden", TAG_COLUMN_WIDTH, className)}>
      <TagChip className="min-w-0 shrink truncate font-mono" label={pluginLabel} title={`Registered by plugin ${pluginLabel}`} />
      {owner ? <TagChip label={owner.label} title={owner.title} /> : null}
      {plugin?.version ? <TagChip className="font-mono" label={versionLabel(plugin.version)} title="Plugin version" /> : null}
      {plugin && !plugin.enabled ? <TagChip label="Disabled" title="Plugin is disabled" /> : null}
    </div>
  );
}

/* === Public Functions === */

/** Searchable plugin text for one suite, so the filter matches provenance too. */
export function pluginSearchText(plugin: RuntimeInstalledPluginRecord | undefined, brandLabel: string): unknown[] {
  if (!plugin) {
    return [];
  }
  const owner = ownerDisplay(plugin.owner, brandLabel);
  return [plugin.plugin_id, plugin.label, plugin.version, owner?.label, plugin.owner?.kind === "bundle" ? plugin.owner.package : null];
}

/* === Private Functions === */

function ownerDisplay(owner: RuntimeInstalledPluginRecord["owner"], brandLabel: string): OwnerDisplay | null {
  switch (owner?.kind) {
    case "host":
      return { label: "Host", title: `Registered by the ${brandLabel} host` };
    case "embedded":
      return { label: "Built-in", title: `Built into this application (${owner.generation})` };
    case "bundle":
      return owner.installation
        ? { label: "Installed", title: `Installed bundle ${owner.package}` }
        : { label: "Configured", title: `Configured bundle ${owner.package}` };
    default:
      return null;
  }
}

function versionLabel(version: string): string {
  return /^v\d/i.test(version) ? version : `v${version}`;
}

async function copyIdentifier(value: string): Promise<void> {
  try {
    await navigator.clipboard.writeText(value);
    toast.success("Identifier copied");
  } catch {
    toast.error("Could not copy identifier");
  }
}
