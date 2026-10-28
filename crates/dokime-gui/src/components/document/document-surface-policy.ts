/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import type { DocumentViewCard } from "@/lib/types";

const CYCLE_DOCUMENT_EXCLUDED_CARD_KINDS = new Set<string>(["notes", "traceability"]);

export function isDocumentSurfaceCard(card: DocumentViewCard): boolean {
  return card.region === "main";
}

export function isCycleDocumentSurfaceCard(card: DocumentViewCard): boolean {
  return isDocumentSurfaceCard(card) && !CYCLE_DOCUMENT_EXCLUDED_CARD_KINDS.has(card.kind);
}
