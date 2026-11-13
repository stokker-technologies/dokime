/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Host Suite and Template Presentation.

/* === External Imports === */

import type { Parameter, ParameterExpression } from "@host/dokime/v1/parameters_pb.js";
import type { Suite, TestCaseTemplate } from "@host/dokime/v1/suites_pb.js";
import { required } from "@/lib/rpc/canonical";
import { ValueType } from "@host/dokime/v1/scalar_pb.js";

/* === Local Imports === */

import { NATIVE_CASE_LIMIT, NativeApiLimitError, safeNumber, uniqueStrings } from "@/lib/api/native-values";
import type { Wire } from "@/lib/rpc/canonical";
import type { SessionIdentity } from "@/lib/rpc/transport";
import type { ParameterDefinition, SuiteTemplateEntry, TestSuiteEntry } from "@/lib/types";
import * as client from "./client";
import { metadataJson, nativeValueToJson, valueTypeName } from "./values";

/* === Public Functions === */

export async function fetchNativeSuites(session: SessionIdentity): Promise<TestSuiteEntry[]> {
  return (await listSuites(session)).map(mapSuite);
}

export async function fetchNativeSuite(session: SessionIdentity, suiteId: string): Promise<TestSuiteEntry | null> {
  return mapSuite(required((await client.suitesGet(session, { name: suiteId })).value, "Get Suite"));
}

async function listSuites(session: SessionIdentity): Promise<Wire<Suite>[]> {
  return (await client.suitesList(session, {})).values;
}

export function mapSuite(definition: Wire<Suite>): TestSuiteEntry {
  const identifier = required(definition.metadata?.identifier, "Read Suite Identity");
  const name = definition.metadata?.label || identifier;
  return {
    suite_id: identifier,
    name,
    description: definition.metadata?.description || null,
    version: null,
    source: "",
    path: [name],
    tags: uniqueStrings(definition.tags),
    members: definition.templates.map((template) => mapTemplate(template, name)),
    metadata: metadataJson(definition.metadata),
  };
}

export function mapParameter(parameter: Wire<Parameter>): ParameterDefinition {
  const identifier = parameter.metadata?.identifier;
  if (!identifier) throw new Error("Read Parameter: Missing Definition Identity");
  const choices = parameter.choices.map((value) => nativeValueToJson(value));
  const expansion = parameter.expansion?.kind;
  return {
    parameter_id: identifier,
    name: identifier,
    values: parameter.expression ? expressionValues(parameter.expression) : choices,
    label: parameter.metadata?.label || null,
    description: parameter.metadata?.description || null,
    unit: parameter.unit || null,
    value_type: valueTypeName(parameter.type),
    choices,
    expansion:
      expansion?.case === "zip"
        ? { kind: "zip", group: expansion.value }
        : { kind: expansion?.case === "cartesian" ? "cartesian" : "single" },
    list:
      parameter.type === ValueType.LIST
        ? { element_type: valueTypeName(parameter.elementType), min_items: parameter.minItems, max_items: parameter.maxItems }
        : null,
    min: parameter.min ?? null,
    max: parameter.max ?? null,
    metadata: {
      ...metadataJson(parameter.metadata),
      ...(parameter.runtimeWritable ? { runtime_writable: true } : {}),
      ...(choices.length ? { choices } : {}),
    },
  };
}

/* === Private Functions === */

function mapTemplate(template: Wire<TestCaseTemplate>, suite: string): SuiteTemplateEntry {
  const identifier = template.metadata?.identifier;
  if (!identifier) throw new Error("Read Template: Missing Definition Identity");
  const name = template.metadata?.label || identifier;
  return {
    template_id: identifier,
    identifier,
    name,
    description: template.metadata?.description || null,
    path: [suite, name],
    tags: uniqueStrings(template.tags),
    parameter_definitions: template.parameters.map(mapParameter),
    file_definitions: template.files.map((file) => {
      const identifier = file.metadata?.identifier;
      if (!identifier) throw new Error("Read File Parameter: Missing Definition Identity");
      return {
        name: identifier,
        default: file.defaultValue ?? null,
        label: file.metadata?.label || null,
        description: file.metadata?.description || null,
        required: file.required,
        media_types: file.mediaTypes,
        metadata: metadataJson(file.metadata),
      };
    }),
    // Steps Belong to the Materialized Case, Not the Template Definition.
    steps: [],
    metadata: metadataJson(template.metadata),
  };
}

function expressionValues(expression: Wire<ParameterExpression>, depth = 0): unknown[] {
  if (depth > 64) throw new Error("Read Parameter Expression: Nesting Exceeds the View Budget");
  const kind = expression.kind;
  switch (kind.case) {
    case "literal":
      return [nativeValueToJson(kind.value)];
    case "array": {
      const result = kind.value.values.flatMap((value) => expressionValues(value, depth + 1));
      if (result.length > NATIVE_CASE_LIMIT) throw new NativeApiLimitError("Read Parameter Expression", "Case Budget Reached");
      return result;
    }
    case "range": {
      const { start, stop, step } = kind.value;
      if (step === 0n) throw new Error("Read Parameter Range: Step Must Be Nonzero");
      const values: unknown[] = [];
      for (let value = start; step > 0n ? value < stop : value > stop; value += step) {
        if (values.length >= NATIVE_CASE_LIMIT) throw new NativeApiLimitError("Read Parameter Range", "Case Budget Reached");
        const number = Number(value);
        values.push(Number.isSafeInteger(number) ? number : nativeValueToJson({ kind: { case: "integer", value } }));
      }
      return values;
    }
    case "arange": {
      const { start, stop, step } = kind.value;
      if (![start, stop, step].every(Number.isFinite) || step === 0)
        throw new Error("Read Parameter Range: Use Finite Bounds and a Nonzero Step");
      const values: number[] = [];
      for (let index = 0; ; index++) {
        const value = start + index * step;
        if (!(step > 0 ? value < stop : value > stop)) return values;
        if (values.length >= NATIVE_CASE_LIMIT) throw new NativeApiLimitError("Read Parameter Range", "Case Budget Reached");
        values.push(value);
      }
    }
    case "linspace": {
      const { start, stop, count: rawCount } = kind.value;
      const count = safeNumber(rawCount, "Parameter Linspace Count");
      if (count > NATIVE_CASE_LIMIT) throw new NativeApiLimitError("Read Parameter Linspace", "Case Budget Reached");
      if (count === 0) return [];
      if (count === 1) return [start];
      return Array.from({ length: count }, (_, index) => (index === count - 1 ? stop : start + (index * (stop - start)) / (count - 1)));
    }
    default:
      return [];
  }
}
