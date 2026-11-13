/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Canonical Host Values at the View Boundary.

/* === External Imports === */

import { ValueType, type Metadata, type Value } from "@host/dokime/v1/values_pb.js";

/* === Local Imports === */

import { NativeIntegerValue, NativeNumberValue, timestampIso } from "@/lib/api/native-values";
import type { Wire } from "@/lib/rpc/canonical";
import type { JsonObject } from "@/lib/types";

/* === Constants === */

const VALUE_DEPTH = 64;

/* === Public Functions === */

export function nativeValueToJson(value?: Wire<Value>): unknown {
  return readValue(value, 0);
}

function readValue(value: Wire<Value> | undefined, depth: number): unknown {
  if (!value) return undefined;
  if (depth > VALUE_DEPTH) throw new Error("Read Host Value: Nesting Exceeds the View Budget");
  const kind = value.kind;
  switch (kind.case) {
    case "null":
      return null;
    case "boolean":
    case "text":
      return kind.value;
    case "integer": {
      const number = Number(kind.value);
      return Number.isSafeInteger(number) ? number : new NativeIntegerValue(kind.value);
    }
    case "unsignedInteger":
      return new NativeIntegerValue(kind.value, true);
    case "number":
      return new NativeNumberValue(kind.value);
    case "data":
      return kind.value;
    case "list":
      return kind.value.values.map((item) => readValue(item, depth + 1));
    case "datetime": {
      const timestamp = timestampIso(kind.value);
      if (!timestamp) throw new Error("Read Host Value: Invalid Timestamp");
      return { $datetime: timestamp };
    }
    default:
      throw new Error("Read Host Value: Missing Kind");
  }
}

export function metadataJson(metadata?: Wire<Metadata>): JsonObject {
  if (!metadata) return {};
  return {
    identifier: metadata.identifier,
    label: metadata.label,
    description: metadata.description,
    tags: metadata.tags,
    attributes: Object.fromEntries(Object.entries(metadata.attributes).map(([key, value]) => [key, nativeValueToJson(value)])),
    created_at: timestampIso(metadata.createdAt),
    updated_at: timestampIso(metadata.updatedAt),
  };
}

export function valueTypeName(type: ValueType): string {
  switch (type) {
    case ValueType.BOOLEAN:
      return "boolean";
    case ValueType.INTEGER:
      return "integer";
    case ValueType.UNSIGNED_INTEGER:
      return "unsigned";
    case ValueType.NUMBER:
      return "number";
    case ValueType.TEXT:
      return "string";
    case ValueType.BYTES:
      return "bytes";
    case ValueType.LIST:
      return "array";
    case ValueType.DATETIME:
      return "datetime";
    default:
      return "unknown";
  }
}
