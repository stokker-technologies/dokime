/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Local Imports === */

import { formatListRaw, type ParseResult } from "@/lib/parameter-parser";
import type { ParameterDefinition, SuiteTemplateEntry, TestCycleSelection } from "@/lib/types";
import { numericValue } from "@/lib/utils";
import type { NotificationFailure } from "@/lib/notification-details";

/* === Types === */

export type TestCyclePlanSelectionDraft = TestCycleSelection & {
  key: string;
  label: string;
  suite_label: string;
  template_label: string;
  summary: string;
};

export type TestcaseFormDraft = {
  files: Record<string, string>;
  parameters: Record<string, string>;
};

export type ParameterRowState = {
  name: string;
  raw: string;
  parsed: ParseResult;
};

export type FileRowState = {
  failure?: NotificationFailure;
  name: string;
  path: string;
  uploading: boolean;
  error: string | null;
};

export type TestcasePayload = {
  values: Record<string, unknown[]>;
  files: Record<string, string | null>;
  errors: string[];
  key: string;
};

/* === Public Functions === */

export function createPlanSelection({
  suiteId,
  suiteName,
  template,
  values,
  files,
}: {
  suiteId: string;
  suiteName: string;
  template: SuiteTemplateEntry;
  values: Record<string, unknown[]>;
  files: Record<string, string | null>;
}): TestCyclePlanSelectionDraft {
  const templateLabel = template.name;
  const selectedTemplateId = templateSelectionId(template);
  const summary = selectionSummary(values, files);
  const key = JSON.stringify({
    suite_id: suiteId,
    template_id: selectedTemplateId,
    parameters: values,
    files,
  });
  return {
    suite_id: suiteId,
    template_id: selectedTemplateId,
    parameters: values,
    files,
    label: summary ? `${templateLabel} - ${summary}` : templateLabel,
    suite_label: suiteName,
    template_label: templateLabel,
    summary,
    key,
  };
}

export function templateSelectionId(template: SuiteTemplateEntry): string {
  const identifier = template.metadata.identifier;
  return typeof identifier === "string" && identifier.length > 0 ? identifier : template.template_id;
}

export function initialFileRows(template: SuiteTemplateEntry, draft?: TestcaseFormDraft): FileRowState[] {
  return template.file_definitions.map((definition) => ({
    name: definition.name,
    path: draft?.files[definition.name] ?? definition.default ?? "",
    uploading: false,
    error: null,
  }));
}

export function buildPayload(rows: ParameterRowState[], fileRows: FileRowState[]): TestcasePayload {
  const values: Record<string, unknown[]> = {};
  const files: Record<string, string | null> = {};
  const errors: string[] = [];
  for (const row of rows) {
    if (!row.parsed.ok) {
      errors.push(`${row.name}: ${row.parsed.error}`);
      continue;
    }
    if (row.parsed.values.length > 0) {
      values[row.name] = row.parsed.values;
    }
  }
  for (const row of fileRows) {
    if (row.error) {
      errors.push(`${row.name}: ${row.error}`);
      continue;
    }
    if (row.uploading) {
      errors.push(`${row.name}: upload in progress`);
      continue;
    }
    if (row.path.trim()) {
      files[row.name] = row.path.trim();
    }
  }
  return { values, files, errors, key: JSON.stringify({ values, files }) };
}

export function formatValueForInput(value: unknown): string {
  if (value === null || value === undefined) {
    return "";
  }
  if (typeof value === "boolean") {
    return value ? "true" : "false";
  }
  return String(value);
}

export function formatPreview(values: unknown[], valueType: string): string {
  return values.map((v) => formatPreviewValue(v, valueType)).join(", ");
}

/* === Private Functions === */

function initialParameterRaw(definition: ParameterDefinition): string {
  if (definition.metadata.kind === "parameter_group") {
    const first = definition.values[0];
    return parameterGroupVariantId(first) ?? "";
  }
  if (definition.list) {
    if ((definition.expansion?.kind ?? "single") === "single" && definition.values[0] == null) return "";
    return formatListRaw(
      (definition.expansion?.kind ?? "single") === "single" ? ((definition.values[0] as unknown[]) ?? []) : definition.values,
    );
  }
  if (definition.expansion?.kind === "zip") {
    return definition.values.length ? formatListRaw(definition.values) : "";
  }
  return definition.values.map((value) => formatValueForInput(value)).join(", ");
}

function formatPreviewValue(value: unknown, valueType: string): string {
  value = numericValue(value) ?? value;
  if (typeof value === "number") {
    if (valueType === "int" || valueType === "integer") {
      return String(Math.trunc(value));
    }
    return Number.isInteger(value) ? String(value) : value.toLocaleString(undefined, { maximumFractionDigits: 4 });
  }
  if (typeof value === "boolean") {
    return value ? "true" : "false";
  }
  const s = String(value);
  return s.length > 32 ? `${s.slice(0, 29)}...` : s;
}

function parameterGroupVariantId(value: unknown): string | null {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    return null;
  }
  const id = (value as Record<string, unknown>).id;
  return typeof id === "string" ? id : null;
}

