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

import type * as hostModel from "@generated/host/model";

import { nativeValueToJson } from "@dokime/plots/lib/host/values";

import type { Wire } from "@dokime/plots/lib/rpc/canonical";

import { traceabilitySnapshotFromItems } from "../traceability";

import type { TraceabilitySnapshot } from "@dokime/plots/lib/types";

import type { EvidenceScope } from "@generated/host/model";

import { NativeApiLimitError, timestampIso } from "@dokime/plots/lib/api/native-values";

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

export type NativeTraceabilitySnapshot = Wire<hostModel.TraceabilityGetResponse["snapshots"][number]>;

/* === Classes === */

/* === Public Functions === */

export function mapNativeTraceability(
  snapshots: NativeTraceabilitySnapshot[],
  requestedScope: Wire<EvidenceScope> | undefined,
  operation: string,
): TraceabilitySnapshot {
  const items = snapshots.map((snapshot) => {
    if (!snapshot?.traceabilityId) {
      throw new NativeApiLimitError(operation, "snapshot omitted its traceability id");
    }
    if (!snapshot.createdAt) {
      throw new NativeApiLimitError(operation, "snapshot omitted its creation timestamp");
    }
    const createdAt = timestampIso(snapshot.createdAt);
    if (!createdAt) {
      throw new NativeApiLimitError(operation, "snapshot contained an invalid creation timestamp");
    }
    if (!Array.isArray(snapshot.entries)) {
      throw new NativeApiLimitError(operation, "snapshot omitted its entry collection");
    }
    const entries = snapshot.entries.map((entry) => {
      if (!entry?.metadata) {
        throw new NativeApiLimitError(operation, "traceability entry omitted its metadata");
      }
      if (!entry.value) {
        throw new NativeApiLimitError(operation, "traceability entry omitted its value");
      }
      let value: unknown;
      try {
        value = nativeValueToJson(entry.value);
      } catch (error) {
        throw new NativeApiLimitError(operation, `traceability entry contained an invalid value: ${String(error)}`);
      }
      if (value === undefined) {
        throw new NativeApiLimitError(operation, "traceability entry contained an unknown value");
      }
      return {
        metadata: {
          identifier: entry.metadata.identifier,
          label: entry.metadata.label,
          description: entry.metadata.description,
        },
        value,
        source: entry.source,
        unit: entry.unit,
      };
    });
    return {
      traceability_id: snapshot.traceabilityId,
      scope: snapshot.scope,
      metadata: snapshot.metadata
        ? {
            identifier: snapshot.metadata.identifier,
            label: snapshot.metadata.label,
            description: snapshot.metadata.description,
          }
        : null,
      entries,
      created_at: createdAt,
    };
  });
  const result = traceabilitySnapshotFromItems(items, (value) => value ?? null);
  if (requestedScope) {
    result.scope = requestedScope ?? null;
  }
  return result;
}

/* === Private Functions === */

/* === Entry Point === */
