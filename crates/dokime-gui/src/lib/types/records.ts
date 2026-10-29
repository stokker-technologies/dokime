/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import type { JsonObject, RecordScope } from "../types";
import type { ValueFormat } from "./format";
import type { TelemetryContractSpec } from "./telemetry";

export type Severity = "debug" | "info" | "warning" | "error";

export type RecordIconKey =
  | "activity"
  | "alert-triangle"
  | "archive"
  | "bell"
  | "check-circle"
  | "circle-dashed"
  | "clipboard-check"
  | "clock"
  | "cpu"
  | "file-text"
  | "flag"
  | "gauge"
  | "heart-pulse"
  | "help-circle"
  | "message-square-text"
  | "package"
  | "plug"
  | "radio-tower"
  | "rotate-ccw"
  | "shield-alert"
  | "target"
  | "timer-reset"
  | "wrench"
  | "zap";

export type RecordTone = "accent" | "danger" | "info" | "muted" | "neutral" | "success" | "warning";

export type RecordDisplay = {
  icon: RecordIconKey;
  tone: RecordTone;
};

export type ContractValidationMode = "strict" | "prototype";

export type ContractValueType = "number" | "integer" | "string" | "boolean" | "datetime" | "object" | "array";

export type HubRecord = {
  record_id: string;
  record_type: string;
  template_id?: string | null;
  source: string;
  timestamp: string;
  session_id?: string | null;
  sequence?: number | null;
  level: Severity;
  display: RecordDisplay;
  message?: string | null;
  details: JsonObject;
  related_sample_id?: string | null;
  related_stream?: string | null;
  related_artifact_id?: string | null;
  scope?: RecordScope | null;
};

export type ContractFieldSpec = {
  name: string;
  value_type: ContractValueType;
  required: boolean;
  unit?: string | null;
  label?: string | null;
  description?: string | null;
  choices: string[];
  default?: unknown;
  format?: ValueFormat | null;
};

export type RecordTypeSpec = {
  record_type: string;
  schema_version: number;
  template_id?: string | null;
  owner?: string | null;
  description?: string | null;
  default_level: Severity;
  fields: ContractFieldSpec[];
  details_schema?: JsonObject | null;
  examples: JsonObject[];
  display: RecordDisplay;
  validation_mode?: ContractValidationMode | null;
  allow_extra_fields: boolean;
};

export type PromptValidation = {
  min_length?: number | null;
  max_length?: number | null;
  pattern?: string | null;
  minimum?: number | null;
  maximum?: number | null;
  min_items?: number | null;
  max_items?: number | null;
};

export type PromptField = {
  name: string;
  label: string;
  kind: "text" | "float" | "int" | "choice" | "boolean";
  multiple?: boolean;
  required?: boolean;
  default?: unknown;
  choices?: string[];
  readonly?: boolean;
  validation?: PromptValidation;
};

export type PromptResponseSource = "gui" | "api" | "webhook" | "verification";

export type PromptStatus = "pending" | "answered" | "cancelled" | "expired";

export type PromptResponse = {
  correlation_id: string;
  action: string;
  values: JsonObject;
  source: PromptResponseSource;
  responded_at: string;
  scope?: RecordScope | null;
};

export type RecordVerificationCondition = {
  kind: "record";
  record_type: string;
  details: JsonObject;
};

export type TelemetryVerificationCondition = {
  kind: "telemetry";
  stream: string;
  field: string;
  operator: "within" | "eq" | "gte" | "lte" | "gt" | "lt";
  target: number;
  tolerance?: number | null;
  unit?: string | null;
};

export type PromptVerification = {
  condition: RecordVerificationCondition | TelemetryVerificationCondition;
};

export type PromptRequest = {
  correlation_id: string;
  template_id: string;
  prompt_type?: string | null;
  schema_version?: number | null;
  kind: "form" | "confirmation" | "choice" | "status" | "verification";
  /** The author's declared level, on the record severity scale. */
  level: Severity;
  title: string;
  message?: string | null;
  fields: PromptField[];
  actions: Array<{
    id: string;
    label: string;
    emphasis: "primary" | "secondary" | "danger";
    requires_condition?: boolean;
  }>;
  context: JsonObject;
  webhook_context: JsonObject;
  verification?: PromptVerification | null;
  status: PromptStatus;
  response?: PromptResponse | null;
  response_source?: PromptResponseSource | null;
  expires_at?: string | null;
  answered_at?: string | null;
  created_at: string;
  scope?: RecordScope | null;
};

export type PromptTypeSpec = {
  prompt_type: string;
  schema_version: number;
  template_id?: string | null;
  owner?: string | null;
  kind: PromptRequest["kind"];
  description?: string | null;
  fields: PromptField[];
  actions: PromptRequest["actions"];
  response_schema?: JsonObject | null;
  examples: JsonObject[];
  validation_mode?: ContractValidationMode | null;
};

export type ContractRegistrySnapshot = {
  schema_version: number;
  session_id: string;
  revision: number;
  records: RecordTypeSpec[];
  telemetry: TelemetryContractSpec[];
  prompts: PromptTypeSpec[];
  generated_at: string;
};
