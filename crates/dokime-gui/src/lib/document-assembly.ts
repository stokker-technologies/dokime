/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Shared Declaration And Binding Assembly
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
import { documentView, unavailableReason } from "./document-presentation";
import type { DocumentViewCard, DocumentViewNode, DocumentViewPayload, Metadata } from "./types";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */
export type DeclaredNode =
  | { type: "section"; node_id: string; repeat: boolean; children: DeclaredNode[] }
  | Extract<DocumentViewNode, { type: "card" }>;

export interface CaptureAssembly {
  document_id: string;
  metadata: Metadata;
  sources: readonly string[];
  bindings: readonly { source_id: string; card: DocumentViewCard }[];
  nodes: DeclaredNode[];
  repeats: ReadonlyMap<string, readonly { suffix: string; node_id: string }[]>;
  sections: ReadonlyMap<string, { metadata: Metadata; unavailable?: string | null }>;
}

/* === Classes === */

/* === Public Functions === */
export function assembleCapture(capture: CaptureAssembly): DocumentViewPayload {
  const sources = new Set(capture.sources);
  if (sources.size !== capture.sources.length || sources.size > 4096 || capture.bindings.length > 4096) {
    throw new Error("Captured document sources or bindings exceed their contract");
  }
  const cards = new Map<string, { source_id: string; card: DocumentViewCard }>();
  for (const binding of capture.bindings) {
    if (!sources.has(binding.source_id) || cards.has(binding.card.card_id)) {
      throw new Error("Captured card binding has an invalid source or duplicate identity");
    }
    cards.set(binding.card.card_id, binding);
  }
  const usedCards = new Set<string>();
  const usedRepeats = new Set<string>();
  const usedSections = new Set<string>();
  let count = 0;
  let work = 0;
  function assemble(nodes: DeclaredNode[], suffixes: string[], depth: number): DocumentViewNode[] {
    if (depth > 32 && nodes.length > 0) throw new Error("Captured document depth exceeds its budget");
    const result: DocumentViewNode[] = [];
    const suffix = suffixes.map((part) => `.${part}`).join("");
    for (const node of nodes) {
      if (++work > 4096 * 32) throw new Error("Captured document assembly exceeds its budget");
      if (node.type === "card") {
        const id = node.card_id + suffix;
        const binding = cards.get(id);
        if (!binding || binding.source_id !== node.card_id) throw new Error(`Captured card binding is missing or mismatched: ${id}`);
        usedCards.add(id);
        if (++count > 4096) throw new Error("Captured document expansion exceeds its budget");
        result.push({ ...node, card_id: id });
        continue;
      }
      const id = node.node_id + suffix;
      const instances = node.repeat ? capture.repeats.get(id) : [{ node_id: id, suffix: null }];
      if (!instances) throw new Error(`Captured repeat selection is missing: ${id}`);
      if (node.repeat) {
        if (usedRepeats.has(id)) throw new Error(`Captured repeat identity is duplicated: ${id}`);
        usedRepeats.add(id);
      }
      const failedSection = capture.sections.get(id);
      const failed = unavailableReason(failedSection?.unavailable);
      if (failed) {
        if (!failedSection?.metadata || !node.repeat || instances.length !== 0 || usedSections.has(id)) {
          throw new Error("Unavailable repeat has instances or duplicate identity");
        }
        usedSections.add(id);
        if (++count > 4096) throw new Error("Captured document expansion exceeds its budget");
        result.push({
          type: "section",
          node_id: id,
          metadata: failedSection.metadata,
          unavailable: failed,
          children: [],
        });
        continue;
      }
      const suffixesSeen = new Set<string>();
      for (const instance of instances) {
        if (++work > 4096 * 32) throw new Error("Captured document assembly exceeds its budget");
        if (instance.suffix !== null) {
          if (!instance.suffix || suffixesSeen.has(instance.suffix)) throw new Error("Captured repeat suffix is empty or duplicated");
          suffixesSeen.add(instance.suffix);
        }
        const children = assemble(node.children, instance.suffix === null ? suffixes : [...suffixes, instance.suffix], depth + 1);
        if (children.length === 0) continue;
        const section = capture.sections.get(instance.node_id);
        if (!section?.metadata || unavailableReason(section.unavailable) || usedSections.has(instance.node_id))
          throw new Error(`Captured section identity is missing or duplicated: ${instance.node_id}`);
        usedSections.add(instance.node_id);
        if (++count > 4096) throw new Error("Captured document expansion exceeds its budget");
        result.push({
          type: "section",
          node_id: instance.node_id,
          metadata: section.metadata,
          children,
        });
      }
    }
    return result;
  }
  const layout = { nodes: assemble(capture.nodes, [], 1) };
  if (usedCards.size !== cards.size || usedRepeats.size !== capture.repeats.size || usedSections.size !== capture.sections.size) {
    throw new Error("Captured document contains unreferenced binding or section identities");
  }
  return documentView(
    capture.document_id,
    capture.metadata,
    [...cards.values()].map((binding) => binding.card),
    layout,
  );
}

/* === Private Functions === */

/* === Entry Point === */
