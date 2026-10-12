/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* One row shape for every plugin the Plugins page lists.
 *
 * An installed bundle and a program the host or the launch configuration
 * brought in are the same four columns — name, source, enabled, settings — at
 * the same height, so the page reads as one list rather than as three
 * differently shaped ones. The row itself opens the plugin's page; the switch
 * and the settings button keep their own hit areas above it.
 */

/* === External Imports === */

import { Settings2 } from "lucide-react";
import type { ReactNode } from "react";

/* === Local Imports === */

import { Button } from "@/components/ui/button";
import { Switch } from "@/components/ui/switch";
import { cn } from "@/lib/utils";

/* === Types === */

export type PluginRowProps = {
  /** Rendered under the name when the plugin reports a problem. */
  alert?: ReactNode;
  /** Trailing content on the name column, such as a state pill. */
  badge?: ReactNode;
  /** Absent when this plugin cannot be enabled or disabled here. */
  enabled?: { checked: boolean; disabled?: boolean; onChange: (next: boolean) => void };
  name: string;
  /** Opens the plugin's own settings page; absent when it contributes none. */
  onOpenSettings?: () => void;
  /** Where the plugin came from, with its version. One muted line. */
  source: string;
  /** Rendered after the settings button, on the row's shared trailing column. */
  trailing?: ReactNode;
};

/* === Constants === */

/** Name, source, the enabled slot, the settings slot, then the row's own actions. */
export const PLUGIN_ROW_GRID =
  "grid grid-cols-[minmax(0,2fr)_minmax(0,3fr)_3.25rem_2.25rem_auto] items-center gap-x-4";

/* === Components === */

export function PluginRow({
  alert,
  badge,
  enabled,
  name,
  onOpenSettings,
  source,
  trailing,
}: PluginRowProps) {
  const configurable = Boolean(onOpenSettings);

  return (
    <li
      className={cn(
        "relative min-h-[var(--settings-row-height)] px-4 py-1.5",
        PLUGIN_ROW_GRID,
        configurable && "hover:bg-muted/50",
      )}
    >
      {/* Pointer convenience only: the settings button below is the one
          keyboard and screen-reader control, so the overlay stays out of the
          tab order rather than doubling every row's stops. */}
      {onOpenSettings ? (
        <button
          aria-hidden
          className="absolute inset-0 cursor-pointer"
          onClick={onOpenSettings}
          tabIndex={-1}
          type="button"
        />
      ) : null}

      <div className="pointer-events-none relative z-10 grid min-w-0 gap-0.5">
        <div className="flex min-w-0 items-center gap-2">
          <span className="truncate text-sm font-medium" title={name}>
            {name}
          </span>
          {badge}
        </div>
        {alert}
      </div>

      <span className="relative z-10 min-w-0 truncate text-xs text-muted-foreground" title={source}>
        {source}
      </span>

      <span className="relative z-10 flex items-center justify-start">
        {enabled ? (
          <Switch
            aria-label={`${enabled.checked ? "Disable" : "Enable"} ${name}`}
            checked={enabled.checked}
            disabled={enabled.disabled}
            onCheckedChange={enabled.onChange}
          />
        ) : null}
      </span>

      {/* A disabled control swallows pointer events, so the reason it is
          disabled has to live on the wrapper. */}
      <span
        className="relative z-10 flex items-center justify-center"
        title={configurable ? "Plugin Settings" : "No settings"}
      >
        <Button
          aria-label="Plugin Settings"
          disabled={!configurable}
          onClick={onOpenSettings}
          size="icon-sm"
          type="button"
          variant="ghost"
        >
          <Settings2 />
        </Button>
      </span>

      <span className="relative z-10 flex items-center justify-end gap-2">{trailing}</span>
    </li>
  );
}
