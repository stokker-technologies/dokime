<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import DocumentViewRenderer from "@/components/document/DocumentViewRenderer.svelte";
  import PlotGroupGrid from "@/components/document/PlotGroupGrid.svelte";
  import type { DocumentViewCard, DocumentViewPayload, DocumentViewPlot } from "@/lib/types";
  import { emptyLivePlotState, type LivePlotStates } from "@/plots/live-frames";

  let { frozen = false }: { frozen?: boolean } = $props();

  const layout = { col_span: { base: 12 }, row_span: 1, order: 0, density: "normal", rail: "main" } as const;
  const plots: DocumentViewPlot[] = [
    { plot_id: "voltage", title: "Voltage", unit: "V", description: "Recorded voltage over the shown window." },
    { plot_id: "current", title: "Current", unit: "A", description: "Recorded current over the shown window." },
  ].map((plot) => ({ ...plot, plot_type: "time_series", stream: "readings", x: "elapsed_s", y: ["value"] }) as DocumentViewPlot);
  const cards: DocumentViewCard[] = plots.map((plot) => ({
    card_id: plot.plot_id,
    kind: "plots",
    title: plot.title,
    layout,
    region: "main",
    groups: [],
    plots: [plot],
    rows: [],
    show_report_outcome: false,
    config: {},
  }));
  const documentView: DocumentViewPayload = {
    document_id: "default",
    metadata: { identifier: "default", label: "Document" },
    title: "Document",
    desktop_sidebar_min_px: 0,
    desktop_sidebar_max_px: 0,
    cards,
    layout: {
      nodes: cards.map((card, index) => ({
        type: "section",
        node_id: `section-${index}`,
        metadata: { identifier: `section-${index}`, label: index ? "Readings" : "Host" },
        children: [{ type: "card", card_id: card.card_id, layout }],
      })),
    },
  };
  const liveStates: LivePlotStates = new Map(
    plots.map((plot, index) => [
      plot.plot_id,
      {
        ...emptyLivePlotState(),
        status: "live",
        payload: {
          stream: "readings",
          x_key: "elapsed_s",
          max_points: 120,
          generated_at: "2026-01-01T00:00:00Z",
          series: [{ key: plot.plot_id, label: plot.title, unit: plot.unit, raw_points: 120, downsampled: false, points: [] }],
          cartesian: {
            mode: "aligned",
            point_count: 120,
            signature: plot.plot_id,
            series: [{ x: Array.from({ length: 120 }, (_, row) => row), y: Array.from({ length: 120 }, (_, row) => index + (row % 12)) }],
          },
          summaries: [
            {
              series_id: plot.plot_id,
              count: 120,
              min: index,
              max: index + 11,
              mean: index + 5.5,
              median: index + 5.5,
              std: Math.sqrt(143 / 12),
              rms: index ? 7.36 : 6.49,
            },
          ],
        },
      },
    ]),
  );
</script>

<main class="mx-auto max-w-6xl p-6">
  <DocumentViewRenderer document={documentView}>
    {#snippet renderCard(
      card,
      section,
    )}
      <PlotGroupGrid
        documentPlots={card.plots}
        {liveStates}
        title={section?.label ?? card.title}
        showGroupHeaders={false}
        viewportMode={frozen ? "fit" : "live"}
        zoom={frozen}
        onPlotSettingsChange={frozen ? undefined : async () => {}}
      />
    {/snippet}
  </DocumentViewRenderer>
</main>
