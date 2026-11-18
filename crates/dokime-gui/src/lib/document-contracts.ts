/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Shared document presentation values decoded from native host responses. */

export type DataExpression = {
  label?: string | null;
  source: DataSource;
  transforms?: DataTransform[];
};

export type DataSource =
  | { type: "telemetry"; stream: string; field: string }
  | { type: "parameter"; name: string }
  | { type: "dimension"; name: string }
  | { type: "record"; record_type: string }
  | { type: "artifact"; artifact: string }
  | { type: "literal"; value: unknown };

/* One member of the Harmos aggregate vocabulary by its display name; a
   percentile is `p` and a whole rank from 0 to 100. */
export type Aggregate = "count" | "sum" | "mean" | "median" | "min" | "max" | "rms" | "std" | "first" | "last" | `p${number}`;

export type DataTransform =
  | { type: "scale"; factor: number }
  | { type: "offset"; value: number }
  | { type: "aggregate"; aggregate: Aggregate }
  | { type: "time_between"; previous_record_type: string }
  | { type: "detail"; name: string }
  | { type: "per_case" };

export type DocumentCardDensity = "compact" | "normal" | "spacious";

export type DocumentCardLayout = {
  col_span?: ResponsiveSpan;
  row_span?: number;
  order?: number;
  density?: DocumentCardDensity;
  rail?: DocumentCardRail;
};

export type DocumentCardRail = "main" | "right" | "left";

export type Identifier = string;

export type Metadata = {
  uuid?: string;
  created_at?: Timestamp;
  id?: string;
  identifier: Identifier;
  label: string;
  description?: string | null;
  tags?: string[];
};

export type ReferenceValue = number | DataExpression;

export type ReferenceBand = {
  min?: ReferenceValue | null;
  max?: ReferenceValue | null;
  label?: string | null;
  color?: string;
  center?: ReferenceValue | null;
  tolerance?: ReferenceValue | null;
};

export type ReferenceLine = {
  value: ReferenceValue;
  label?: string | null;
  color?: string;
};

export type ResponsiveSpan = {
  base?: number;
  sm?: number | null;
  md?: number | null;
  lg?: number | null;
  xl?: number | null;
};

export type TelemetrySummaryFormat = "auto" | "number" | "percent" | "duration" | "distance" | "status" | "text";

export type TelemetrySummaryPresentation = "value" | "progress" | "badge";

export type TelemetrySummaryState = "ok" | "warning" | "critical" | "muted";

export type Timestamp = string;
