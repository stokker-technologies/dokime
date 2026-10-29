/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import type { DocumentViewCard } from "@/lib/types";

export function isDocumentSurfaceCard(card: DocumentViewCard): boolean {
  return card.region === "main";
}
