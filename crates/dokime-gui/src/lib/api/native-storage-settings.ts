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

type WirePolicyView = Wire<Awaited<ReturnType<typeof rpc.settingsReadStoragePolicy>>>;
type StorageSources = Wire<Awaited<ReturnType<typeof rpc.settingsListStorageSources>>>;

import type { SessionIdentity } from "@/lib/rpc/transport";
import { safeNumber } from "./native-values";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */
export type StoragePolicy = {
  compactionIntervalMs: number;
  localRetentionMs: number | null;
};
export type StoragePolicyView = {
  revision: bigint;
  configured: StoragePolicy;
  effective: StoragePolicy;
  restartRequired: boolean;
};

/* === Classes === */

/* === Public Functions === */

export async function fetchStorageSources(session: SessionIdentity): Promise<StorageSource[]> {
  return sources(await rpc.settingsListStorageSources(session, {}));
}

export async function reorderStorageSources(
  session: SessionIdentity,
  identifiers: string[],
): Promise<StorageSource[]> {
  return sources(await rpc.settingsReorderStorageSources(session, { identifiers }));
}

export async function setStorageSourceEnabled(
  session: SessionIdentity,
  identifier: string,
  enabled: boolean,
): Promise<StorageSource[]> {
  return sources(await rpc.settingsSetStorageSourceEnabled(session, { identifier, enabled }));
}

export async function readStoragePolicy(session: SessionIdentity): Promise<StoragePolicyView> {
  return policyView(await rpc.settingsReadStoragePolicy(session, {}));
}

export async function updateStoragePolicy(
  session: SessionIdentity,
  revision: bigint,
  policy: StoragePolicy,
): Promise<StoragePolicyView> {
  return policyView(
    await rpc.settingsUpdateStoragePolicy(session, {
      expectedRevision: revision,
      policy: {
        compactionIntervalMs: BigInt(policy.compactionIntervalMs),
        localRetentionMs:
          policy.localRetentionMs === null ? undefined : BigInt(policy.localRetentionMs),
      },
    }),
  );
}

/* === Private Functions === */
function policyView(view: WirePolicyView): StoragePolicyView {
  const policy = (value: WirePolicyView["configured"]): StoragePolicy => {
    if (!value) throw new Error("Backend omitted storage policy");
    return {
      compactionIntervalMs: safeNumber(value.compactionIntervalMs, "compaction interval"),
      localRetentionMs:
        value.localRetentionMs === undefined
          ? null
          : safeNumber(value.localRetentionMs, "retention"),
    };
  };
  return {
    revision: view.revision,
    configured: policy(view.configured),
    effective: policy(view.effective),
    restartRequired: view.restartRequired,
  };
}

function sources(reply: StorageSources): StorageSource[] {
  return reply.sources.map((source) => ({
    identifier: source.identifier,
    adapter: source.adapter,
    priority: safeNumber(source.priority, "storage source priority"),
    enabled: source.enabled,
  }));
}

/* === Entry Point === */
