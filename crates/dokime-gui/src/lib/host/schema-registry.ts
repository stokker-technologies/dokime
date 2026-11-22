/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Shared Native Response Projection
 */

/* === Imports === */

import { create, createFileRegistry } from "@bufbuild/protobuf";

import { FileDescriptorSetSchema, type FileDescriptorSet } from "@bufbuild/protobuf/wkt";

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Public Functions === */

export function recordMessage(
  registry: ReturnType<typeof createFileRegistry>,
  identity: string,
  records?: readonly { recordType: string; messageName: string }[],
) {
  const name = records ? records.find((record) => record.recordType === identity)?.messageName : identity;
  return name ? registry.getMessage(name) : undefined;
}

/** Descriptor Inventories Are Unordered; Resolve Imports After Verifying Their Original Bytes. */
export function frozenRegistry(types: FileDescriptorSet): ReturnType<typeof createFileRegistry> {
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

/* === Private Functions === */

/* === Entry Point === */
