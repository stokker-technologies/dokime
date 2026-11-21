/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Local Imports === */

import type { DocumentViewCard } from "@dokime/plots/lib/types";

/* === Constants === */

/* === Public Functions === */

export function isDocumentSurfaceCard(card: DocumentViewCard): boolean {
  return card.region === "main";
}
