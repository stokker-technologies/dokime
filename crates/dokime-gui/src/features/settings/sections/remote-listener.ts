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
  return error instanceof Error && error.message ? error.message : "The host rejected the remote access change.";
}

/** Typed identities, split on commas and whitespace. */
export function certificateNames(text: string): string[] {
  return text.split(/[\s,]+/).filter(Boolean);
}

/* === Private Functions === */

function appliedSettings(status: ListenerStatus): ListenerSettings {
  if (!status.settings) throw new Error("Backend omitted listener settings.");
  return status.settings;
}

/* === Entry Point === */
