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

import { saveNativeDocument } from "@/lib/api/native-delivery";
import { commands } from "@/lib/desktop/commands";
// Local Imports
import type { Wire } from "@/lib/rpc/canonical";
import * as rpc from "@/lib/rpc/generated/client";
import type { SessionIdentity } from "@/lib/rpc/transport";

/* === Re-Exports === */

/* === Constants === */

const MAX_ARCHIVE_BYTES = 512 * 1024 + 4096;

/* === Types === */

type Snapshot = Wire<Awaited<ReturnType<typeof rpc.secretsList>>>;
type Entry = Snapshot["entries"][number];
export type SecretMetadata = NonNullable<Entry["metadata"]>;
export type SecretEntry = Omit<Entry, "metadata"> & { metadata: SecretMetadata };
export type SecretCatalog = Omit<Snapshot, "entries"> & { entries: SecretEntry[] };
type WirePreview = Wire<Awaited<ReturnType<typeof rpc.secretsPreview>>>;
export type SecretPreview = Omit<WirePreview, "entries"> & {
  entries: (Omit<WirePreview["entries"][number], "metadata"> & { metadata: SecretMetadata })[];
};

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

export async function exportSecrets(session: SessionIdentity, names: string[], password: string, generation: bigint) {
  const archive = await rpc.secretsExport(session, { names, password, generation });
  if (archive.ciphertext.byteLength > MAX_ARCHIVE_BYTES) throw new Error("Encrypted archive exceeds its size limit");
  return saveNativeDocument(session, { kind: "secret_archive", filename: "dokime-secrets.age", bytes: archive.ciphertext });
}

export async function pickSecretArchive(session: SessionIdentity): Promise<Uint8Array | null> {
  const encoded = await commands.desktopPickSecretArchive(session.generation);
  if (!session.isCurrent()) throw new Error("Secret archive lost its owning connection");
  if (encoded === null) return null;
  if (encoded.length > Math.ceil(MAX_ARCHIVE_BYTES / 3) * 4) throw new Error("Encrypted archive exceeds its size limit");
  const bytes = Uint8Array.from(atob(encoded), (character) => character.charCodeAt(0));
  if (bytes.byteLength > MAX_ARCHIVE_BYTES) throw new Error("Encrypted archive exceeds its size limit");
  return bytes;
}

export async function previewSecrets(session: SessionIdentity, ciphertext: Uint8Array, password: string): Promise<SecretPreview> {
  const preview = await rpc.secretsPreview(session, { ciphertext, password });
  const entries = preview.entries.map((entry) => {
    if (!entry.metadata) throw new Error("Backend omitted imported secret metadata");
    return { ...entry, metadata: entry.metadata };
  });
  return { ...preview, entries };
}

export async function importSecrets(
  session: SessionIdentity,
  ciphertext: Uint8Array,
  password: string,
  preview: SecretPreview,
  replace: ReadonlySet<string>,
) {
  const choices = preview.entries.map((entry) => ({
    name: entry.metadata.name,
    action: {
      HarmosVariant: replace.has(entry.metadata.name) ? { case: "Replace" as const, value: {} } : { case: "Skip" as const, value: {} },
    },
  }));
  return catalog(await rpc.secretsImport(session, { ciphertext, password, choices, generation: preview.generation }));
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
