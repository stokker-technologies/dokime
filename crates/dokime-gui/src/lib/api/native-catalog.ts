// Dokime — SPDX-License-Identifier: Apache-2.0

export { fetchNativeSuites, fetchNativeSuite } from "../host/catalog";

import type { Row7 as GeneratedParameterSchema } from "@generated/dokime/v1/CatalogMessages/GetSuite/messages_pb";
import type { Wire } from "@/lib/rpc/canonical";
import type { ParameterDefinition, ParameterListDefinition } from "@/lib/types";
import {
  jsonToNativeValue,
  metadataJson,
  NATIVE_CASE_LIMIT,
  NativeApiLimitError,
  nativeValueToJson,
  safeNumber,
  valueTypeName,
} from "./native-values";

type ParameterSchema = Wire<GeneratedParameterSchema>;

export function mapParameter(parameter: ParameterSchema): ParameterDefinition {
  if (!parameter.parameterId) {
    throw new Error("Native parameter has no stable parameter ID.");
  }
  const choices = parameter.selection?.choices.map(nativeValueToJson) ?? [];
  const values = parameter.expression ? expressionValues(parameter.expression) : choices;
  const metadata = {
    ...metadataJson(parameter.metadata),
    ...(choices.length ? { choices } : {}),
    ...(parameter.runtimeWritable ? { runtime_writable: true } : {}),
  };
  return {
    parameter_id: parameter.parameterId,
    name: parameterIdentifier(parameter),
    values,
    label: parameter.metadata?.label ?? null,
    description: parameter.metadata?.description ?? null,
    unit: parameter.unit?.symbol ?? null,
    value_type: valueTypeName(parameter.valueType),
    choices,
    expansion:
      parameter.expansion?.kind?.HarmosVariant.case === "Zip"
        ? { kind: "zip", group: parameter.expansion.kind.HarmosVariant.value.field0 }
        : {
            kind: parameter.expansion?.kind?.HarmosVariant.case === "Cartesian" ? "cartesian" : "single",
          },
    list: mapParameterList(parameter.list),
    min: parameter.min ?? null,
    max: parameter.max ?? null,
    metadata,
  };
}

function mapParameterList(list: ParameterSchema["list"]): ParameterListDefinition | null {
  if (!list) {
    return null;
  }
  return {
    element_type: valueTypeName(list.elementType),
    min_items: list.minItems ?? null,
    max_items: list.maxItems ?? null,
  };
}

function parameterIdentifier(parameter: ParameterSchema): string {
  const identifier = parameter.metadata?.identifier;
  if (!identifier) {
    throw new Error(`Native parameter ${parameter.parameterId || "<unknown>"} has no metadata identifier.`);
  }
  return identifier;
}

function expressionValues(expression: NonNullable<ParameterSchema["expression"]>): unknown[] {
  return read(0, 0);
  function read(index: number, depth: number): unknown[] {
    if (depth > 64) throw new Error("Native expression exceeds 64 levels.");
    const kind = expression.nodes[index]?.kind?.HarmosVariant;
    if (!kind?.case) throw new Error("Native expression node has no kind.");
    function member(child: number): unknown[] {
      if (child <= index || child >= expression.nodes.length) throw new Error("Invalid expression member index.");
      return read(child, depth + 1);
    }
    switch (kind.case) {
      case "Literal":
        return [nativeValueToJson(kind.value.field0)];
      case "Array":
        return kind.value.field0?.values.flatMap(member) ?? [];
      case "Range": {
        const { start, stop, step } =
          kind.value.field0 ??
          (() => {
            throw new Error("Native expression payload is missing.");
          })();
        if (step === 0n || (step > 0n && start > stop) || (step < 0n && start < stop)) return [];
        const values: unknown[] = [];
        for (let current = start; step > 0n ? current <= stop : current >= stop; current += step) {
          if (values.length >= NATIVE_CASE_LIMIT) throw new NativeApiLimitError("Catalog.ParameterExpression", "more than 1000 values");
          values.push(nativeValueToJson(jsonToNativeValue(current)));
        }
        return values;
      }
      case "Arange": {
        const { start, stop, step } =
          kind.value.field0 ??
          (() => {
            throw new Error("Native expression payload is missing.");
          })();
        if (step === 0 || (step > 0 && start >= stop) || (step < 0 && start <= stop)) return [];
        const values: unknown[] = [];
        for (let current = start; step > 0 ? current < stop : current > stop; current += step) {
          if (values.length >= NATIVE_CASE_LIMIT) throw new NativeApiLimitError("Catalog.ParameterExpression", "more than 1000 values");
          values.push(Number(current.toFixed(12)));
        }
        return values;
      }
      case "Linspace": {
        const {
          start,
          stop,
          count: rawCount,
        } = kind.value.field0 ??
        (() => {
          throw new Error("Native expression payload is missing.");
        })();
        const count = safeNumber(rawCount, "linspace count");
        if (count < 0 || count > NATIVE_CASE_LIMIT) throw new NativeApiLimitError("Catalog.ParameterExpression", `linspace count ${count}`);
        if (count === 0) return [];
        if (count === 1) return [start];
        const step = (stop - start) / (count - 1);
        return Array.from({ length: count }, (_, index) => Number((start + step * index).toFixed(12)));
      }
      default:
        // References and dynamic expressions require a runtime context and are
        // intentionally left unevaluated in a catalog-only read.
        return [];
    }
  }
}
