/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Local Imports === */

import { serializedValueObjectToJson } from "@/lib/api/native-values";
import type { AnalysisSelection } from "@/lib/document-contracts";
import type { ReportPayload } from "@/lib/report-types";
import type { EvidenceAnalysisResult, PlotPayload } from "@/lib/types";

/* === Constants === */

const REPORT_DATA_ID = "hub-report-data";

const EMPTY_REPORT_PAYLOAD: ReportPayload = {
  schema_version: 1,
  generated_at: null,
  snapshot: null,
  documents: {},
  test_cycle_plan: null,
  test_executions: [],
  plots: {},
  histograms: {},
  box_summaries: {},
  case_runs: [],
  cycle_summary: null,
  warnings: ["Report data is unavailable."],
};

/* === Public Functions === */

export function readReportPayload(): ReportPayload {
  const node = document.getElementById(REPORT_DATA_ID);
  const text = node?.textContent?.trim();
  if (!text) {
    return EMPTY_REPORT_PAYLOAD;
  }

  try {
    const parsed = JSON.parse(text) as Partial<ReportPayload>;
    return {
      ...EMPTY_REPORT_PAYLOAD,
      ...parsed,
      test_executions: parsed.test_executions ?? [],
      documents: parsed.documents ?? {},
      plots: capturedPlots(parsed.plots),
      histograms: parsed.histograms ?? {},
      box_summaries: parsed.box_summaries ?? {},
      case_runs: (parsed.case_runs ?? []).map((caseRun) => ({
        ...caseRun,
        documents: caseRun.documents ?? {},
        plots: capturedPlots(caseRun.plots),
      })),
      cycle_summary: parsed.cycle_summary ?? null,
      warnings: parsed.warnings ?? [],
    };
  } catch (exc) {
    return {
      ...EMPTY_REPORT_PAYLOAD,
      warnings: [`Report data could not be parsed: ${String(exc)}`],
    };
  }
}

/* === Types === */

type CapturedAnalysis = {
  definition: Omit<EvidenceAnalysisResult["definition"], "maxSamples" | "maxOutputValues"> & {
    limits: { max_samples: number; max_output_values: number };
  };
  selection: AnalysisSelection;
  source: {
    epoch: string;
    stream: string;
    scope: EvidenceAnalysisResult["source"]["scope"];
    cursor: { telemetry_streams: Record<string, string>; record_sequence?: string };
    window_s?: number;
    fields: string[];
    x_key: string;
  };
  series: {
    key: string;
    label: string;
    origin?: string;
    input_samples: number;
    first_axis: number;
    last_axis: number;
    display_indices: Record<string, number[]>;
    output: {
      outputs: { key: string; unit: string; axis_unit: string; axis: number[]; values: number[] }[];
    };
  }[];
};

/* === Private Functions === */

function capturedPlots(plots?: Record<string, PlotPayload>): Record<string, PlotPayload> {
  return Object.fromEntries(
    Object.entries(plots ?? {}).map(([key, payload]) => {
      if (!payload.computed) return [key, payload];
      const captured = payload.computed as unknown as CapturedAnalysis;
      const computed: EvidenceAnalysisResult = {
        definition: {
          ...captured.definition,
          maxSamples: captured.definition.limits.max_samples,
          maxOutputValues: captured.definition.limits.max_output_values,
        },
        selection: {
          ...captured.selection,
          parameters: serializedValueObjectToJson(captured.selection.parameters),
        },
        source: {
          epoch: captured.source.epoch,
          stream: captured.source.stream,
          scope: captured.source.scope,
          telemetryStreams: captured.source.cursor.telemetry_streams,
          recordSequence: captured.source.cursor.record_sequence,
          windowS: captured.source.window_s,
          fields: captured.source.fields,
          xKey: captured.source.x_key,
        },
        series: captured.series.map((series) => ({
          key: series.key,
          label: series.label,
          origin: series.origin,
          inputSamples: series.input_samples,
          firstAxis: series.first_axis,
          lastAxis: series.last_axis,
          outputs: series.output.outputs.map((output) => ({
            key: output.key,
            unit: output.unit,
            axisUnit: output.axis_unit,
            axis: output.axis,
            values: output.values,
            displayIndices: series.display_indices[output.key],
          })),
        })),
      };
      return [key, { ...payload, computed }];
    }),
  );
}
