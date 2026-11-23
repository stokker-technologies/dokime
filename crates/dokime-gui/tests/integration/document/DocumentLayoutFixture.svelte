<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import DocumentCardContent from "@/components/document/DocumentCardContent.svelte";
  import DocumentViewRenderer, { type DocumentSectionChrome } from "@/components/document/DocumentViewRenderer.svelte";
  import PlotGroupGrid from "@/components/document/PlotGroupGrid.svelte";
  import type { DocumentViewPayload, DocumentViewPlot } from "@/lib/types";

  type Props = { document: DocumentViewPayload; sectionChrome?: DocumentSectionChrome; headingsOnly?: boolean };
  let { document, sectionChrome, headingsOnly = false }: Props = $props();
</script>

<DocumentViewRenderer {document} {sectionChrome}>
  {#snippet renderCard(
    card,
  )}
    {#if headingsOnly}
      <PlotGroupGrid
        documentPlots={[
          {
            plot_id: card.card_id,
            title: `${card.title} Plot`,
            plot_type: "time_series",
            stream: "readings",
            y: ["value"],
          } as DocumentViewPlot,
        ]}
        title={card.title}
        showGroupHeaders={false}
        viewportMode="fit"
      />
    {:else if card.kind === "overview"}
      <DocumentCardContent
        {card}
        documentId={document.document_id}
        sectionTitle={card.title}
        renderMode="report"
        artifacts={[]}
        streams={[]}
        criterionRecords={[]}
        records={[]}
        health={[]}
        mediaArtifacts={[]}
        notes={[]}
        stepRecords={[]}
        traceability={null}
        reportPlots={{}}
        reportTelemetry={null}
      />
    {:else}
      <p>{card.title}</p>
    {/if}
  {/snippet}
</DocumentViewRenderer>
