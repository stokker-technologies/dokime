/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import { serializedValueObjectToJson, serializedValueToJson } from "./api/native-values";
/* === Local Imports === */
import { assembleCapture, type DeclaredNode } from "./document-assembly";

import type {
  CardBinding,
  DocumentCapture_Serialize as DocumentCapturePayload,
  DocumentCardDefinition_Serialize,
  DocumentNode,
} from "./document-contracts";
import {
  documentCard,
  documentCardLayout,
  documentPlot,
  documentSummaryTile,
  documentTableRows,
  documentText,
  validateSummaryResult,
} from "./document-presentation";
import type { DocumentPlotType, DocumentViewCard, DocumentViewPayload } from "./types";

export type { DocumentCapture_Serialize as DocumentCapturePayload } from "./document-contracts";

/* === Public Functions === */

export function mapDocumentCapture(document: DocumentCapturePayload): DocumentViewPayload {
  if (document.definition.cards.length > 4096 || document.cards.length > 4096)
    throw new Error("Captured document sources or bindings exceed their contract");
  const sources = new Map(
    document.definition.cards.map((card) => [card.metadata.identifier, card]),
  );
  let declared = 0;
  const nodes = (source: DocumentNode[], depth = 1): DeclaredNode[] => {
    if (depth > 32 && source.length > 0)
      throw new Error("Captured document depth exceeds its budget");
    return source.map((node) => {
      if (++declared > 4096) throw new Error("Captured document declarations exceed their budget");
      return node.type === "card"
        ? { type: "card", card_id: node.data.card_id, layout: documentCardLayout(node.data.layout) }
        : {
            type: "section",
            node_id: node.data.node_id,
            repeat: Boolean(node.data.repeat),
            children: nodes(node.data.children, depth + 1),
          };
    });
  };
  return assembleCapture({
    document_id: document.definition.document_id,
    metadata: document.definition.metadata,
    sources: document.definition.cards.map((card) => card.metadata.identifier),
    bindings: document.cards.map((binding) => {
      const source = sources.get(binding.source_id);
      if (!source)
        throw new Error(`Captured card binding has an invalid source: ${binding.source_id}`);
      return {
        source_id: binding.source_id,
        card: mapCardBinding(source, binding),
      };
    }),
    nodes: nodes(document.definition.layout.nodes),
    repeats: new Map(Object.entries(document.repeats)),
    sections: new Map(Object.entries(document.sections)),
  });
}

/* === Private Functions === */

function mapCardBinding(
  card: DocumentCardDefinition_Serialize,
  binding: CardBinding,
): DocumentViewCard {
  const result = binding.result;
  const metadata = binding.metadata;
  if (
    (card.type === "telemetry_summary" && result.type !== "summary") ||
    (card.type === "plot" && result.type !== "plot") ||
    (card.type !== "telemetry_summary" && card.type !== "plot" && result.type !== "static")
  ) {
    throw new Error("Captured card result differs from its declaration");
  }
  if (result.type === "summary") {
    return documentCard(metadata, "overview", {
      groups: result.data.groups.map((group) => ({
        group_id: group.metadata.identifier,
        title: group.metadata.label,
        metrics: group.tiles.map((tile) => {
          validateSummaryResult(tile);
          return documentSummaryTile({
            ...tile,
            value: tile.value === null ? null : serializedValueToJson(tile.value),
          });
        }),
      })),
    });
  }
  if (result.type === "plot") {
    const descriptor = result.data.descriptor;
    const input = descriptor.input;
    const plot = documentPlot({
      plot_id: descriptor.plot_id,
      title: descriptor.metadata.label,
      plot_type: mapPlotType(descriptor.plot_type),
      stream: input.stream ?? undefined,
      available_streams: input.available_streams,
      style: input.style,
      x_scale: input.x_scale,
      y_scale: input.y_scale,
      fft: input.fft,
      analysis: input.analysis
        ? {
            ...input.analysis,
            parameters: serializedValueObjectToJson(input.analysis.parameters),
          }
        : null,
      interval: input.interval,
      histogram: input.histogram,
      y: input.series_fields,
      x: input.x_field,
      x_expression: input.x_expression,
      y_expressions: input.y_expressions,
      series_by_expression: input.series_by_expression,
      z_expression: input.z_expression,
      z_stream: input.z_stream,
      z_field: input.z_field,
      series_by: input.series_by,
      unit: input.unit,
      x_format: input.x_format,
      y_format: input.y_format,
      x_axis: input.x_axis,
      description: descriptor.metadata.description,
      marker_record_types: input.marker_record_types,
      filters: input.filters?.map((filter) => ({
        ...filter,
        value:
          filter.value === undefined || filter.value === null
            ? filter.value
            : serializedValueToJson(filter.value),
      })),
      reference_lines: input.reference_lines,
      reference_bands: input.reference_bands,
      live_window_s: input.live_window_s,
      display_points: input.display_points,
      downsample: input.downsample,
    });
    return documentCard(metadata, "plots", { plots: [plot] });
  }
  if (card.type === "table") {
    return documentCard(metadata, "table", {
      rows: documentTableRows(metadata.identifier, card.data.rows.map(serializedValueObjectToJson)),
    });
  }
  if (card.type === "artifacts") {
    return documentCard(metadata, "artifacts");
  }
  if (card.type === "media") {
    return documentCard(metadata, "media");
  }
  if (
    card.type === "steps" ||
    card.type === "criteria" ||
    card.type === "traceability" ||
    card.type === "notes" ||
    card.type === "records" ||
    card.type === "health" ||
    card.type === "telemetry_context"
  ) {
    return documentCard(metadata, card.type);
  }

  if (card.type === "text") {
    return documentCard(metadata, "overview", {
      groups: [documentText(metadata, card.data.markdown)],
    });
  }
  throw new Error("Captured card result differs from its declaration");
}

function mapPlotType(value: string): DocumentPlotType {
  if (value === "xy") return "xy_scatter";
  if (value === "frequency") return "frequency_series";
  if (
    value === "time_series" ||
    value === "bar" ||
    value === "xy_scatter" ||
    value === "xy_line" ||
    value === "xy_interval" ||
    value === "trajectory" ||
    value === "heatmap" ||
    value === "xy_value_heatmap" ||
    value === "wifi_channel_occupancy" ||
    value === "frequency_series" ||
    value === "histogram" ||
    value === "box_summary"
  ) {
    return value;
  }
  throw new Error(`Unsupported document plot type: ${value}`);
}
