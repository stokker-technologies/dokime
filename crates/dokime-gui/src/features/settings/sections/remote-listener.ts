/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Live Remote Listener Commits Against The Host's Applied State
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
import type { Wire } from "@/lib/rpc/canonical";
import { normalizeRpcError } from "@/lib/rpc/errors";
import type * as rpc from "@/lib/rpc/generated/client";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

export type ListenerStatus = Wire<Awaited<ReturnType<typeof rpc.remoteReadListener>>>;
export type ListenerSettings = NonNullable<ListenerStatus["settings"]>;

/** One committed control; certificate identities change only through regeneration. */
export type ListenerChange = Partial<Pick<ListenerSettings, "address" | "authenticationEnabled" | "enabled" | "tlsEnabled">>;

type Commit = {
  status: ListenerStatus;
  change: ListenerChange;
  send: (settings: ListenerSettings) => Promise<ListenerStatus>;
  fail: (message: string) => void;
};

/* === Classes === */

/* === Public Functions === */

/** The listener runs only when the host reports a bound address; saved intent alone is not running. */
export function listenerRunning(status: ListenerStatus): boolean {
  return Boolean(status.listeningAddress);
}

/** Suggest an origin from saved settings without advertising a wildcard bind address. */
export function pairingEndpoint(settings: ListenerSettings): { endpoint: string; port: string } {
  const address = /^(\[[^\]]+\]|[^:]+):(\d+)$/.exec(settings.address);
  if (!address) return { endpoint: "", port: "" };
  const [, listenerHost, port] = address;
  const wildcard = listenerHost === "0.0.0.0" || /^\[[0:]+\]$/.test(listenerHost);
  const certificateHost = settings.certificateNames.find((name) => name !== "0.0.0.0" && !/^[0:]*:[0:]*$/.test(name));
  const localHost = settings.tlsEnabled ? undefined : listenerHost.startsWith("[") ? "[::1]" : "127.0.0.1";
  const host = wildcard ? (certificateHost ?? localHost) : listenerHost;
  if (!host) return { endpoint: "", port };
  const authority = host.includes(":") && !host.startsWith("[") ? `[${host}]` : host;
  return { endpoint: `${settings.tlsEnabled ? "https" : "http"}://${authority}:${port}`, port };
}

/** The host's applied settings with one committed change; never a local draft. */
export function listenerRequest(status: ListenerStatus, change: ListenerChange): ListenerSettings {
  const applied = appliedSettings(status);
  return { ...applied, ...change, certificateNames: applied.certificateNames };
}

/** Applies one change; on failure the host's state stands and the host error is surfaced. */
export async function commitListener({ status, change, send, fail }: Commit): Promise<ListenerStatus> {
  try {
    return await send(listenerRequest(status, change));
  } catch (error) {
    fail(listenerError(error));
    return status;
  }
}

export function certificateIssue(status: ListenerStatus): string | null {
  return appliedSettings(status).tlsEnabled && !status.certificatePem
    ? "TLS is on but this host has no certificate, so the listener cannot start. Generate one for this host's names."
    : null;
}

export function listenerError(error: unknown): string {
  const normalized = normalizeRpcError(error);
  return normalized instanceof Error && normalized.message.trim() ? normalized.message : "The host rejected the remote access change.";
}

/* === Private Functions === */

function appliedSettings(status: ListenerStatus): ListenerSettings {
  if (!status.settings) throw new Error("Backend omitted listener settings.");
  return status.settings;
}

/* === Entry Point === */
