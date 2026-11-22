/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Local Imports === */

import type { DocumentCardLayout, DocumentViewCard, DocumentViewNode, DocumentViewPayload, DocumentViewSection } from "@/lib/types";

/* === Types === */

export type DocumentCardLayoutAttributes = {
  "data-col-span-base": number;
  "data-col-span-sm": number;
  "data-col-span-md": number;
  "data-col-span-lg": number;
  "data-col-span-xl": number;
};

/* === Constants === */

/* === Public Functions === */

export function orderedDocumentSections(document: DocumentViewPayload | null | undefined): DocumentViewSection[] {
  return normalizedDocumentSections(document).map((section) => ({
    ...section,
    cards: orderedSectionCards(section.cards),
  }));
}

export function orderedSectionCards(cards: DocumentViewCard[]): DocumentViewCard[] {
  return cards
    .map((card, index) => ({ card, index }))
    .sort((left, right) => left.card.layout.order - right.card.layout.order || left.index - right.index)
    .map((item) => item.card);
}

export function documentCardLayoutStyle(layout: DocumentCardLayout, spans = documentCardLayoutAttributes(layout)): string {
  return [
    `--document-card-col-span-base:${spans["data-col-span-base"]}`,
    `--document-card-col-span-sm:${spans["data-col-span-sm"]}`,
    `--document-card-col-span-md:${spans["data-col-span-md"]}`,
    `--document-card-col-span-lg:${spans["data-col-span-lg"]}`,
    `--document-card-col-span-xl:${spans["data-col-span-xl"]}`,
    `--document-card-row-span:${Math.max(1, layout.row_span)}`,
    `order:${layout.order}`,
  ].join(";");
}

export function documentCardLayoutAttributes(layout: DocumentCardLayout): DocumentCardLayoutAttributes {
  return {
    "data-col-span-base": clampedSpan(layout.col_span.base),
    "data-col-span-sm": clampedSpan(layout.col_span.sm ?? layout.col_span.base),
    "data-col-span-md": clampedSpan(layout.col_span.md ?? layout.col_span.sm ?? layout.col_span.base),
    "data-col-span-lg": clampedSpan(layout.col_span.lg ?? layout.col_span.md ?? layout.col_span.sm ?? layout.col_span.base),
    "data-col-span-xl": clampedSpan(
      layout.col_span.xl ?? layout.col_span.lg ?? layout.col_span.md ?? layout.col_span.sm ?? layout.col_span.base,
    ),
  };
}

function clampedSpan(value: number): number {
  return Math.min(12, Math.max(1, Math.round(value)));
}

function normalizedDocumentSections(document: DocumentViewPayload | null | undefined): DocumentViewSection[] {
  if (!document) return [];
  const cardMap = new Map(document.cards.map((card) => [card.card_id, card]));
  return sectionsFromNodes(document.layout.nodes, cardMap);
}

function sectionsFromNodes(nodes: DocumentViewNode[], cardMap: Map<string, DocumentViewCard>): DocumentViewSection[] {
  return nodes.flatMap((node) => {
    if (node.type === "card") {
      const card = documentCardForPlacement(cardMap, node);
      return card
        ? [
            {
              metadata: { identifier: "default", label: "Default" },
              cards: [card],
            },
          ]
        : [];
    }

    return [
      {
        metadata: node.metadata,
        unavailable: node.unavailable,
        loading: node.loading,
        cards: cardsFromNodes(node.children, cardMap),
      },
    ];
  });
}

function cardsFromNodes(nodes: DocumentViewNode[], cardMap: Map<string, DocumentViewCard>): DocumentViewCard[] {
  return nodes.flatMap((node) => {
    if (node.type === "card") {
      const card = documentCardForPlacement(cardMap, node);
      return card ? [card] : [];
    }
    return cardsFromNodes(node.children, cardMap);
  });
}

export function documentCardForPlacement(
  cardMap: ReadonlyMap<string, DocumentViewCard>,
  node: Extract<DocumentViewNode, { type: "card" }>,
): DocumentViewCard | null {
  const card = cardMap.get(node.card_id);
  return card ? { ...card, layout: node.layout, region: node.layout.rail === "right" ? "sidebar" : "main" } : null;
}

/** Empty Or Filtered Branches Do Not Create Section Chrome. */
export function documentNodeHasCards(
  node: DocumentViewNode,
  cards: ReadonlyMap<string, DocumentViewCard>,
  filter: (card: DocumentViewCard) => boolean,
): boolean {
  if (node.type === "section")
    return Boolean(node.unavailable || node.loading) || node.children.some((child) => documentNodeHasCards(child, cards, filter));
  const card = documentCardForPlacement(cards, node);
  return card !== null && filter(card);
}
