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
import { required } from "@/lib/rpc/canonical";
import type { Wire } from "@/lib/rpc/canonical";
import * as rpc from "@/lib/host/client";

type StorageSources = Wire<Awaited<ReturnType<typeof rpc.storageSourcesGet>>>;

import type { SessionIdentity } from "@/lib/rpc/transport";
import { safeNumber } from "./native-values";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */
/* === Classes === */

/* === Public Functions === */

export async function fetchStorageSources(session: SessionIdentity): Promise<StorageSource[]> {
  return sources(await rpc.storageSourcesGet(session, {}));
}

export async function reorderStorageSources(session: SessionIdentity, identifiers: string[]): Promise<StorageSource[]> {
  const current = await rpc.storageSourcesGet(session, {});
  const value = required(current.value, "Reorder Storage Sources");
  if (identifiers.length !== value.sources.length || new Set(identifiers).size !== identifiers.length)
    throw new Error("Reorder Storage Sources: Include Every Source Once");
  const ordered = identifiers.map((identifier, index) => ({
    ...required(
      value.sources.find((source) => source.identifier === identifier),
      "Reorder Storage Source",
    ),
    priority: BigInt(index),
  }));
  const reply = await rpc.storageSourcesSet(session, {
    value: { sources: ordered },
    expectedRevision: current.revision,
    requestId: crypto.randomUUID(),
  });
  return sources(reply);
}

export async function setStorageSourceEnabled(session: SessionIdentity, identifier: string, enabled: boolean): Promise<StorageSource[]> {
  const current = await rpc.storageSourcesGet(session, {});
  const value = required(current.value, "Set Storage Source");
  if (!value.sources.some((source) => source.identifier === identifier)) throw new Error("Set Storage Source: Select an Existing Source");
  const reply = await rpc.storageSourcesSet(session, {
    value: { sources: value.sources.map((source) => (source.identifier === identifier ? { ...source, enabled } : source)) },
    expectedRevision: current.revision,
    requestId: crypto.randomUUID(),
  });
  return sources(reply);
}

/* === Private Functions === */
function sources(reply: Pick<StorageSources, "value">): StorageSource[] {
  return required(reply.value, "Read Storage Sources").sources.map((source) => ({
    identifier: source.identifier,
    adapter: source.adapter,
    priority: safeNumber(source.priority, "storage source priority"),
    enabled: source.enabled,
  }));
}

/* === Entry Point === */
