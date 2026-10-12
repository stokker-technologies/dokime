/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Local Imports === */

import type {
  JsonObject,
  RecordScope,
  TraceabilityGroup,
  TraceabilityItem,
  TraceabilitySnapshot,
} from "@/lib/types";

/* === Types === */

type TraceabilityMetadata = {
  identifier: string;
  label: string;
  description?: string | null;
};

type TraceabilityEntryInput = {
  metadata: TraceabilityMetadata;
  source?: string | null;
  unit?: string | null;
  value: unknown;
};

type TraceabilitySnapshotInput<TScope> = {
  traceability_id: string;
  scope?: TScope | null;
  /**
   * What this group is, when something said so. A template states its
   * traceability as labelled sections, and a section arrives with its label
   * — so it is rendered as the author wrote it rather than guessed at.
   */
  metadata?: TraceabilityMetadata | null;
  entries: TraceabilityEntryInput[];
  created_at: string;
};

/* === Public Functions === */

export function traceabilitySnapshotFromItems<TScope>(
  items: TraceabilitySnapshotInput<TScope>[],
  mapScope: (scope?: TScope | null) => RecordScope | null,
): TraceabilitySnapshot {
  const groups = items.flatMap((item) => traceabilityGroups(item, mapScope));
  const dedupedGroups = latestTraceabilityGroups(groups);

  return {
    groups: dedupedGroups,
    warnings: [],
    artifacts: [],
    updated_at: items.at(-1)?.created_at ?? new Date().toISOString(),
    scope: mapScope(items.at(-1)?.scope),
  };
}

/* === Private Functions === */

function traceabilityGroups<TScope>(
  item: TraceabilitySnapshotInput<TScope>,
  mapScope: (scope?: TScope | null) => RecordScope | null,
): TraceabilityGroup[] {
  const scope = mapScope(item.scope);
  const visibleEntries = item.entries.filter(
    (entry) => entry.source !== "traceability.environment",
  );
  if (!visibleEntries.length) {
    return [];
  }
  if (item.metadata) {
    return statedTraceabilityGroup(item, item.metadata, scope, visibleEntries);
  }

  return [...entriesByTitle(item, scope, visibleEntries).entries()].flatMap(([title, entries]) => {
    const items = cleanTraceabilityItems(
      title,
      entries.map((entry) => ({
        label: entry.metadata.label,
        value: entry.value,
        unit: null,
        level: "info" as const,
        details: {
          identifier: entry.metadata.identifier,
          ...(entry.source ? { source: entry.source } : {}),
        },
        scope,
      })),
    );
    if (!items.length) {
      return [];
    }
    const sources = traceabilityItemSources(items);

    return [
      {
        group_id: `${item.traceability_id}:${title}`,
        title,
        summary: "",
        level: "info" as const,
        items,
        details: sources.length ? { sources } : {},
        updated_at: item.created_at,
      },
    ];
  });
}

/**
 * One section a template stated, rendered as it was stated.
 *
 * No title is inferred, no row is rewritten, and nothing is hidden: the author
 * chose the label, the rows and their order, and a reader is shown that choice.
 */
function statedTraceabilityGroup<TScope>(
  item: TraceabilitySnapshotInput<TScope>,
  metadata: TraceabilityMetadata,
  scope: RecordScope | null,
  entries: TraceabilityEntryInput[],
): TraceabilityGroup[] {
  const items: TraceabilityItem[] = entries.map((entry) => ({
    label: entry.metadata.label,
    value: entry.value,
    unit: entry.unit ?? null,
    level: "info" as const,
    details: {
      identifier: entry.metadata.identifier,
      ...(entry.source ? { source: entry.source } : {}),
    },
    scope,
  }));
  const sources = traceabilityItemSources(items);

  return [
    {
      group_id: `${item.traceability_id}:${metadata.identifier}`,
      title: metadata.label,
      summary: "",
      level: "info" as const,
      items,
      details: { stated: true, ...(sources.length ? { sources } : {}) },
      updated_at: item.created_at,
    },
  ];
}

function entriesByTitle<TScope>(
  item: TraceabilitySnapshotInput<TScope>,
  scope: RecordScope | null,
  entries: TraceabilityEntryInput[],
): Map<string, TraceabilityEntryInput[]> {
  const groups = new Map<string, TraceabilityEntryInput[]>();
  for (const entry of entries) {
    const title = traceabilityGroupTitle(item, scope, [entry]);
    groups.set(title, [...(groups.get(title) ?? []), entry]);
  }
  return groups;
}

