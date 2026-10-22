/**
 * GUI projection contracts used by native adapters and views.
 *
 * These preserve the existing UI shapes while native protobuf adapters map
 * native responses into them. They are GUI-owned contracts, not generated
 * server API types.
 */

import type { Row13 as GeneratedNativeExecutionCheckpoint } from "@generated/dokime/v1/HistoryMessages/BrowseExecutions/messages_pb";
import type { Wire } from "@/lib/rpc/canonical";
import type { ValueFormat } from "./format";

type NativeExecutionCheckpoint = Wire<GeneratedNativeExecutionCheckpoint>;
type NativeStorageCursor = NonNullable<NativeExecutionCheckpoint["position"]>;
type NativeStoreHistory = NonNullable<NativeStorageCursor["history"]>;

export type ActorId = string;

export type ArtifactId = string;

export type PluginRuntimeSecretBindings = {
  bindings?: { [key in string]: RedactedSecretBinding };
  missing_required?: string[];
};

export type Boolean = {
  id: string;
  title: string;
  description: string | null;
  default: boolean;
};

export type CaseId = string;

export type Choice = {
  value: string;
  title: string;
};

export type SecretBackendKind = "os_keyring" | "memory" | "local_file";

export type CriterionId = string;

export type CycleHistorySummary = {
  test_cycle_id: TestCycleId;
  session_id: string | null;
  title: string;
  phase: "active" | "sealing" | "sealed" | null;
  lifecycle: string;
  result_status: string | null;
  case_count: number;
  execution_count: number;
  queued_count: number;
  running_count: number;
  passed_count: number;
  failed_count: number;
  blocked_count: number;
  created_at: string;
  started_at: string | null;
  finished_at: string | null;
  updated_at: string;
  duration_s: number | null;
};

export type DebugLevel = "debug" | "info" | "warning" | "error";

export type DiagnosticCheckStatus = "ok" | "warning" | "failed";

export type DiagnosticFileFormat = "json" | "jsonl" | "zip";

export type DiagnosticPathConvention = {
  key: DiagnosticPathKey;
  scope: DiagnosticPathScope;
  format: DiagnosticFileFormat;
  relative_path: string;
  absolute_path: string | null;
};

export type DiagnosticPathKey =
  | "hub_log"
  | "startup_doctor"
  | "diagnostics_snapshot"
  | "collect_bundle"
  | "cycle_log_pattern"
  | "execution_log_pattern";

export type DiagnosticPathScope = "root" | "cycle_pattern" | "execution_pattern";

export type DiagnosticsCounts = {
  test_suites: number;
  cycles: number;
  installed_plugin_errors: number;
  /**
   *  Content-bearing runtime collections whose growth must be visible before
   *  retention and eviction policy can be changed safely.
   */
  state_cycles: number;
  state_executions_total: number;
  state_record_count: number;
  telemetry_cursor_entries: number;
};

export type DiagnosticsResponse = {
  server: ServerDiagnostics;
  runtime: DiagnosticsSnapshot;
};

export type DiagnosticsSnapshot = {
  captured_at: Timestamp;
  session_id: SessionId;
  identity: DokimeIdentity;
  revision: number;
  shutting_down: boolean;
  storage_root: string;
  hub_metadata: Metadata;
  startup_doctor: StartupDoctorSnapshot;
  path_conventions: DiagnosticPathConvention[];
  counts: DiagnosticsCounts;
  installed_plugin_errors: InstalledPluginErrorRecord[];
};

export type DokimeActorIdentity = {
  actor_id: ActorId;
  display_name: string;
};

export type DokimeIdentity = {
  node: DokimeNodeIdentity;
  actor: DokimeActorIdentity | null;
};

export type DokimeNodeIdentity = {
  node_id: NodeId;
  hostname_label: string | null;
  created_at: Timestamp;
  updated_at: Timestamp;
};

export type ErrorCause = {
  message: string;
  code: string | null;
  source: string | null;
};

export type ExecutionAvailability = "live" | "stored" | "interrupted" | "uncheckpointed";

export type ExecutionBrowseEntry = {
  metadata: ExecutionMetadata;
  checkpoint: ExecutionCheckpoint | null;
  availability: ExecutionAvailability;
  /** The execution, or its whole cycle, is archived. */
  archived: boolean;
};

