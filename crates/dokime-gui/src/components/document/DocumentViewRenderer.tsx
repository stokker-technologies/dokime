/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { createContext, type ReactNode, useContext } from "react";

/* === Local Imports === */

import { DocumentGrid } from "@/components/document/Document";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import {
  documentCardForPlacement,
  documentCardLayoutAttributes,
  documentCardLayoutStyle,
  documentNodeHasCards,
} from "@/lib/document-layout";
import type {
  DocumentViewCard,
  DocumentViewNode,
  DocumentViewPayload,
  DocumentViewSection,
  Metadata,
} from "@/lib/types";
import { cn } from "@/lib/utils";

/* === Constants === */

/**
 * A control a surface places in one section's header, by section id — the
 * live plot window beside the plots it scales. Absent, headers are plain.
 */
export const DocumentSectionActionContext = createContext<(sectionId: string) => ReactNode>(
  () => null,
);

/* === Types === */

/** Renders one card; `section` is the enclosing section, absent at the top level. */
export type DocumentCardRenderer = (card: DocumentViewCard, section: Metadata | null) => ReactNode;

/** An optional control beside a card's title; `null` leaves the header as it is. */
export type DocumentCardActionRenderer = (card: DocumentViewCard) => ReactNode;

type DocumentViewRendererProps = {
  document: DocumentViewPayload | null | undefined;
  renderCard: DocumentCardRenderer;
  cardAction?: DocumentCardActionRenderer;
  cardFilter?: (card: DocumentViewCard) => boolean;
  cardChrome?: (card: DocumentViewCard) => DocumentBlockChrome;
  emptyMessage?: ReactNode;
  prependSections?: ReactNode;
  sectionChrome?: DocumentSectionChrome;
};

type DocumentBlockChrome = "normal" | "embedded" | "flat";
type DocumentSectionChrome = "plain" | "grouped" | "card";

type DocumentViewBlockProps = {
  action?: ReactNode;
  card: DocumentViewCard;
  children: ReactNode;
  chrome?: DocumentBlockChrome;
  contentClassName?: string;
};

/* === Components === */

export function DocumentViewRenderer({
  document,
  cardAction,
  cardChrome = defaultCardChrome,
  renderCard,
  cardFilter = () => true,
  emptyMessage = "This document does not currently expose any cards.",
  prependSections,
  sectionChrome = "plain",
}: DocumentViewRendererProps) {
  const resolvedCardChrome = (card: DocumentViewCard): DocumentBlockChrome => {
    const chrome = cardChrome(card);
    return sectionChrome === "card" && chrome === "normal" ? "embedded" : chrome;
  };
  const cardMap = new Map(document?.cards.map((card) => [card.card_id, card]) ?? []);
  const layoutNodes = (document?.layout.nodes ?? []).filter((node) =>
    documentNodeHasCards(node, cardMap, cardFilter),
  );
  if (!document) {
    const notice = (
      <p className="text-sm text-muted-foreground">
        No document has been resolved for the selected scope yet.
      </p>
    );
    return notice;
  }

  return (
    <div className="grid min-w-0 gap-6">
      {prependSections}
      {layoutNodes.length ? (
        layoutNodes.map((node) => (
          <DocumentLayoutNodeBlock
            cardAction={cardAction}
            cardFilter={cardFilter}
            cardMap={cardMap}
            chrome={sectionChrome}
            key={layoutNodeKey(node)}
            node={node}
            renderCard={renderCard}
            resolvedCardChrome={resolvedCardChrome}
            section={null}
          />
        ))
      ) : (
        <div className="grid min-h-48 place-items-center gap-2 rounded-lg border border-dashed bg-muted/15 px-6 py-8 text-center">
          <strong className="text-base font-semibold">No content in this document yet</strong>
          <p className="max-w-2xl text-sm text-muted-foreground">{emptyMessage}</p>
        </div>
      )}
    </div>
  );
}

