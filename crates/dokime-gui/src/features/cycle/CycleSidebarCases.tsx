/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The open cycle as a compact explorer tree: the cycle node and, under it,
 * its executions in cycle order.
 *
 * Every row is one line on one grid — chevron, status glyph, label, Run
 * Again, ordinal or progress — with the same column widths on every
 * row, so glyphs, ordinals and actions line up whatever the labels say. A row
 * is `--tree-row-height` tall (28px on a desk, 44px at touch). The label is
 * cut with an ellipsis and its full text is the row's tooltip and accessible
 * name; the status is a coloured glyph whose word is its tooltip and is also
 * read as text. Entry actions live in the page header.
 *
 * It is a WAI-ARIA tree: arrow keys move between rows, left and right collapse
 * and expand the cycle node, Enter selects. Selecting the cycle node selects
 * the cycle itself (`view: "cycle"`); a row selects its execution, or its case
 * while the runtime has none. The rows read the same query the workspace
 * reads, so the tree and the document never disagree. The running execution
 * carries a marker while Follow Active is following it.
 */

/* === External Imports === */

import { useNavigate } from "@tanstack/react-router";
import { useMemo } from "react";
import { CycleSidebarCasesView } from "@/features/cycle/CycleSidebarCasesView";
import { useCycleDetailQuery, useCyclePendingPromptsQuery, useRerunCase } from "./cycle-queries";
/* === Local Imports === */
import type { CycleSelection } from "@/features/cycle/cycle-workspace-model";
import { useSelectionStore } from "@/stores/selection-store";

/* === Types === */
export type CycleSidebarCasesProps = {
  cycleId: string;
  selection: CycleSelection;
};

/* === Components === */
export function CycleSidebarCases(props: CycleSidebarCasesProps) {
  const navigate = useNavigate();
  const followActive = useSelectionStore((state) => state.followActive);
  const detailQuery = useCycleDetailQuery(props.cycleId);
  const rerunCase = useRerunCase(props.cycleId);
  const prompts = useCyclePendingPromptsQuery(props.cycleId).data;
  const inputNeeded = useMemo(
    () => new Set((prompts ?? []).flatMap((prompt) => (prompt.scope?.executionId ? [prompt.scope.executionId] : []))),
    [prompts],
  );
  const select = (next: CycleSelection) => {
    void navigate({
      params: { cycleId: props.cycleId },
      replace: true,
      search: (current: Record<string, unknown>) => ({
        ...current,
        caseId: next.caseId,
        executionId: next.executionId,
        view: next.view,
      }),
      to: "/cycles/$cycleId",
    });
  };

  return (
    <CycleSidebarCasesView
      detailQuery={detailQuery}
      inputNeeded={inputNeeded}
      selection={props.selection}
      followActive={followActive}
      live
      onSelect={select}
      actions={{
        addCases: () => void navigate({ search: { appendTo: props.cycleId }, to: "/library" }),
        rerunCase: rerunCase.rerun,
      }}
    />
  );
}