/** One cycle's totals over the whole browse selection, as the backend folded them. */
export type ExecutionBrowseCycle = {
  cycle_id: TestCycleId;
  /** The cycle's own label; null once its header is no longer readable. */
  label: string | null;
  phase: "active" | "sealing" | "sealed" | null;
  lifecycle: "running" | "interrupted" | "pending" | "uncheckpointed" | "completed";
  result_status: "pending" | "passed" | "failed" | "blocked" | "cancelled" | null;
  case_count: number;
  execution_count: number;
  /** Planned executions that never started; every other count excludes them. */
  unrun_count: number;
  queued_count: number;
  running_count: number;
  passed_count: number;
  failed_count: number;
  blocked_count: number;
  started_at: string | null;
  finished_at: string | null;
};

export type ExecutionBrowseIssue = {
  source: string | null;
  dataset: string | null;
  message: string;
};

export type ExecutionBrowsePage = {
  entries: ExecutionBrowseEntry[];
  next: TestExecutionId | null;
  discovery_complete: boolean;
  conflicts: TestExecutionId[];
  omitted_conflicts: number;
  issues: ExecutionBrowseIssue[];
  omitted_issues: number;
  /** Totals of every cycle the entries belong to, over the whole selection. */
  cycles: ExecutionBrowseCycle[];
};

export type SavedCycleSummary = {
  cycle_id: string;
  label: string;
  phase: "sealed";
  created_at: string;
  updated_at: string;
  execution_count: number;
  archived: boolean;
};

export type ExecutionCheckpoint = Pick<NativeExecutionCheckpoint, "executionIdentifier" | "version"> & {
  position: CheckpointCursor;
  evidenceThrough: CheckpointCursor;
};

export type CheckpointCursor = Pick<NativeStorageCursor, "sequence"> & {
  history: Pick<NativeStoreHistory, "store" | "epoch">;
};

export type ExecutionMetadata = {
  execution_id: TestExecutionId;
  cycle_id: TestCycleId | null;
  case_id: CaseId;
  /** Null when the stored row no longer resolves its case revision. */
  template_id?: string | null;
  /** Null when the stored row no longer resolves its suite revision. */
  suite_label?: string | null;
  title: string;
  status: ExecutionStatus;
  started_at: Timestamp | null;
  finished_at: Timestamp | null;
  result: ResultSummary | null;
};

export type ExecutionStatus =
  | "skipped"
  | "invalidated"
  | "pending"
  | "running"
  | "passed"
  | "failed"
  | "blocked"
  | "cancelled"
  | "interrupted";

export type HostCardBlock =
  | { type: "text"; text: string }
  | { type: "status"; label: string; tone: HostCardTone; detail?: string | null }
  | { type: "key_value_list"; items?: HostCardKeyValue[] }
  | { type: "metrics"; items?: HostCardMetric[] };

export type HostCardBody = {
  blocks?: HostCardBlock[];
};

export type HostCardKeyValue = {
  label: string;
  value: string;
  detail?: string | null;
};

export type HostCardMetric = {
  label: string;
  value: string;
  unit?: string | null;
  trend?: string | null;
};

export type HostCardTone = "neutral" | "info" | "success" | "warning" | "danger";

export type Identifier = string;

export type Installation = {
  install_id: string;
  sha256: string;
  staged_at: Timestamp;
  consented_at: Timestamp | null;
  state: PluginState;
  last_boot: PluginLastBoot | null;
};

export type InstalledPluginErrorRecord = {
  plugin_id: Identifier;
  contribution_id: Identifier | null;
  phase: InstalledPluginPhase;
  level: DebugLevel;
  message: string;
  code: string | null;
  source: string | null;
  cause_chain: ErrorCause[];
  stacktrace_attachment: ArtifactId | null;
  trace_id: TraceId | null;
  scope: RecordScope | null;
  session_id: SessionId | null;
  test_cycle_id: TestCycleId | null;
  test_execution_id: TestExecutionId | null;
  recorded_at: Timestamp;
};

export type InstalledPluginPhase = "entrypoint" | "action" | "sidecar" | "provider" | "initialize" | "run" | "finalize" | "interrupt";

export type Metadata = {
  uuid?: string;
  created_at?: Timestamp;
  id?: string;
  identifier: Identifier;
  label: string;
  description?: string | null;
  tags?: string[];
};

export type MultipleChoice = {
  id: string;
  title: string;
  description: string | null;
  default: string;
  choices: Choice[];
  /** The id of the plugin route that fills this dropdown at run time. */
  provider?: string | null;
};

export type NodeId = string;

