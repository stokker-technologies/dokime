/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Canonical Document Capture Test Fixtures
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
import type {
  CardBinding,
  DocumentCapture_Serialize,
  DocumentCardDefinition_Serialize,
  DocumentNode,
  Metadata,
  PlotInput,
} from "./document-contracts";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */
interface CaptureFixture {
  sources: DocumentCardDefinition_Serialize[];
  bindings: CardBinding[];
  metadata?: Metadata;
  nodes?: DocumentNode[];
  sections?: Record<string, Metadata>;
}

/* === Classes === */

/* === Public Functions === */
export function documentCaptureFixture(input: CaptureFixture): DocumentCapture_Serialize {
  const metadata = input.metadata ?? { identifier: "default", label: "Plots" };
  return {
    definition: {
      document_id: metadata.identifier,
      metadata,
      cards: input.sources,
      layout: {
        nodes:
          input.nodes ??
          input.sources.map((card) => ({
            type: "card",
            data: { card_id: card.metadata.identifier },
          })),
      },
    },
    selection: { scope: {} },
    cards: input.bindings,
    sections: Object.fromEntries(
      Object.entries(input.sections ?? {}).map(([id, metadata]) => [id, { metadata }]),
    ),
    repeats: {},
    warnings: [],
  };
}

export function plotInputFixture(input: Partial<PlotInput> = {}): PlotInput {
  return {
    plot_id: "plot",
    scope: {},
    plot_type: "time_series",
    x_field: null,
    series_fields: [],
    z_stream: null,
    z_field: null,
    stream: null,
    series_by: null,
    unit: null,
    style: "line",
    x_scale: "linear",
    y_scale: "linear",
    fft: {},
    interval: {},
    histogram: {},
    marker_record_types: [],
    reference_lines: [],
    reference_bands: [],
    live_window_s: null,
    available_streams: [],
    marker_count: 0,
    ...input,
  };
}

/* === Private Functions === */

/* === Entry Point === */
