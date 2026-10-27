/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Installed-plugin status badges. */

/* === Local Imports === */

import { Badge } from "@/components/ui/badge";
import { type InstalledPluginRuntimeKind, RUNTIME_LABELS } from "@/features/settings/installed-plugins/installed-plugin-contracts";
import { cn } from "@/lib/utils";

/* === Constants === */

const RUNTIME_CLASS: Record<InstalledPluginRuntimeKind, string> = {
  code: "border-info/30 bg-info/10 text-info",
};

const STATUS_DISPLAY: Record<string, { className: string; label: string }> = {
  registered: {
    className: "border-success/30 bg-success/10 text-success",
    label: "Registered",
  },
  disabled: { className: "text-muted-foreground", label: "Disabled" },
};

/* === Components === */

export function InstalledPluginRuntimeBadge({ runtime }: { runtime: string }) {
  const kind = runtime as InstalledPluginRuntimeKind;
  return (
    <Badge className={cn("h-5 rounded px-2 text-xs font-medium", RUNTIME_CLASS[kind])} variant="secondary">
      {RUNTIME_LABELS[kind]}
    </Badge>
  );
}

export function InstalledPluginStatusBadge({ status }: { status: string }) {
  const display = STATUS_DISPLAY[status] ?? { className: "text-muted-foreground", label: status };
  return (
    <Badge className={cn("h-5 rounded px-2 text-xs font-medium", display.className)} variant="outline">
      {display.label}
    </Badge>
  );
}
