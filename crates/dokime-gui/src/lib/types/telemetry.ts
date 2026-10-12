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
import type { ContractValidationMode } from "./records";

/* === Telemetry stream/spec/sample types === */

export type TelemetryColumnSpec = {
  name: string;
  value_type: "number" | "integer" | "string" | "boolean" | "datetime";
  unit?: string | null;
  label?: string | null;
  role: "value" | "counter" | "state" | "distance" | "latency" | "throughput";
  display_unit?: string | null;
  required: boolean;
  format?: ValueFormat | null;
};

export type TelemetryStreamSpec = {
  stream: string;
  label?: string | null;
  columns: TelemetryColumnSpec[];
  default_x?: string | null;
  plot_window_s?: number | null;
  allow_extra_fields: boolean;
};

export type TelemetryContractSpec = {
  stream: string;
  schema_version: number;
  template_id?: string | null;
  owner?: string | null;
  spec: TelemetryStreamSpec;
  sample_schema?: JsonObject | null;
  summary_schema?: JsonObject | null;
  examples: JsonObject[];
  validation_mode?: ContractValidationMode | null;
};

export type TelemetryStream = {
  stream: string;
  spec?: TelemetryStreamSpec | null;
  summary: JsonObject;
  samples: JsonObject[];
  max_samples: number;
  total_samples: number;
  dropped_samples: number;
  updated_at: string;
};

export type TelemetryStreamLiveSummary = Omit<TelemetryStream, "samples"> & {
  retained_sample_count: number;
  samples?: JsonObject[];
};

export type TelemetrySampleRecord = {
  stream: string;
  sequence: number;
  sample: JsonObject;
  scope?: RecordScope | null;
};
