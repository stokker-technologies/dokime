/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Native storage source settings adapter
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
import type { StorageSource } from "@/features/settings/settings-contracts";
import type { Wire } from "@/lib/rpc/canonical";
import * as rpc from "@/lib/rpc/generated/client";

type StorageSources = Wire<Awaited<ReturnType<typeof rpc.settingsListStorageSources>>>;

import type { SessionIdentity } from "@/lib/rpc/transport";
import { safeNumber } from "./native-values";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */
/* === Classes === */

/* === Public Functions === */

export async function fetchStorageSources(session: SessionIdentity): Promise<StorageSource[]> {
  return sources(await rpc.settingsListStorageSources(session, {}));
}

export async function reorderStorageSources(session: SessionIdentity, identifiers: string[]): Promise<StorageSource[]> {
  return sources(await rpc.settingsReorderStorageSources(session, { identifiers }));
}

export async function setStorageSourceEnabled(session: SessionIdentity, identifier: string, enabled: boolean): Promise<StorageSource[]> {
  return sources(await rpc.settingsSetStorageSourceEnabled(session, { identifier, enabled }));
}

/* === Private Functions === */
function sources(reply: StorageSources): StorageSource[] {
  return reply.sources.map((source) => ({
    identifier: source.identifier,
    adapter: source.adapter,
    priority: safeNumber(source.priority, "storage source priority"),
    enabled: source.enabled,
  }));
}

/* === Entry Point === */