function selectionSummary(values: Record<string, unknown[]>, files: Record<string, string | null>): string {
  const parts = [
    ...Object.entries(values).map(([name, entryValues]) => `${name}=${formatSelectionValues(entryValues)}`),
    ...Object.entries(files).map(([name, path]) => `${name}=${path || "default"}`),
  ];
  if (!parts.length) {
    return "defaults";
  }
  const summary = parts.slice(0, 3).join(", ");
  return parts.length > 3 ? `${summary}, ...` : summary;
}

function formatSelectionValues(values: unknown[]): string {
  if (values.length === 1) {
    return formatPreviewValue(values[0], "");
  }
  if (values.length <= 3) {
    return `[${values.map((value) => formatPreviewValue(value, "")).join(", ")}]`;
  }
  return `${values.length} values`;
}

/* === Parameter helpers === */

export function nonEmptyString(value: unknown): string | null {
  return typeof value === "string" && value.trim() ? value.trim() : null;
}

export function titleCaseIdentifier(value: string): string {
  const words = value
    .replace(/([a-z0-9])([A-Z])/g, "$1 $2")
    .split(/[._\-\s/]+/)
    .map((word) => word.trim())
    .filter(Boolean);
  if (!words.length) {
    return "Parameter";
  }
  return words.map(titleCaseWord).join(" ");
}

export function titleCaseWord(word: string): string {
  const upper = word.toUpperCase();
  if (["API", "BSSID", "ID", "IP", "SSID", "URL", "USB", "WIFI"].includes(upper)) {
    return upper === "WIFI" ? "WiFi" : upper;
  }
  return word.charAt(0).toUpperCase() + word.slice(1).toLowerCase();
}

export function parameterInputId(name: string): string {
  return `parameter-${name.replace(/[^A-Za-z0-9_-]/g, "-")}`;
}

export type ParameterChoice = {
  value: string;
  label: string;
};

export type WifiChannelChoice = ParameterChoice & {
  band: string | null;
  mode: string | null;
  stream: number | null;
};

export function parameterChoices(definition: ParameterDefinition): ParameterChoice[] {
  if (definition.metadata.kind === "parameter_group") {
    const variants = Array.isArray(definition.metadata.variants) ? definition.metadata.variants : [];
    return variants.flatMap((variant): ParameterChoice[] => {
      if (!variant || typeof variant !== "object" || Array.isArray(variant)) {
        return [];
      }
      const id = (variant as Record<string, unknown>).id;
      const label = (variant as Record<string, unknown>).label;
      if (typeof id !== "string") {
        return [];
      }
      return [{ value: id, label: typeof label === "string" ? label : id }];
    });
  }

  const choiceLabels = definition.metadata.choice_labels;
  const labels =
    choiceLabels && typeof choiceLabels === "object" && !Array.isArray(choiceLabels) ? (choiceLabels as Record<string, unknown>) : {};
  return (definition.choices ?? [])
    .filter((choice): choice is string | number | boolean => ["boolean", "number", "string"].includes(typeof choice))
    .map((choice) => {
      const value = String(choice);
      return {
        value,
        label: typeof labels[value] === "string" ? labels[value] : value,
      };
    });
}

export function wifiChannelChoices(definition: ParameterDefinition, fallbackChoices: ParameterChoice[]): WifiChannelChoice[] {
  const options = Array.isArray(definition.metadata.wifi_channel_options) ? definition.metadata.wifi_channel_options : [];
  const parsed = options.flatMap((option): WifiChannelChoice[] => {
    if (!option || typeof option !== "object" || Array.isArray(option)) {
      return [];
    }
    const record = option as Record<string, unknown>;
    if (typeof record.value !== "string" || typeof record.label !== "string") {
      return [];
    }
    return [
      {
        value: record.value,
        label: record.label,
        band: typeof record.band === "string" ? record.band : null,
        mode: typeof record.mode === "string" ? record.mode : null,
        stream: typeof record.stream === "number" ? record.stream : null,
      },
    ];
  });
  if (parsed.length) {
    return parsed;
  }
  return fallbackChoices.map((choice) => ({
    ...choice,
    band: null,
    mode: null,
    stream: null,
  }));
}

export function wifiSelectionSummary(choices: WifiChannelChoice[]): string {
  if (!choices.length) {
    return "Template default";
  }
  if (choices.length <= 2) {
    return choices.map((choice) => choice.label).join(", ");
  }
  const autoCount = choices.filter((choice) => choice.mode === "auto").length;
  const explicitCount = choices.length - autoCount;
  const parts = [];
  if (autoCount) {
    parts.push(`${autoCount} auto`);
  }
  if (explicitCount) {
    parts.push(`${explicitCount} stream${explicitCount === 1 ? "" : "s"}`);
  }
  return parts.join(", ");
}

/**
 * The registered default of a parameter in the raw editor form.
 *
 * The parameter editor seeds and compares against this single string: an
 * override differs from the default exactly when its raw text differs, which
 * is also what the overrides store persists. Comparing typed values instead
 * would have to reason about the native integer and number wrappers.
 */
export function registeredParameterRaw(definition: ParameterDefinition): string {
  return initialParameterRaw(definition);
}
