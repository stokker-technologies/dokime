<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script module lang="ts">
  import type { Snippet } from "svelte";
  import type { DocumentViewCard, DocumentViewNode, DocumentViewPayload, Metadata } from "../../lib/types";

  export type DocumentBlockChrome = "normal" | "embedded" | "flat";
  export type DocumentSectionChrome = "plain" | "grouped" | "card";

  export type DocumentViewRendererProps = {
    document: DocumentViewPayload | null | undefined;
    renderCard: Snippet<[DocumentViewCard, Metadata | null]>;
    cardAction?: Snippet<[DocumentViewCard]>;
    hasCardAction?: (card: DocumentViewCard) => boolean;
    sectionAction?: Snippet<[string]>;
    hasSectionAction?: (sectionId: string) => boolean;
    cardFilter?: (card: DocumentViewCard) => boolean;
    cardChrome?: (card: DocumentViewCard) => DocumentBlockChrome;
    emptyMessage?: string;
    sectionChrome?: DocumentSectionChrome;
    onRetry?: (cardId?: string) => void;
  };

  function layoutNodeKey(node: DocumentViewNode): string {
    return node.type === "card" ? `card:${node.card_id}` : `section:${node.node_id}`;
  }

  function normalizedLabel(value: string): string {
    return value
      .toLowerCase()
      .replaceAll(/[\s_-]+/g, " ")
      .trim();
  }
</script>

<script lang="ts">
  import {
    documentCardForPlacement,
    documentCardLayoutAttributes,
    documentCardLayoutStyle,
    documentNodeHasCards,
  } from "../../lib/document-layout";
  import { failureOf } from "../../lib/notification-details";
  import { cn } from "../../lib/utils";
  import ErrorCard from "../ErrorCard.svelte";

  let {
    document,
    renderCard,
    cardAction,
    hasCardAction,
    sectionAction,
    hasSectionAction,
    cardFilter = () => true,
    cardChrome = (card) => (card.kind === "plots" ? "flat" : "normal"),
    emptyMessage = "This document does not currently expose any cards.",
    sectionChrome = "plain",
    onRetry,
  }: DocumentViewRendererProps = $props();

  const cardMap = $derived(new Map(document?.cards.map((card) => [card.card_id, card]) ?? []));
  const nodes = $derived(visibleNodes(document?.layout.nodes ?? []));
  const sectionHeading = "grid min-w-0 grid-cols-[minmax(0,1fr)_auto] items-center gap-2";
  const descriptionStyle = "text-sm text-muted-foreground [overflow-wrap:anywhere]";

  function visibleNodes(nodes: DocumentViewNode[]): DocumentViewNode[] {
    return nodes.filter((node) => documentNodeHasCards(node, cardMap, cardFilter));
  }

  function sameTitleCard(node: Extract<DocumentViewNode, { type: "section" }>): DocumentViewCard | null {
    const child = node.children[0];
    if (node.children.length !== 1 || child.type !== "card") {
      return null;
    }
    const card = documentCardForPlacement(cardMap, child);
    return card && normalizedLabel(node.metadata.label) === normalizedLabel(card.title) ? card : null;
  }
</script>