export type Number = {
  id: string;
  title: string;
  description: string | null;
  default: number | null;
  min: number | null;
  max: number | null;
  step: number | null;
};

export type OrganizationIdentity = {
  name: string;
  unit?: string | null;
  website?: string | null;
};

export type Path = {
  id: string;
  title: string;
  description: string | null;
  default: string;
  required?: boolean;
};

export type PlotCartesianFrame = {
  mode: PlotCartesianMode;
  series: PlotCartesianSeriesFrame[];
  x_domain: PlotDomain | null;
  y_domain: PlotDomain | null;
  point_count: number;
  signature: string;
};

export type PlotCartesianMode = "aligned" | "faceted";

export type PlotCartesianSeriesFrame = {
  x: (number | null)[];
  y: (number | null)[];
};

export type PlotDomain = {
  min: number | null;
  max: number | null;
};

export type PlotMarker = {
  x: Value;
  label: string;
  record_type: string;
  color: string;
};

export type PlotPayloadCursor = {
  telemetry_streams?: { [key in string]: number };
  record_sequence?: number | null;
};

export type PlotPayloadDiagnostics = {
  series_count: number;
  point_count: number;
  raw_point_count: number;
  downsampled: boolean;
  delta: boolean;
  max_points: number;
  window_s: number | null;
};

export type PlotPayloadEnvelope_Serialize = {
  plot_id: string;
  revision: number;
  cursor: PlotPayloadCursor;
  payload: PlotPayload_Serialize;
  diagnostics: PlotPayloadDiagnostics;
  warnings: string[];
};

export type PlotPayloadStreamKind = "snapshot" | "append" | "replace" | "error";

export type PlotPayload_Serialize = {
  stream: string;
  x_key: string;
  series_by: string | null;
  context: { [key in string]: Value };
  series: PlotSeries_Serialize[];
  cartesian?: PlotCartesianFrame | null;
  markers: PlotMarker[];
  max_points: number;
  window_s: number | null;
  generated_at: string;
  /** The resolved winner of the x-axis format precedence chain; `null` when none applies. */
  x_format?: ValueFormat | null;
  run_started_at?: string | null;
  cycle_started_at?: string | null;
};

export type PlotPoint_Serialize = {
  x: Value;
  y: number | null;
  z?: number | null;
};

export type PlotSeries_Serialize = {
  key: string;
  label: string;
  unit: string | null;
  points: PlotPoint_Serialize[];
  raw_points: number;
  downsampled: boolean;
  /** The resolved winner of the y-axis format precedence chain; `null` when none applies. */
  format?: ValueFormat | null;
};

export type PluginEntry = {
  metadata: PluginMetadata | null;
  error: string | null;
  /**  Whether this installation was admitted by this process. */
  active?: boolean;
  restart_required?: boolean;
  boot_error?: string | null;
} & Installation;

export type PluginEnvironmentReference = {
  owner: string;
  setting: string;
  variable: string;
};

export type PluginIdentity = {
  name: string;
  version: string;
  description: string;
};

export type PluginLastBoot = { state: "loaded"; at: Timestamp } | { state: "failed"; error: string; at: Timestamp };

export type PluginMetadata = {
  identity: PluginIdentity;
  owners: string[];
  size_bytes: number;
  environment_references: PluginEnvironmentReference[];
};

export type PluginState = "staged" | "installed" | "disabled" | "removal_pending";

export type RecordScope = {
  session_id?: SessionId | null;
  test_cycle_id?: TestCycleId | null;
  test_case_id?: CaseId | null;
  test_execution_id?: TestExecutionId | null;
  step_id?: StepId | null;
  criterion_id?: CriterionId | null;
  template_id?: TemplateId | null;
  suite_id?: SuiteId | null;
};

export type RedactedSecretBinding = {
  secret_id: string;
  backend: SecretBackendKind;
  configured: boolean;
};

export type ResultStatus = "pending" | "passed" | "failed" | "blocked" | "cancelled";

export type ResultSummary = {
  status: ResultStatus;
  passed: number;
  failed: number;
  blocked: number;
  skipped: number;
};

export type RuntimeAppLogo = {
  variant: "transparent_on_light" | "transparent_on_dark" | "contained_on_light" | "contained_on_dark";
  plugin_id: Identifier;
  asset_id: Identifier;
  url: string;
  alt?: string | null;
};

