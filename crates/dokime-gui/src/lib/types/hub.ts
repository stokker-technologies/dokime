/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import type { JsonObject } from "../types";
import type { DocumentViewPayload } from "./document";
import type { ContractRegistrySnapshot, HubRecord, PromptRequest, Severity } from "./records";
import type { TelemetryStream } from "./telemetry";
import type { RecordScope } from "./view-contracts";

/* === Hub / runtime / artifact / traceability types ===
   Hub snapshot+live state, run/command status, diagnostics, and the
   artifact/evidence/traceability cluster. */

export type Metadata = {
  uuid?: string | null;
  created_at?: string | null;
  id?: string | null;
  identifier: string;
  label: string;
  description?: string | null;
  tags?: string[];
};

export type RunStatus =
  | "idle"
  | "running"
  | "waiting_for_prompt"
  | "finalizing"
  | "paused"
  | "stopping"
  | "completed"
  | "failed"
  | "stopped"
  | "disconnected";

export type ExecutionOutcome = "completed" | "failed" | "interrupted" | "stopped" | "unknown";

export type RunMetadata = {
  session_id: string;
  title: string;
  template_id?: string | null;
  template_slug: string;
  label?: string | null;
  metadata: JsonObject;
  status: RunStatus;
  started_at: string | null;
  finished_at?: string | null;
  updated_at: string;
  run_dir?: string | null;
  local_url?: string | null;
  lan_url?: string | null;
  rerun_command?: string[] | null;
};

export type { RecordScope } from "./view-contracts";

export type OperatorNote = {
  note_id: string;
  /** The authenticated identity that filed the note. */
  author?: string | null;
  text: string;
  timestamp: string;
  elapsed_since_start_label?: string | null;
  related_record_id?: string | null;
  related_sample_id?: string | null;
  related_artifact_id?: string | null;
  scope?: RecordScope | null;
};

export type HealthItem = {
  health_id: string;
  level: Severity;
  title: string;
  message?: string | null;
  details: JsonObject;
  related_artifact_id?: string | null;
  created_at: string;
  scope?: RecordScope | null;
};

export type ExecutionResultSummary = {
  outcome: ExecutionOutcome;
  headline?: string | null;
  warnings: string[];
  evidence: JsonObject[];
  updated_at: string;
  scope?: RecordScope | null;
};

export type TraceabilityItem = {
  label: string;
  value: unknown;
  unit?: string | null;
  level: Severity;
  details: JsonObject;
  scope?: RecordScope | null;
};

export type TraceabilityGroup = {
  group_id: string;
  title: string;
  summary?: string | null;
  level: Severity;
  items: TraceabilityItem[];
  details: JsonObject;
  updated_at: string;
};

export type TraceabilitySnapshot = {
  groups: TraceabilityGroup[];
  warnings: string[];
  artifacts: string[];
  updated_at: string;
  scope?: RecordScope | null;
};

export type ArtifactRecord = {
  artifact_id: string;
  label: string;
  path: string;
  href?: string | null;
  downloadHref?: string | null;
  media_type?: string | null;
  content_kind?: "text" | "binary" | "media" | "unknown" | null;
  stream_state?: "static" | "streaming" | "finalized" | "interrupted" | null;
  description?: string | null;
  size_bytes?: number | null;
  group_id?: string | null;
  group_label?: string | null;
  supersedes_artifact_id?: string | null;
  created_at: string;
  scope?: RecordScope | null;
};

export type DocumentStatistic =
  | "last"
  | "first"
  | "mean"
  | "median"
  | "std"
  | "rms"
  | "min"
  | "max"
  | "max_abs"
  | "sum"
  | "count"
  | "p95"
  | "p99";

export type ExecutionFact = {
  test_cycle_id: string;
  template_id: string;
  case_id: string;
  queue_item_id: string;
  test_execution_id?: string | null;
  ordinal: number;
  values: JsonObject;
};

export type MetricPresentation = "value" | "progress" | "badge";

export type MetricState = "ok" | "warning" | "critical" | "muted";

export type HubSnapshot = {
  run: RunMetadata;
  session_id: string;
  title: string;
  revision: number;
  records: HubRecord[];
  telemetry: Record<string, TelemetryStream>;
  pending_prompts: PromptRequest[];
  notes: OperatorNote[];
  health: HealthItem[];
  result_summary?: ExecutionResultSummary | null;
  traceability?: TraceabilitySnapshot | null;
  artifacts: ArtifactRecord[];
  installed_plugins?: InstalledPluginReadiness[];
  contracts?: ContractRegistrySnapshot | null;
  document?: DocumentViewPayload | null;
  updated_at: string;
};

export type InstalledPluginReadiness = {
  artifact_id: string;
  runtime: "native" | "wasm" | string;
  status: "pending" | "provisioning" | "ready" | "failed" | string;
  ready: boolean;
  cache_hit: boolean;
  domain_registrations: number;
  last_error?: string | null;
};

export type DokimeIdentity = {
  node: {
    node_id: string;
    hostname_label?: string | null;
    created_at: string;
    updated_at: string;
  };
  actor?: {
    actor_id: string;
    display_name: string;
  } | null;
};

export type {
  DiagnosticsResponse as DiagnosticsSnapshot,
  InstalledPluginErrorRecord,
} from "./view-contracts";

export type HubRevisionSet = {
  run: number;
  prompts: number;
  cycles: number;
  document: number;
  records: number;
  artifacts: number;
  executionFiles: number;
  notes: number;
  health: number;
  result_summary: number;
  traceability: number;
  contracts: number;
  telemetry: Record<string, number>;
};

export type HubLiveState = Omit<HubSnapshot, "contracts" | "document"> & {
  revisions: HubRevisionSet;
};

export type CommandStatus = "queued" | "running" | "applied" | "failed" | "rejected";

export type CommandReceipt = {
  command_id: string;
  kind: "runtime.command" | "prompt.response" | "note.add";
  target_id: string;
  status: CommandStatus;
  accepted_at: string;
  updated_at: string;
  dedupe_key?: string | null;
  message?: string | null;
  result?: Record<string, unknown> | null;
};
