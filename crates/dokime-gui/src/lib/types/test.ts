/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import type { JsonObject } from "../types";
import type { CycleSummaryDefinition } from "./cycle-summary";

type ExecutionControls = {
  requestId: string;
  paused: boolean;
  stopRequested: boolean;
  canPause: boolean;
  canResume: boolean;
  canStop: boolean;
  mode: "automatic" | "interactive" | null;
};

/* === Test-domain types ===
   Authoring (Parameter/Step/Measurement/.../FileDefinition + TestCase/TestSuiteEntry),
   cycle lifecycle (Start/Delete/Preview/TestCycle) and execution (TestExecution/StepResult). */

/**
 * The element contract of a list parameter.
 *
 * Present only on a list. Its value is one array belonging to one case, so a
 * list is edited as a list of `element_type` elements and never expanded into
 * cases the way a parameter space is.
 */
export type ParameterListDefinition = {
  element_type: string;
  min_items?: number | null;
  max_items?: number | null;
};

export type ParameterDefinition = {
  parameter_id: string;
  name: string;
  values: unknown[];
  label?: string | null;
  description?: string | null;
  unit?: string | null;
  value_type?: string | null;
  choices: unknown[];
  expansion?: { kind: "single" } | { kind: "cartesian" } | { kind: "zip"; group: string };
  list?: ParameterListDefinition | null;
  min?: number | null;
  max?: number | null;
  metadata: JsonObject;
};

export type StepDefinition = {
  step_id: string;
  method_name: string;
  label?: string | null;
  description?: string | null;
  order: number;
  timeout_s?: number | null;
  required: boolean;
  metadata: JsonObject;
};

export type MeasurementDefinition = {
  measurement_id: string;
  method_name: string;
  stream: string;
  interval_s: number;
  field_name?: string | null;
  required: boolean;
  metadata: JsonObject;
};

export type ValidationCriterionDefinition = {
  criterion_id: string;
  method_name: string;
  label?: string | null;
  order: number;
  timeout_s?: number | null;
  required: boolean;
  metadata: JsonObject;
};

export type InitializeDefinition = {
  initialize_id: string;
  method_name: string;
  label?: string | null;
  order: number;
  timeout_s?: number | null;
  required: boolean;
  metadata: JsonObject;
};

export type FileDefinition = {
  name: string;
  default?: string | null;
  label?: string | null;
  description?: string | null;
  required: boolean;
  media_types?: string[];
  metadata: JsonObject;
};

export type FinalizeDefinition = {
  finalize_id: string;
  method_name: string;
  label?: string | null;
  order: number;
  timeout_s?: number | null;
  required: boolean;
  always: boolean;
  metadata: JsonObject;
};

export type TestTemplateMetadata = {
  template_id: string;
  name: string;
  description?: string | null;
  source_file?: string | null;
  parameters: ParameterDefinition[];
  files: FileDefinition[];
  steps: StepDefinition[];
  measurements: MeasurementDefinition[];
  criteria: ValidationCriterionDefinition[];
  initialize_hooks: InitializeDefinition[];
  finalize_hooks: FinalizeDefinition[];
  interrupt_hooks: InterruptDefinition[];
  cycle_summary?: CycleSummaryDefinition | null;
  metadata: JsonObject;
};

export type InterruptDefinition = {
  interrupt_id: string;
  method_name: string;
  label?: string | null;
  order: number;
  timeout_s?: number | null;
  required: boolean;
  metadata: JsonObject;
};

export type TestCase = {
  suite_id: string;
  case_id: string;
  template_id: string;
  parameters: JsonObject;
  generation_index: number;
  label: string;
  metadata: JsonObject;
};

export type SuiteTemplateEntry = {
  /** Internal registered UUID; never shown to people. */
  template_id: string;
  /** Readable hierarchical identifier (`mini.charger.steady.measurement`) the
   *  CLI and Python client accept; absent when the catalog does not carry one. */
  identifier?: string | null;
  name: string;
  description?: string | null;
  path: string[];
  tags: string[];
  /** The template's own policy, as registered; not the suite's. */
  continue_on_failure: boolean;
  parameter_definitions: ParameterDefinition[];
  file_definitions: FileDefinition[];
  steps: StepDefinition[];
  metadata: JsonObject;
};

export type TestSuiteEntry = {
  suite_id: string;
  name: string;
  description?: string | null;
  version?: string | null;
  source: string;
  product?: string | null;
  path: string[];
  tags: string[];
  continue_on_failure: boolean;
  members: SuiteTemplateEntry[];
  metadata: JsonObject;
};

export type TestCycleSelection = {
  suite_id: string;
  template_id: string;
  parameters: Record<string, unknown[]>;
  files?: Record<string, string | null>;
  label?: string | null;
  suite_label?: string | null;
  template_label?: string | null;
};

export type StartTestCycleRequest = {
  suite_id?: string | null;
  template_id?: string | null;
  parameters: Record<string, unknown[]>;
  files?: Record<string, string | null>;
  selections?: TestCycleSelection[];
  title?: string | null;
  continue_on_failure?: boolean;
  max_cases?: number | null;
  force?: boolean;
};

export type StartTestCycleResponse = {
  plan: TestCyclePlan;
};

export type DeleteTestCycleCasesRequest = {
  case_ids: string[];
};