export type RuntimeAppearanceFrame = {
  app_name: string;
  organization?: OrganizationIdentity | null;
  title_template?: string | null;
  logos: RuntimeAppLogo[];
  themes?: RuntimeThemeFamily[];
  active_theme?: Identifier | null;
  theme_family_locked: boolean;
  mode_policy: ThemeModePolicy;
  mode_locked: boolean;
  source_plugin_id?: Identifier | null;
};

export type RuntimePluginSettings = {
  settings: Settings;
  values: { [key in string]: Value };
  /**
   * What a plugin's discovery routes computed, for the fields nobody saved.
   *
   * A value here is a suggestion rather than a setting: the card shows it with
   * a `Discovered` marker and no run uses it until it is saved.
   */
  discovered?: { [key in string]: Value };
  secret_bindings: PluginRuntimeSecretBindings;
};

export type RuntimeSettingsCard = {
  plugin_id: Identifier;
  card_id: Identifier;
  section_id: Identifier;
  label: string;
  description?: string | null;
  body: HostCardBody;
  refresh?: SettingsCardRefreshPolicy;
  state: RuntimeSettingsCardState;
  order: number;
};

export type RuntimeSettingsCardState = "ready" | "disabled" | "disconnected" | "stale" | "error" | "permission_denied";

export type RuntimeSettingsFrame = {
  sections: RuntimeSettingsSection[];
  cards: RuntimeSettingsCard[];
};

export type RuntimeSettingsSection = {
  section_id: Identifier;
  label: string;
  description?: string | null;
  owner: RuntimeSettingsSectionOwner;
  order: number;
};

export type RuntimeSettingsSectionOwner = { kind: "app" } | { kind: "installed_plugin"; plugin_id: Identifier };

export type RuntimeThemeFamily = {
  plugin_id: Identifier;
  theme_id: Identifier;
  label: string;
  description?: string | null;
  stylesheet?: string | null;
  variables?: { [key in string]: string };
  corner_radii?: ThemeCornerRadii;
  light: ThemeVariant;
  dark: ThemeVariant;
};

export type SecretReference = { kind: "environment"; variable: string } | { kind: "stored_secret"; secret_id: string };

export type SecretSetting = {
  id: string;
  title: string;
  description: string | null;
  default: SecretReference | null;
  required?: boolean;
};

export type ServerDiagnostics = {
  bind_addr: string;
  body_limit_bytes: number;
  auth: string;
  allowed_origins: string[];
  heartbeat_interval_ms: number;
  version: string;
  process: ServerProcessDiagnostics;
};

export type ServerProcessDiagnostics = {
  pid: number;
  cpu_percent: number | null;
  memory_bytes: number | null;
  virtual_memory_bytes: number | null;
};

export type SessionId = string;

export type Settings = {
  contributions?: SettingsEntry[];
};

export type SettingsCardRefreshPolicy = { kind: "manual" } | { kind: "cached"; max_age_ms: number } | { kind: "on_demand" };

export type SettingsEntry =
  | ({
      kind: "boolean";
    } & Boolean)
  | ({
      kind: "text_area";
    } & TextArea)
  | ({
      kind: "path";
    } & Path)
  | ({
      kind: "number";
    } & Number)
  | ({
      kind: "multiple_choice";
    } & MultipleChoice)
  | ({
      kind: "secret_reference";
    } & SecretSetting);

export type StartupDoctorCheck = {
  key: StartupDoctorCheckKey;
  status: DiagnosticCheckStatus;
  message: string;
  detail: string | null;
};

export type StartupDoctorCheckKey = "storage_root" | "shutdown_state";

export type StartupDoctorSnapshot = {
  checked_at: Timestamp;
  status: DiagnosticCheckStatus;
  checks: StartupDoctorCheck[];
};

export type StepId = string;

export type SuiteId = string;

export type TemplateId = string;

export type TestCycleId = string;

export type TestExecutionId = string;

export type TextArea = {
  id: string;
  title: string;
  description: string | null;
  default: string;
  required?: boolean;
};

export type ThemeCornerRadii = {
  base?: string | null;
  sm?: string | null;
  md?: string | null;
  lg?: string | null;
  xl?: string | null;
  "2xl"?: string | null;
  "3xl"?: string | null;
  "4xl"?: string | null;
};

export type ThemeModePolicy = "user_or_system" | "force_system" | "force_light" | "force_dark";

export type ThemeVariant = {
  variables?: { [key in string]: string };
  plot_palette?: string[];
};

export type Timestamp = string;

export type TraceId = string;

export type Value = "Null" | boolean | number | number | null | string | Value[] | { [key in string]: Value };
