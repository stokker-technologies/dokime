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

import type * as hostModel from "@host/model";

import { fallbackRecordDisplay } from "@dokime/plots/lib/record-display";

import type { Wire } from "@dokime/plots/lib/rpc/canonical";

import type { HubRecord, JsonObject } from "@dokime/plots/lib/types";

import { ScalarType, fromBinary, type createFileRegistry } from "@bufbuild/protobuf";

import { isReflectList, isReflectMap, isReflectMessage, reflect, type ReflectMessage } from "@bufbuild/protobuf/reflect";

import { FeatureSet_FieldPresence } from "@bufbuild/protobuf/wkt";

import { LogsAppendRequestSchema, Severity } from "@host/model";

import { recordMessage } from "../host/schema-registry";

import { NativeIntegerValue, NativeNumberValue, safeNumber, timestampIso } from "@dokime/plots/lib/api/native-values";

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

export type RecordPage = Wire<hostModel.RecordsListResponse>;

export type CapturedRecord = RecordPage["records"][number];

/* === Classes === */

/* === Public Functions === */

export function mapNativeRecord(record: CapturedRecord, details: JsonObject): HubRecord {
  const timestamp = timestampIso(record.timestamp);
  if (!timestamp) throw new Error(`Native record ${record.recordId} has no valid timestamp.`);
  const level =
    record.severity === Severity.FAILURE
      ? "error"
      : record.severity === Severity.WARNING
        ? "warning"
        : record.severity === Severity.DEBUG
          ? "debug"
          : "info";
  if (record.error) {
    details.error = {
      message: record.error.message,
      code: record.error.code ?? null,
      source: record.error.source ?? null,
      cause_chain: record.error.causeChain.map((cause) => ({
        message: cause.message,
        code: cause.code ?? null,
        source: cause.source ?? null,
      })),
      stacktrace_attachment: record.error.stacktraceAttachment ?? null,
      trace_id: record.error.traceId ?? null,
      correlation_id: record.error.correlationId ?? null,
    };
  }
  return {
    record_id: record.recordId,
    record_type: record.recordType,
    template_id: record.scope?.templateId ?? null,
    source: record.source,
    timestamp,
    session_id: record.scope?.sessionId ?? null,
    sequence: safeNumber(record.sequence, "Evidence.CapturedRecord sequence"),
    level,
    display: fallbackRecordDisplay(level),
    message: record.message ?? record.error?.message ?? null,
    details,
    related_sample_id: record.relatedSampleId ?? null,
    related_stream: record.relatedStreamId ?? null,
    related_artifact_id: record.relatedArtifactId ?? null,
    scope: record.scope ?? null,
  };
}

export function recordFields(message: ReflectMessage, depth: number): JsonObject {
  if (depth > 64) throw new Error("Read Record: Payload Nesting Exceeds the View Budget");
  const result: JsonObject = {};
  for (const field of message.fields) {
    if (field.presence !== FeatureSet_FieldPresence.IMPLICIT && !message.isSet(field)) continue;
    result[field.name] = recordValue(message.get(field), field.scalar, depth + 1);
  }
  return result;
}

/** Decode Immutable Evidence With Its Verified Frozen Registry. */
export function decodeFrozenRecords(
  records: CapturedRecord[],
  registry?: ReturnType<typeof createFileRegistry>,
  recordsBindings?: readonly { recordType: string; messageName: string }[],
): HubRecord[] {
  return records.map((record) => {
    if (record.recordType === "dokime.log") {
      const log = fromBinary(LogsAppendRequestSchema, record.payload);
      return mapNativeRecord(record, { message: log.message, severity: log.severity });
    }
    const message = registry ? recordMessage(registry, record.recordType, recordsBindings) : undefined;
    if (!message) throw new Error(`Read Record ${record.recordType}: Frozen Contract Is Missing`);
    return mapNativeRecord(record, recordFields(reflect(message, fromBinary(message, record.payload)), 0));
  });
}

/* === Private Functions === */

function recordValue(value: unknown, scalar: ScalarType | undefined, depth: number): unknown {
  if (depth > 64) throw new Error("Read Record: Payload Nesting Exceeds the View Budget");
  if (isReflectMessage(value)) {
    if (value.desc.typeName === "google.protobuf.Timestamp") {
      const fields = recordFields(value, depth);
      const seconds = fields.seconds instanceof NativeIntegerValue ? BigInt(fields.seconds.decimal) : BigInt(String(fields.seconds ?? 0));
      const timestamp = timestampIso({ seconds, nanos: Number(fields.nanos ?? 0) });
      if (!timestamp) throw new Error("Read Record: Invalid Payload Timestamp");
      return { $datetime: timestamp };
    }
    return recordFields(value, depth);
  }
  if (isReflectList(value)) return Array.from(value, (item) => recordValue(item, value.field().scalar, depth + 1));
  if (isReflectMap(value))
    return Object.fromEntries(Array.from(value, ([key, item]) => [String(key), recordValue(item, value.field().scalar, depth + 1)]));
  if (typeof value === "bigint") {
    const unsigned = scalar === ScalarType.UINT64 || scalar === ScalarType.FIXED64;
    const number = Number(value);
    return unsigned || !Number.isSafeInteger(number) ? new NativeIntegerValue(value, unsigned) : number;
  }
  if (typeof value === "number" && (scalar === ScalarType.DOUBLE || scalar === ScalarType.FLOAT)) return new NativeNumberValue(value);
  return value;
}

/* === Entry Point === */
