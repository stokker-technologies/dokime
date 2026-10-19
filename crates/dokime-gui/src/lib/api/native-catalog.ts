/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Native catalog and parameter-default adapters for the original routed UI. */

/* === External Imports === */

/* === Local Imports === */

import type {
  Row8 as GeneratedFileParameterDefinition,
  Row7 as GeneratedParameterSchema,
  Row2 as GeneratedSuiteDefinition,
  Row3 as GeneratedSuiteIdentity,
  Row4 as GeneratedTemplateDefinition,
} from "@generated/dokime/v1/CatalogMessages/GetSuite/messages_pb";
import type { Row3 as GeneratedCursor } from "@generated/dokime/v1/CatalogMessages/ListSuites/messages_pb";
import type { Wire } from "@/lib/rpc/canonical";
import * as rpc from "@/lib/rpc/generated/client";
import type { SessionIdentity } from "@/lib/rpc/transport";
import type { FileDefinition, ParameterDefinition, ParameterListDefinition, SuiteTemplateEntry, TestSuiteEntry } from "@/lib/types";
import {
  jsonToNativeValue,
  mapBounded,
  metadataJson,
  NATIVE_CASE_LIMIT,
  NATIVE_MAX_PAGES,
  NATIVE_PAGE_SIZE,
  NativeApiLimitError,
  nativeValueObjectToJson,
  nativeValueToJson,
  safeNumber,
  uniqueStrings,
  valueTypeName,
} from "./native-values";

type SuiteDefinition = Wire<GeneratedSuiteDefinition>;
type SuiteIdentity = Wire<GeneratedSuiteIdentity>;
type TemplateDefinition = Wire<GeneratedTemplateDefinition>;
type ParameterSchema = Wire<GeneratedParameterSchema>;
type FileParameterDefinition = Wire<GeneratedFileParameterDefinition>;
type Cursor = Wire<GeneratedCursor>;

/* === Catalog === */

export async function fetchNativeSuites(session: SessionIdentity): Promise<TestSuiteEntry[]> {
  const identities = await listSuites(session);
  const definitions = await mapBounded(identities, (identity) => rpc.catalogGetSuite(session, { suiteId: identity.suiteId }));
  return definitions.map((definition, index) => mapSuite(definition, identities[index] ?? definition.identity));
}

export async function fetchNativeSuite(session: SessionIdentity, suiteId: string): Promise<TestSuiteEntry | null> {
  const identity = (await listSuites(session)).find((item) => item.suiteId === suiteId);
  if (!identity) return null;
  return mapSuite(await rpc.catalogGetSuite(session, { suiteId }), identity);
}

/* === Private catalog mapping === */

async function listSuites(session: SessionIdentity): Promise<SuiteIdentity[]> {
  const result: SuiteIdentity[] = [];
  let afterSuiteId = "";
  let cursor: Cursor | undefined;
  for (let page = 0; page < NATIVE_MAX_PAGES; page++) {
    const reply = await rpc.catalogListSuites(session, {
      limit: NATIVE_PAGE_SIZE,
      afterSuiteId,
      cursor,
    });
    result.push(...reply.suites);
    const next = reply.nextAfterSuiteId;
    if (!next) return result;
    if (next === afterSuiteId) throw new NativeApiLimitError("Catalog.ListSuites", `cursor did not advance at ${next}`);
    afterSuiteId = next;
    cursor = reply.cursor;
  }
  throw new NativeApiLimitError("Catalog.ListSuites", `more than ${NATIVE_MAX_PAGES} pages`);
}

function mapSuite(definition: SuiteDefinition, identity?: SuiteIdentity): TestSuiteEntry {
  const suiteIdentity = definition.identity ?? identity;
  if (!suiteIdentity?.suiteId) throw new Error("Native suite definition has no suite identity.");
  const name = suiteIdentity.metadata?.label || suiteIdentity.suiteId;
  return {
    suite_id: suiteIdentity.suiteId,
    name,
    description: suiteIdentity.metadata?.description ?? null,
    version: null,
    source: suiteIdentity.source,
    path: [name],
    tags: uniqueStrings(definition.tags),
    continue_on_failure: false,
    members: definition.templates.map((template) => mapSuiteTemplate(template, name)),
    metadata: metadataJson(suiteIdentity.metadata),
  };
}

function mapSuiteTemplate(template: TemplateDefinition, suiteName: string): SuiteTemplateEntry {
  if (!template.templateId) throw new Error("Native template has no stable template ID.");
  // The authored template id; never fall back to the internal UUID.
  const identifier = template.metadata?.identifier || null;
  const name = template.metadata?.label || identifier || template.templateId;
  return {
    template_id: template.templateId,
    identifier,
    name,
    description: template.metadata?.description ?? null,
    path: [suiteName, name],
    tags: uniqueStrings(template.tags),
    continue_on_failure: template.continueOnFailure,
    parameter_definitions: template.parameters.map(mapParameter),
    file_definitions: template.files.map(mapFile),
    steps: template.steps.map((step, index) => ({
      step_id: step.stepId,
      method_name: step.metadata?.identifier || step.stepId,
      label: step.metadata?.label ?? null,
      description: step.metadata?.description ?? null,
      order: index,
      timeout_s: null,
      required: true,
      metadata: metadataJson(step.metadata),
    })),
    metadata: metadataJson(template.metadata),
  };
}

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

function mapFile(file: FileParameterDefinition): FileDefinition {
  return {
    name: file.metadata?.identifier || "file",
    default: file.defaultValue ?? null,
    label: file.metadata?.label ?? null,
    description: file.metadata?.description ?? null,
    required: file.required,
    media_types: [...file.mediaTypes],
    metadata: {
      ...metadataJson(file.metadata),
      ...nativeValueObjectToJson(file.attributes),
    },
  };
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

export function valueObjectFromJson(values: Record<string, unknown>): ReturnType<typeof jsonToNativeValue> {
  return jsonToNativeValue(values);
}