export type PreviewTestCycleRequest = {
  suite_id?: string | null;
  template_id?: string | null;
  parameters: Record<string, unknown[]>;
  files?: Record<string, string | null>;
  selections?: TestCycleSelection[];
  max_cases?: number | null;
  force?: boolean;
};

export type PreviewTestCycleResponse = {
  case_count: number;
  cases: TestCase[];
  exceeds_default_cap: boolean;
  default_cap: number;
};

export type TestCycle = {
  test_cycle_id: string;
  title: string;
  suite_id?: string | null;
  suite_name?: string | null;
  status:
    | "planned"
    | "running"
    | "paused"
    | "stopping"
    | "completed"
    | "failed"
    | "stopped"
    | "interrupted";
  case_count: number;
  queue_count: number;
  current_test_execution_id?: string | null;
  current_queue_item_id?: string | null;
  continue_on_failure: boolean;
  started_at?: string | null;
  finished_at?: string | null;
  duration_s?: number | null;
  duration_delta_s?: number | null;
  metadata: JsonObject;
  generated_at: string;
};

/**
 * Which case mutations the runtime would admit right now.
 *
 * `unavailable_reasons` is the refused operations' own diagnostic copy,
 * deduplicated by the runtime; it is not a code vocabulary and never a promise,
 * because admission is re-checked on the mutation itself.
 */
export type CaseActions = {
  run: boolean;
  schedule_rerun: boolean;
  delete: boolean;
  unavailable_reasons: string[];
};

export type ExecutionQueueItem = {
  queue_item_id: string;
  test_cycle_id: string;
  ordinal: number;
  template_id: string;
  case_id: string;
  repeat_index: number;
  /* A case's own state, as `CaseSummary.status` projects its latest attempt.
     `unknown` is the historical projection: a captured cycle grouping keeps the
     plan without its attempt records and therefore states no case state at
     all. `planned` is a case with no attempt; `queued` one whose attempt is
     materialized and not yet dispatched. */
  status:
    | "unknown"
    | "planned"
    | "queued"
    | "running"
    | "waiting_for_prompt"
    | "finalizing"
    | "passed"
    | "failed"
    | "skipped"
    | "stopped"
    | "blocked"
    | "invalidated"
    | "interrupted";
  current_test_execution_id?: string | null;
  /** The status of `current_test_execution_id`, as the summary projects it. */
  current_test_execution_status?: TestExecution["status"] | null;
  attempt_count: number;
  /* A recorded outcome with a fresh undispatched attempt on top of it.
     Optional because a report frozen before the projection carried it has no
     such field, and a missing statement is not a scheduled rerun. */
  rerun_scheduled?: boolean;
  /** Absent where the projection reports none, which is every captured cycle. */
  actions?: CaseActions | null;
  blocked_reason?: string | null;
};

export type ExecutionPluginIdentity = {
  source: string;
  version?: string;
  implementation:
    | { kind: "bundle"; content_digest: string }
    | { kind: "embedded"; generation: string };
};

export type ExecutionInvalidation = {
  previous: ExecutionPluginIdentity;
  current: ExecutionPluginIdentity | null;
};

export type TestExecution = {
  controls?: ExecutionControls | null;
  test_execution_id: string;
  test_cycle_id: string;
  queue_item_id: string;
  case_id: string;
  template_id: string;
  attempt_index: number;
  invalidation?: ExecutionInvalidation | null;
  status: /* A planned attempt: the runtime reports it `SKIPPED` before sealing,
       which means "not run yet" rather than "passed over". */
    | "planned"
    | "created"
    | "running"
    | "waiting_for_prompt"
    | "finalizing"
    | "passed"
    | "failed"
    | "skipped"
    | "stopped"
    | "disconnected"
    | "invalidated"
    | "interrupted";
  started_at?: string | null;
  finished_at?: string | null;
  parameters?: JsonObject | null;
  metadata: JsonObject;
  steps: StepExecutionRecord[];
};

export type StepResult = {
  status: "pending" | "running" | "passed" | "failed" | "indeterminate" | "skipped" | "stopped";
  reason?: string | null;
  code?: number | null;
  details: JsonObject;
};

/**
 * The whole story behind one failure.
 *
 * The message is the line an operator reads; the chain, the location, the span
 * trace and the backtrace are what a maintainer needs and are shown behind an
 * expander. Already redacted by the host — no absolute home path reaches here.
 */
export type StepFailureDetail = {
  message: string;
  chain: string[];
  location: string;
  span_trace: string[];
  backtrace: string;
};

export type StepExecutionRecord = {
  record_type: "step" | "criterion" | "initialize" | "finalize" | "interrupt";
  test_execution_id: string;
  test_cycle_id: string;
  queue_item_id: string;
  case_id: string;
  template_id: string;
  step_id: string;
  method_name: string;
  order: number;
  status: "pending" | "running" | "passed" | "failed" | "indeterminate" | "skipped" | "stopped";
  result: StepResult;
  started_at: string;
  finished_at: string;
  duration_s: number;
  metadata: JsonObject;
  /** Why this step failed, for the one step that did. */
  detail?: StepFailureDetail | null;
};

export type TestCyclePlan = {
  test_cycle: TestCycle;
  templates: TestTemplateMetadata[];
  cases: TestCase[];
  queue: ExecutionQueueItem[];
  generated_at: string;
};

export type { CycleHistorySummary } from "./view-contracts";