function DocumentLayoutNodeBlock({
  cardAction,
  cardFilter,
  cardMap,
  chrome,
  node,
  renderCard,
  resolvedCardChrome,
  section: parent,
}: {
  cardAction?: DocumentCardActionRenderer;
  cardFilter: (card: DocumentViewCard) => boolean;
  cardMap: Map<string, DocumentViewCard>;
  chrome: DocumentSectionChrome;
  node: DocumentViewNode;
  renderCard: DocumentCardRenderer;
  resolvedCardChrome: (card: DocumentViewCard) => DocumentBlockChrome;
  section: Metadata | null;
}) {
  if (node.type === "card") {
    const card = documentCardForPlacement(cardMap, node);
    if (!card || !cardFilter(card)) {
      return null;
    }

    return (
      <DocumentViewBlock
        action={cardAction?.(card)}
        card={card}
        chrome={resolvedCardChrome(card)}
        key={card.card_id}
      >
        {renderCard(card, parent)}
      </DocumentViewBlock>
    );
  }

  // A section whose only child is a card of the same name is the same
  // introduction twice; the card speaks for itself.
  const onlyChild = node.children.length === 1 ? node.children[0] : null;
  if (onlyChild?.type === "card") {
    const card = documentCardForPlacement(cardMap, onlyChild);
    if (card && sameLabel(node.metadata.label, card.title)) {
      return cardFilter(card) ? (
        <DocumentViewBlock
          action={cardAction?.(card) ?? <SectionAction sectionId={node.metadata.identifier} />}
          card={card}
          chrome={resolvedCardChrome(card)}
          key={card.card_id}
        >
          {renderCard(card, node.metadata)}
        </DocumentViewBlock>
      ) : null;
    }
  }

  const renderedChildren = node.children
    .filter((child) => documentNodeHasCards(child, cardMap, cardFilter))
    .map((child) => (
      <DocumentLayoutNodeBlock
        cardAction={cardAction}
        cardFilter={cardFilter}
        cardMap={cardMap}
        chrome={chrome}
        key={layoutNodeKey(child)}
        node={child}
        renderCard={renderCard}
        resolvedCardChrome={resolvedCardChrome}
        section={node.metadata}
      />
    ));
  if (!renderedChildren.length && !node.unavailable) {
    return null;
  }

  const section = {
    metadata: node.metadata,
    cards: [],
  };
  return (
    <DocumentSectionBlock chrome={sectionChromeFor(section, chrome)} section={section}>
      {node.unavailable ? (
        <p role="status" className="text-sm text-muted-foreground">
          <strong>Unavailable.</strong> {node.unavailable}
        </p>
      ) : (
        <DocumentGrid>{renderedChildren}</DocumentGrid>
      )}
    </DocumentSectionBlock>
  );
}

export function DocumentViewBlock({
  action,
  card,
  children,
  chrome = "normal",
  contentClassName,
}: DocumentViewBlockProps) {
  const showHeader = chrome !== "flat";
  return (
    <section
      className={cn(
        "document-card grid min-w-0",
        chrome === "normal" && "gap-3 rounded-md border bg-background/60 p-3",
        chrome === "embedded" && "gap-3",
        chrome === "flat" && "gap-2",
      )}
      {...documentCardLayoutAttributes(card.layout)}
      data-density={card.layout.density}
      style={documentCardLayoutStyle(card.layout)}
    >
      {showHeader ? (
        <header className={cn("grid min-w-0 gap-0.5", chrome === "embedded" && "px-0.5")}>
          {/* An eyebrow that only repeats the title below it introduces the
              same card twice. */}
          {sameLabel(documentCardEyebrow(card), card.title) ? null : (
            <p className="text-xs font-bold uppercase text-muted-foreground">
              {documentCardEyebrow(card)}
            </p>
          )}
          {action ? (
            /* The control shares the title's row and keeps its height, so a
               card with an action lines up with one without. */
            <div className="grid min-w-0 grid-cols-[minmax(0,1fr)_auto] items-center gap-2">
              <h5 className="truncate text-sm font-semibold">{card.title}</h5>
              <div className="-my-0.5 flex items-center">{action}</div>
            </div>
          ) : (
            <h5 className="truncate text-sm font-semibold">{card.title}</h5>
          )}
          {card.metadata?.description ? (
            <p className="text-sm text-muted-foreground [overflow-wrap:anywhere]">
              {card.metadata.description}
            </p>
          ) : null}
        </header>
      ) : null}
      <div className={cn("grid min-w-0 gap-3", contentClassName)}>{children}</div>
    </section>
  );
}

