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

import { saveNativeDocument } from "@/lib/api/native-document-delivery";
import { commands } from "@/lib/desktop/commands";
// Local Imports
import { required } from "@/lib/rpc/canonical";
import type { Wire } from "@/lib/rpc/canonical";
import * as rpc from "@/lib/host/client";
import { SecretImportAction } from "@host/dokime/client/v1/secrets_pb.js";
import type { Metadata } from "@host/dokime/v1/metadata_pb.js";
import type { SessionIdentity } from "@/lib/rpc/transport";

/* === Re-Exports === */

/* === Constants === */

const SECRETS_DOCUMENT_BYTES = 524_288;

const MAX_ARCHIVE_BYTES = 4 * Math.ceil(SECRETS_DOCUMENT_BYTES / 3);

/* === Types === */

type Snapshot = Wire<Awaited<ReturnType<typeof rpc.secretsList>>>;
type Entry = Snapshot["entries"][number];
export type SecretMetadata = Pick<Wire<Metadata>, "identifier" | "label"> & Partial<Omit<Wire<Metadata>, "identifier" | "label">>;
export type SecretEntry = Omit<Entry, "metadata"> & { metadata: SecretMetadata };
export type SecretSnapshot = Omit<Snapshot, "entries"> & {
  entries: SecretEntry[];
  hostId: string;
  hostLabel: string;
  cleanupPending?: boolean;
};
type WirePreview = Wire<NonNullable<Awaited<ReturnType<typeof rpc.secretsImport>>["preview"]>>;
export type SecretPreview = Omit<WirePreview, "entries"> & {
  entries: (Omit<WirePreview["entries"][number], "metadata"> & { metadata: SecretMetadata })[];
};

/* === Classes === */

/* === Public Functions === */

export async function readSecrets(session: SessionIdentity): Promise<SecretSnapshot> {
  const [snapshot, status] = await Promise.all([rpc.secretsList(session, {}), rpc.statusGet(session, {})]);
  const identity = required(status.status?.identity, "Read Secret Host Identity");
  return { ...catalog(snapshot), hostId: identity.hostId, hostLabel: identity.applicationName || identity.hostId };
}

export async function putSecret(session: SessionIdentity, metadata: SecretMetadata, value: string | undefined, generation: bigint) {
  const current = await rpc.secretsGet(session, { name: metadata.identifier });
  if ((await rpc.secretsList(session, {})).generation !== generation) throw new Error("Set Secret: Storage Changed; Refresh Before Saving");
  await rpc.secretsSet(session, {
    name: metadata.identifier,
    value: { metadata, value },
    expectedRevision: current.revision,
    requestId: crypto.randomUUID(),
  });
  return readSecrets(session);
}

export async function removeSecret(session: SessionIdentity, identifier: string, generation: bigint) {
  const current = await rpc.secretsGet(session, { name: identifier });
  if ((await rpc.secretsList(session, {})).generation !== generation)
    throw new Error("Delete Secret: Storage Changed; Refresh Before Deleting");
  const reply = await rpc.secretsDelete(session, {
    names: [identifier],
    expectedRevision: current.revision,
    requestId: crypto.randomUUID(),
  });
  return { ...(await readSecrets(session)), cleanupPending: reply.cleanupPending };
}

export async function exportSecrets(session: SessionIdentity, identifiers: string[], generation: bigint) {
  const archive = required((await rpc.secretsExport(session, { names: identifiers, generation })).archive, "Export Secrets");
  if (archive.data.byteLength > MAX_ARCHIVE_BYTES) throw new Error("Secret archive exceeds its size limit");
  return saveNativeDocument(session, { kind: "secret_archive", filename: "secrets.b64", bytes: archive.data });
}

export async function pickSecretArchive(session: SessionIdentity): Promise<Uint8Array | null> {
  const encoded = await commands.desktopPickSecretArchive(session.generation);
  if (!session.isCurrent()) throw new Error("Secret archive lost its owning connection");
  if (encoded === null) return null;
  if (encoded.length > Math.ceil(MAX_ARCHIVE_BYTES / 3) * 4) throw new Error("Secret archive exceeds its size limit");
  const bytes = Uint8Array.from(atob(encoded), (character) => character.charCodeAt(0));
  if (bytes.byteLength > MAX_ARCHIVE_BYTES) throw new Error("Secret archive exceeds its size limit");
  return bytes;
}

export async function previewSecrets(session: SessionIdentity, data: Uint8Array): Promise<SecretPreview> {
  const preview = required(
    (await rpc.secretsImport(session, { data, validateOnly: true, requestId: crypto.randomUUID() })).preview,
    "Preview Secrets",
  );
  const entries = preview.entries.map((entry) => {
    if (!entry.metadata) throw new Error("Backend omitted imported secret metadata");
    return { ...entry, metadata: entry.metadata };
  });
  return { ...preview, entries };
}

export async function importSecrets(session: SessionIdentity, data: Uint8Array, preview: SecretPreview, replace: ReadonlySet<string>) {
  const choices = preview.entries.map((entry) => ({
    identifier: entry.metadata.identifier,
    action: replace.has(entry.metadata.identifier) ? SecretImportAction.REPLACE : SecretImportAction.SKIP,
  }));
  await rpc.secretsImport(session, { data, choices, generation: preview.generation, requestId: crypto.randomUUID() });
  return readSecrets(session);
}

/* === Private Functions === */

function catalog(snapshot: Snapshot): Omit<SecretSnapshot, "hostId" | "hostLabel"> {
  const entries = snapshot.entries.map((entry) => {
    if (!entry.metadata) throw new Error("Backend omitted secret metadata");
    return { ...entry, metadata: entry.metadata };
  });
  return { ...snapshot, entries };
}

/* === Entry Point === */
