/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Shared Secret Catalog Operations For The Connected Host.
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
import type { Wire } from "@/lib/rpc/canonical";
import * as rpc from "@/lib/rpc/generated/client";
import type { SessionIdentity } from "@/lib/rpc/transport";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

type Snapshot = Wire<Awaited<ReturnType<typeof rpc.secretsList>>>;
type Entry = Snapshot["entries"][number];
export type SecretMetadata = NonNullable<Entry["metadata"]>;
export type SecretEntry = Omit<Entry, "metadata"> & { metadata: SecretMetadata };
export type SecretCatalog = Omit<Snapshot, "entries"> & { entries: SecretEntry[] };

/* === Classes === */

/* === Public Functions === */

export async function readSecrets(session: SessionIdentity): Promise<SecretCatalog> {
  return catalog(await rpc.secretsList(session, {}));
}

export async function putSecret(session: SessionIdentity, metadata: SecretMetadata, value: string | undefined, generation: bigint) {
  return catalog(await rpc.secretsPut(session, { metadata, value, generation }));
}

export async function removeSecret(session: SessionIdentity, name: string, generation: bigint) {
  return catalog(await rpc.secretsRemove(session, { names: [name], generation }));
}

export async function adoptSecret(session: SessionIdentity, metadata: SecretMetadata, legacyId: string, generation: bigint) {
  return catalog(await rpc.secretsAdopt(session, { metadata, legacyId, generation }));
}

/* === Private Functions === */

function catalog(snapshot: Snapshot): SecretCatalog {
  const entries = snapshot.entries.map((entry) => {
    if (!entry.metadata) throw new Error("Backend omitted secret metadata");
    return { ...entry, metadata: entry.metadata };
  });
  return { ...snapshot, entries };
}

/* === Entry Point === */