{#snippet sectionControl(
  sectionId: string,
)}
  {#if sectionAction && (hasSectionAction?.(sectionId) ?? true)}
    <div class="flex min-w-0 items-center">{@render sectionAction(sectionId)}</div>
  {/if}
{/snippet}

{#snippet cardBlock(
  card: DocumentViewCard,
  parent: Metadata | null,
  foldedSectionId?: string,
)}
  {@const selectedChrome = cardChrome(card)}
  {@const chrome = sectionChrome === "card" && selectedChrome === "normal" ? "embedded" : selectedChrome}
  {@const eyebrow = card.kind === "telemetry_context" ? "Telemetry" : card.kind.replaceAll("_", " ")}
  {@const useCardAction = cardAction && (hasCardAction?.(card) ?? true)}
  {@const spans = documentCardLayoutAttributes(card.layout)}
  <section
    class={cn(
      "document-card grid min-w-0",
      chrome === "normal" && "gap-3 rounded-lg border bg-background/60 p-3",
      chrome === "embedded" && "gap-3",
      chrome === "flat" && "gap-2",
    )}
    {...spans}
    data-density={card.layout.density}
    style={documentCardLayoutStyle(card.layout, spans)}
  >
    {#if chrome !== "flat"}
      <header class={cn("grid min-w-0 gap-1", chrome === "embedded" && "px-1")}>
        {#if normalizedLabel(eyebrow) !== normalizedLabel(card.title)}
          <p class="text-xs font-bold uppercase text-muted-foreground">{eyebrow}</p>
        {/if}
        {#if useCardAction || foldedSectionId !== undefined}
          <div class={sectionHeading}>
            <h5 class="truncate text-sm font-semibold">{card.title}</h5>
            <div class="-my-0.5 flex items-center">
              {#if useCardAction && cardAction}
                {@render cardAction(card)}
              {:else if foldedSectionId !== undefined}
                {@render sectionControl(foldedSectionId)}
              {/if}
            </div>
          </div>
        {:else}
          <h5 class="truncate text-sm font-semibold">{card.title}</h5>
        {/if}
        {#if card.metadata?.description}
          <p class={descriptionStyle}>{card.metadata.description}</p>
        {/if}
      </header>
    {/if}
    <div class="grid min-w-0 gap-3">
      {#if card.refreshing}
        <p role="status" class="text-xs text-muted-foreground">Refreshing…</p>
      {/if}
      {@render renderCard(card, parent)}
    </div>
  </section>
{/snippet}

{#snippet layoutNode(
  node: DocumentViewNode,
  parent: Metadata | null,
)}
  {#if node.type === "card"}
    {@const card = documentCardForPlacement(cardMap, node)}
    {#if card && cardFilter(card)}
      {@render cardBlock(card, parent)}
    {/if}
  {:else}
    {@const folded = sameTitleCard(node)}
    {#if folded}
      {#if cardFilter(folded)}
        {@render cardBlock(folded, node.metadata, node.metadata.identifier)}
      {/if}
    {:else}
      {@const children = visibleNodes(node.children)}
      {@const chrome = sectionChrome === "plain" && node.metadata.tags?.includes("chrome:grouped") ? "grouped" : sectionChrome}
      {#if children.length || node.unavailable || node.loading}
        {#snippet heading()}
          <div class={sectionHeading}>
            <h4 class="document-section-title truncate">{node.metadata.label}</h4>
            {@render sectionControl(node.metadata.identifier)}
          </div>
          {#if node.metadata.description}
            <p class={descriptionStyle}>{node.metadata.description}</p>
          {/if}
        {/snippet}
        {#snippet content()}
          {#if node.unavailable}
            <ErrorCard failure={failureOf(node.unavailable, node.metadata.label)} {onRetry} />
          {:else if node.loading}
            <p role="status" class="text-sm text-muted-foreground">Loading…</p>
          {:else}
            {#if node.refreshing}
              <p role="status" class="text-xs text-muted-foreground">Refreshing…</p>
            {/if}
            <div class="document-grid grid min-w-0 grid-cols-1 gap-3 md:grid-cols-12">
              {#each children as child (layoutNodeKey(child))}
                {@render layoutNode(child, node.metadata)}
              {/each}
            </div>
          {/if}
        {/snippet}
        {#if chrome === "card"}
          <div
            class="group/card col-span-full flex flex-col overflow-hidden border-t border-border bg-transparent text-sm text-card-foreground"
            data-slot="card"
            data-size="default"
            data-section-id={node.metadata.identifier}
          >
            <div
              class="group/card-header @container/card-header grid auto-rows-min items-start gap-1 border-b px-4 py-3 [.border-b]:pb-4"
              data-slot="card-header"
            >
              {@render heading()}
            </div>
            <div class="grid gap-4 p-4" data-slot="card-content">{@render content()}</div>
          </div>
        {:else}
          <section
            class={cn(
              "col-span-full grid min-w-0",
              chrome === "grouped"
                ? "gap-4 rounded-lg border border-border/70 border-l-4 border-l-primary/70 bg-muted/15 px-4 py-3"
                : "gap-3",
            )}
            data-section-id={node.metadata.identifier}
          >
            <header class={cn("grid min-w-0 gap-1", chrome === "grouped" && "border-b border-border/60 pb-2")}>
              {@render heading()}
            </header>
            {@render content()}
          </section>
        {/if}
      {/if}
    {/if}
  {/if}
{/snippet}

{#if document}
  <div class="grid min-w-0 gap-6">
    {#each nodes as node (layoutNodeKey(node))}
      {@render layoutNode(node, null)}
    {:else}
      <div class="grid min-h-48 place-items-center gap-2 rounded-lg border border-dashed bg-muted/15 px-6 py-8 text-center">
        <strong class="text-base font-semibold">No Content In This Document Yet</strong>
        <p class="max-w-2xl text-sm text-muted-foreground">{emptyMessage}</p>
      </div>
    {/each}
  </div>
{:else}
  <p class="text-sm text-muted-foreground">No document has been resolved for the selected scope yet.</p>
{/if}