function latestTraceabilityGroups(groups: TraceabilityGroup[]): TraceabilityGroup[] {
  const mergedByTitle = new Map<string, TraceabilityGroup>();
  for (const group of groups) {
    // A stated section keys on its own identity: two templates may both call a
    // section "Device Under Test" and mean two different devices, and merging
    // them would put one attempt's serial number beside another's.
    const key = isStatedGroup(group) ? group.group_id : group.title;
    const existing = mergedByTitle.get(key);
    if (!existing) {
      mergedByTitle.set(key, group);
      continue;
    }

    const updatedAt =
      existing.updated_at.localeCompare(group.updated_at) <= 0
        ? group.updated_at
        : existing.updated_at;
    const sources = [
      ...new Set([
        ...traceabilityItemSources(existing.items),
        ...traceabilityItemSources(group.items),
      ]),
    ];
    const items = cleanTraceabilityItems(group.title, [...existing.items, ...group.items]);
    mergedByTitle.set(key, {
      ...existing,
      group_id:
        existing.updated_at.localeCompare(group.updated_at) <= 0
          ? group.group_id
          : existing.group_id,
      summary: "",
      items,
      details: sources.length ? { sources } : {},
      updated_at: updatedAt,
    });
  }

  return [...mergedByTitle.values()].filter((group) => group.items.length);
}

/// The groups the host derives on its own, keyed by the provenance their
/// rows are filed under. Mirrors `DERIVED_TRACEABILITY_GROUPS` in `dokime-app`:
/// a row of the host snapshot is filed beneath `traceability.host` — `.cpu`,
/// `.ram` and the rest — and all of them are the one group a reader is shown.
const DERIVED_TRACEABILITY_GROUPS: ReadonlyArray<{ source: string; label: string }> = [
  { source: "traceability.test_definition", label: "Test Definition" },
  { source: "traceability.test_execution", label: "Test Execution" },
  { source: "traceability.leases", label: "Leases" },
  { source: "traceability.host", label: "Host" },
];

function derivedTraceabilityGroup(source: string | null | undefined): string | undefined {
  if (!source) return undefined;
  return DERIVED_TRACEABILITY_GROUPS.find(
    (group) => source === group.source || source.startsWith(`${group.source}.`),
  )?.label;
}

function traceabilityGroupTitle<TScope>(
  item: TraceabilitySnapshotInput<TScope>,
  scope: RecordScope | null,
  entries = item.entries,
): string {
  // Host-owned groups are titled from their provenance rather than the
  // words in their rows. A row
  // whose label is rewritten stays in the group it was filed under. This
  // mirrors `derived_group_title` in the Rust reporter; the ladder below it is
  // the older one, which still names the one-per-resource groups.
  const derived = entries
    .map((entry) => derivedTraceabilityGroup(entry.source))
    .find((group) => group !== undefined);
  if (derived) return derived;

  const text = entries
    .flatMap((entry) => [entry.metadata.identifier, entry.metadata.label, entry.source ?? ""])
    .join(" ")
    .toLowerCase();

  if (text.includes("traceability.python_interfaces")) return "Python Interfaces";
  if (text.includes("traceability.head_unit")) return "Head Unit Traceability";
  if (text.includes("bluetooth")) return "Bluetooth Traceability";
  if (text.includes("traceability.wifi") || text.includes("wifi")) return "WiFi Traceability";
  if (text.includes("traceability.network") || text.includes("network"))
    return "Network Traceability";
  if (text.includes("host")) return "Host Traceability";
  if (text.includes("power") || text.includes("battery")) return "Power Traceability";
  if (text.includes("environment") || text.includes("weather") || text.includes("location"))
    return "Environment Traceability";
  if (text.includes("cycle") || text.includes("suite")) return "Cycle Traceability";
  if (text.includes("execution")) return "Execution Traceability";
  if (text.includes("instrument") || text.includes("device") || text.includes("dut"))
    return "Head Unit Traceability";
  if (scope?.test_execution_id) return "Execution Traceability";
  if (scope?.test_case_id) return "Case Traceability";
  if (scope?.test_cycle_id) return "Cycle Traceability";
  return "Runtime Traceability";
}

export function isStatedGroup(group: TraceabilityGroup): boolean {
  return group.details?.stated === true;
}

function cleanTraceabilityItems(title: string, items: TraceabilityItem[]): TraceabilityItem[] {
  const deduped = new Map<string, TraceabilityItem>();
  for (const item of expandTraceabilityItems(title, items)) {
    if (!hasTraceabilityValue(item.value) || isHiddenTraceabilityItem(title, item)) {
      continue;
    }
    const key = canonicalTraceabilityItemKey(title, item);
    const existing = deduped.get(key);
    deduped.set(key, existing ? preferredTraceabilityItem(title, existing, item) : item);
  }
  return [...deduped.values()];
}

