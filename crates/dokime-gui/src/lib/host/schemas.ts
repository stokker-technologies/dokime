// Dokime — SPDX-License-Identifier: Apache-2.0
// Frozen Descriptor Sets Are Cached by Their Verified Digest.

/* === External Imports === */

import { create, createFileRegistry, toBinary } from "@bufbuild/protobuf";
import { FileDescriptorSetSchema, type FileDescriptorSet } from "@bufbuild/protobuf/wkt";
import type { ExecutionDetail } from "@host/dokime/client/v1/executions_pb.js";

/* === Local Imports === */

import * as rpc from "./client";
import { cutKey, required, type Wire } from "@/lib/rpc/canonical";
import type { SessionIdentity } from "@/lib/rpc/transport";
import { NATIVE_MAX_PAGES } from "@/lib/api/native-values";

/* === Classes === */

/** A Connection Owner Retains and Releases Its Frozen Descriptor Sets. */
export class FrozenSchemaCache {
  private readonly sets = new Map<string, Promise<ReturnType<typeof createFileRegistry>>>();

  get(session: SessionIdentity, detail: Wire<ExecutionDetail>): Promise<ReturnType<typeof createFileRegistry>> {
    const digest = required(detail.schemas, "Read Execution Schemas").digest;
    if (digest.length !== 32) throw new Error("Read Schemas: Supply a SHA-256 Digest");
    const key = [...digest].map((byte) => byte.toString(16).padStart(2, "0")).join("");
    const existing = this.sets.get(key);
    if (existing) return existing;
    const pending = this.load(session, detail, digest);
    this.sets.set(key, pending);
    void pending.catch(() => {
      if (this.sets.get(key) === pending) this.sets.delete(key);
    });
    return pending;
  }

  private async load(session: SessionIdentity, detail: Wire<ExecutionDetail>, digest: Uint8Array) {
    const name = required(detail.summary, "Read Schema Execution Identity").executionId;
    const types = create(FileDescriptorSetSchema);
    const tokens = new Set<string>();
    const records = new Map<string, string>();
    const streams = new Map<string, string>();
    let pageToken = "";
    let at = detail.resolvedAt;
    for (let index = 0; index < NATIVE_MAX_PAGES; index++) {
      if (!session.isCurrent()) throw new Error("Read Schemas: Owning Connection Changed");
      const page = await rpc.executionsSchemasGet(session, { name, at, page: { pageSize: 1, pageToken } });
      if (cutKey(page.digest) !== cutKey(digest) || !page.resolvedAt || (at && cutKey(at) !== cutKey(page.resolvedAt)))
        throw new Error("Read Schemas: Digest or Retained Cut Changed");
      for (const record of page.records) {
        if (!record.messageName || (records.has(record.recordType) && records.get(record.recordType) !== record.messageName))
          throw new Error("Read Schemas: Invalid Record Binding");
        records.set(record.recordType, record.messageName);
      }
      for (const stream of page.streams) {
        if (!stream.messageName || (streams.has(stream.stream) && streams.get(stream.stream) !== stream.messageName))
          throw new Error("Read Schemas: Invalid Stream Binding");
        streams.set(stream.stream, stream.messageName);
      }
      types.file.push(...required(page.types, "Read Schema Descriptor Page").file);
      if (!page.nextPageToken) {
        const actual = new Uint8Array(await crypto.subtle.digest("SHA-256", toBinary(FileDescriptorSetSchema, types)));
        if (cutKey(actual) !== cutKey(digest)) throw new Error("Read Schemas: Frozen Descriptor Digest Does Not Match");
        if (!session.isCurrent()) throw new Error("Read Schemas: Owning Connection Changed");
        const registry = frozenRegistry(types);
        for (const name of [...records.values(), ...streams.values()])
          if (!registry.getMessage(name)) throw new Error("Read Schemas: Bound Message Is Missing");
        return registry;
      }
      if (tokens.has(page.nextPageToken)) throw new Error("Read Schemas: Repeated Descriptor Page");
      tokens.add(page.nextPageToken);
      pageToken = page.nextPageToken;
      at = page.resolvedAt;
    }
    throw new Error("Read Schemas: Descriptor Page Budget Reached");
  }
}

/* === Constants === */

export function recordMessage(
  registry: ReturnType<typeof createFileRegistry>,
  identity: string,
  records?: readonly { recordType: string; messageName: string }[],
) {
  const name = records ? records.find((record) => record.recordType === identity)?.messageName : identity;
  return name ? registry.getMessage(name) : undefined;
}

const caches = new Map<string, FrozenSchemaCache>();

/* === Public Functions === */

export function executionSchemas(session: SessionIdentity): FrozenSchemaCache {
  let cache = caches.get(session.generation);
  if (!cache) {
    cache = new FrozenSchemaCache();
    caches.clear();
    caches.set(session.generation, cache);
  }
  return cache;
}

/** Descriptor Inventories Are Unordered; Resolve Imports After Verifying Their Original Bytes. */
function frozenRegistry(types: FileDescriptorSet): ReturnType<typeof createFileRegistry> {
  const pending = new Map(types.file.map((file) => [file.name, file]));
  if (pending.size !== types.file.length) throw new Error("Read Schemas: Duplicate Descriptor File");
  const ordered: FileDescriptorSet["file"] = [];
  const ready = new Set<string>();
  while (pending.size) {
    const before = pending.size;
    for (const [name, file] of pending) {
      if (!file.dependency.every((dependency) => ready.has(dependency))) continue;
      ordered.push(file);
      ready.add(name);
      pending.delete(name);
    }
    if (pending.size === before) throw new Error(`Read Schemas: Missing or Cyclic Descriptor Imports in ${[...pending.keys()].join(", ")}`);
  }
  return createFileRegistry(create(FileDescriptorSetSchema, { file: ordered }));
}
