// Dokime — SPDX-License-Identifier: Apache-2.0
import { PluginState } from "@generated/host/model";
import type { ManagedPlugin } from "../../../lib/api/native-plugins";

type StatusPresentation = {
  label: string;
  reason: string | null;
  variant: "destructive" | "outline" | "success" | "warning";
};

/**
 * The running backend applies every change live: an installation shows what it
 * is doing now, and a refused change names what keeps running.
 */
export function statusPresentation(entry: ManagedPlugin): StatusPresentation {
  if (entry.failure) {
    return { label: "Failed", reason: entry.failure.message, variant: "destructive" };
  }
  if (!entry.value?.enabled) {
    return { label: "Disabled", reason: null, variant: "outline" };
  }
  switch (entry.state) {
    case PluginState.READY:
      return { label: "Ready", reason: null, variant: "success" };
    case PluginState.CONNECTING:
      return { label: "Starting", reason: null, variant: "outline" };
    case PluginState.OFFLINE:
      return { label: "Offline", reason: null, variant: "warning" };
    case PluginState.STOPPING:
      return { label: "Stopping", reason: null, variant: "outline" };
    case PluginState.STOPPED:
      return { label: "Stopped", reason: null, variant: "outline" };
    default:
      return { label: "Failed", reason: null, variant: "destructive" };
  }
}

/** Managed entries are started by the host; unmanaged ones by their author. */
export function entryPlacement(entry: ManagedPlugin): "Managed" | "Unmanaged" {
  return entry.placement?.kind.case === "remote" ? "Unmanaged" : "Managed";
}

/* === Private Functions === */

export function entrySource(entry: ManagedPlugin): string {
  const placement = entry.placement?.kind;
  const source = placement?.case === "remote" ? placement.value.url : placement?.case === "managed" ? "Bundle" : "Host Guest";
  const version = entry.description?.version;
  return version ? `${source} · v${version}` : source;
}