function expandTraceabilityItems(title: string, items: TraceabilityItem[]): TraceabilityItem[] {
  if (!isHeadUnitGroup(title)) {
    return items;
  }

  const expanded = [...items];
  for (const item of items) {
    if (!isJsonObject(item.value)) {
      continue;
    }
    const identifier = traceabilityIdentifier(item);
    const reads = isJsonObject(item.value.reads) ? item.value.reads : item.value;
    const identity = isJsonObject(reads.identity) ? reads.identity : reads;
    const properties = [
      isJsonObject(reads.tool_properties) ? reads.tool_properties : null,
      isJsonObject(identity.properties) ? identity.properties : null,
      isJsonObject(item.value.tool_properties) ? item.value.tool_properties : null,
    ].find((candidate): candidate is JsonObject => Boolean(candidate));

    pushExpandedHeadUnitValue(
      expanded,
      item,
      `${identifier}.firmware_version`,
      "Firmware Version",
      identity.firmware_version ?? properties?.FirmwareVersion,
    );
    pushExpandedHeadUnitValue(
      expanded,
      item,
      `${identifier}.serial_number`,
      "Serial Number",
      identity.serial_number ?? properties?.SerialNumber,
    );
    pushExpandedHeadUnitValue(
      expanded,
      item,
      `${identifier}.instrument_serial_number`,
      "Instrument Serial Number",
      identity.instrument_serial_number,
    );
    pushExpandedHeadUnitValue(
      expanded,
      item,
      `${identifier}.production_number`,
      "Production Number",
      identity.production_number ?? properties?.ProductionNumber,
    );
    pushExpandedHeadUnitValue(
      expanded,
      item,
      `${identifier}.model_number`,
      "Model Number",
      identity.model_number,
    );
    pushExpandedHeadUnitValue(
      expanded,
      item,
      `${identifier}.tool_type`,
      "Tool Type",
      properties?.ToolType,
    );
  }
  return expanded;
}

function pushExpandedHeadUnitValue(
  items: TraceabilityItem[],
  source: TraceabilityItem,
  identifier: string,
  label: string,
  value: unknown,
) {
  if (!hasTraceabilityValue(value)) {
    return;
  }
  items.push({
    ...source,
    label,
    value,
    details: {
      ...source.details,
      identifier,
    },
  });
}

function preferredTraceabilityItem(
  title: string,
  current: TraceabilityItem,
  candidate: TraceabilityItem,
): TraceabilityItem {
  const currentScore = traceabilityItemScore(title, current);
  const candidateScore = traceabilityItemScore(title, candidate);
  return candidateScore > currentScore ? candidate : current;
}

function traceabilityItemScore(title: string, item: TraceabilityItem): number {
  const identifier = traceabilityIdentifier(item);
  const label = item.label.toLowerCase();
  let score = isJsonObject(item.value) ? 0 : 10;
  if (identifier.includes(".head_unit.")) score += 5;
  if (identifier.includes(".traceability.")) score += 3;
  if (identifier.includes(".device_info.")) score -= 4;
  if (isHeadUnitGroup(title) && (label.includes("firmware") || label.includes("serial"))) {
    score += 8;
  }
  return score;
}

function canonicalTraceabilityItemKey(title: string, item: TraceabilityItem): string {
  const identifier = traceabilityIdentifier(item);
  const label = normalizeTraceabilityKey(item.label);
  if (isHeadUnitGroup(title)) {
    if (identifier.includes("instrument_serial_number")) {
      return "head_unit.instrument_serial_number";
    }
    if (identifier.includes("serial_number") || label.includes("serial_number")) {
      return "head_unit.serial_number";
    }
    if (identifier.includes("firmware_version") || label.includes("firmware")) {
      return "head_unit.firmware_version";
    }
    if (identifier.includes("production_number")) {
      return "head_unit.production_number";
    }
    if (identifier.includes("model_number")) {
      return "head_unit.model_number";
    }
    if (identifier.includes("tool_type") || label === "tool_type") {
      return "head_unit.tool_type";
    }
  }
  return identifier
    .replace(/^resource\.[^.]+\.(traceability|head_unit|device_info|python_interface)\./u, "")
    .replace(/^traceability\.[^.]+\./u, "")
    .replace(/\s+/gu, "_")
    .toLowerCase();
}

function isHiddenTraceabilityItem(title: string, item: TraceabilityItem): boolean {
  const identifier = traceabilityIdentifier(item);
  if (!isHeadUnitGroup(title)) {
    return false;
  }
  if (identifier.endsWith(".captured_at")) {
    return true;
  }
  return (
    identifier.endsWith(".reads") ||
    identifier.endsWith(".tool_properties") ||
    identifier.endsWith(".properties") ||
    identifier.endsWith(".identity")
  );
}

function hasTraceabilityValue(value: unknown): boolean {
  if (value === null || value === undefined || value === "") {
    return false;
  }
  if (Array.isArray(value)) {
    return value.length > 0;
  }
  if (isJsonObject(value)) {
    return Object.values(value).some(hasTraceabilityValue);
  }
  return true;
}

function traceabilityItemSources(items: TraceabilityItem[]): string[] {
  return [
    ...new Set(
      items
        .map((item) => item.details.source)
        .filter((source): source is string => typeof source === "string" && Boolean(source)),
    ),
  ];
}

function traceabilityIdentifier(item: TraceabilityItem): string {
  return typeof item.details.identifier === "string"
    ? item.details.identifier.toLowerCase()
    : normalizeTraceabilityKey(item.label);
}

function normalizeTraceabilityKey(value: string): string {
  return value
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/gu, "_")
    .replace(/^_+|_+$/gu, "");
}

function isHeadUnitGroup(title: string): boolean {
  return title.toLowerCase().includes("head unit");
}

function isJsonObject(value: unknown): value is JsonObject {
  return Boolean(value) && typeof value === "object" && !Array.isArray(value);
}