/** The control the surface placed on this section, if any. */
function SectionAction({ sectionId }: { sectionId: string }) {
  const action = useContext(DocumentSectionActionContext)(sectionId);
  return action ? <div className="flex min-w-0 items-center">{action}</div> : null;
}

function DocumentSectionBlock({
  chrome,
  children,
  section,
}: {
  chrome: DocumentSectionChrome;
  children: ReactNode;
  section: DocumentViewSection;
}) {
  if (chrome === "card") {
    return (
      <Card className="rounded-lg bg-muted/20 py-0">
        <CardHeader className="border-b px-4 py-3">
          <div className="grid min-w-0 grid-cols-[minmax(0,1fr)_auto] items-center gap-2">
            <CardTitle className="text-base">{section.metadata.label}</CardTitle>
            <SectionAction sectionId={section.metadata.identifier} />
          </div>
          {section.metadata.description ? (
            <p className="text-sm text-muted-foreground [overflow-wrap:anywhere]">
              {section.metadata.description}
            </p>
          ) : null}
        </CardHeader>
        <CardContent className="grid gap-4 p-4">{children}</CardContent>
      </Card>
    );
  }

  if (chrome === "grouped") {
    return (
      <section
        className="grid min-w-0 gap-4 rounded-md border border-border/70 border-l-4 border-l-primary/70 bg-muted/15 px-4 py-3"
        data-section-id={section.metadata.identifier}
      >
        <header className="grid min-w-0 gap-0.5 border-b border-border/60 pb-2">
          <div className="grid min-w-0 grid-cols-[minmax(0,1fr)_auto] items-center gap-2">
            <h4 className="truncate text-sm font-semibold">{section.metadata.label}</h4>
            <SectionAction sectionId={section.metadata.identifier} />
          </div>
          {section.metadata.description ? (
            <p className="text-sm text-muted-foreground [overflow-wrap:anywhere]">
              {section.metadata.description}
            </p>
          ) : null}
        </header>
        {children}
      </section>
    );
  }

  return (
    <section className="grid min-w-0 gap-3">
      <header className="grid min-w-0 gap-0.5">
        <div className="grid min-w-0 grid-cols-[minmax(0,1fr)_auto] items-center gap-2">
          <h4 className="truncate text-sm font-semibold">{section.metadata.label}</h4>
          <SectionAction sectionId={section.metadata.identifier} />
        </div>
        {section.metadata.description ? (
          <p className="text-sm text-muted-foreground [overflow-wrap:anywhere]">
            {section.metadata.description}
          </p>
        ) : null}
      </header>
      {children}
    </section>
  );
}

function sectionChromeFor(
  section: DocumentViewSection,
  defaultChrome: DocumentSectionChrome,
): DocumentSectionChrome {
  if (defaultChrome !== "plain") {
    return defaultChrome;
  }
  return section.metadata.tags?.includes("chrome:grouped") ? "grouped" : "plain";
}

function defaultCardChrome(card: DocumentViewCard): DocumentBlockChrome {
  return ["plots", "histograms", "box_summaries"].includes(card.kind) ? "flat" : "normal";
}

/** Do two labels say the same thing, ignoring case and word separators? */
function sameLabel(left: string, right: string): boolean {
  const normalize = (value: string) =>
    value
      .toLowerCase()
      .replaceAll(/[\s_-]+/g, " ")
      .trim();
  return normalize(left) === normalize(right);
}

function documentCardEyebrow(card: DocumentViewCard): string {
  if (card.kind === "box_summaries") {
    return "Distribution";
  }
  if (card.kind === "telemetry_context") {
    return "Telemetry";
  }
  return card.kind.replaceAll("_", " ");
}

function layoutNodeKey(node: DocumentViewNode): string {
  return node.type === "card" ? `card:${node.card_id}` : `section:${node.node_id}`;
}
