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
 * Again, ordinal or progress, menu — with the same column widths on every
 * row, so glyphs, ordinals and actions line up whatever the labels say. A row
 * is `--tree-row-height` tall (28px on a desk, 44px at touch). The label is
 * cut with an ellipsis and its full text is the row's tooltip and accessible
 * name; the status is a coloured glyph whose word is its tooltip and is also
 * read as text. The row menu appears on hover or focus, and on a long press at
 * touch.
 *
 * It is a WAI-ARIA tree: arrow keys move between rows, left and right collapse
 * and expand the cycle node, Enter selects. Selecting the cycle node selects
 * the cycle itself (`view: "cycle"`); a row selects its execution, or its case
 * while the runtime has none. The rows read the same query the workspace
 * reads, so the tree and the document never disagree. The running execution
 * carries a marker while Follow Active is following it.
 *
 * With `live={false}` the same tree is the frozen report's outline: the same
 * grid and glyphs, no menus and no commands, and selection is handed back to
 * the report rather than written to a URL it does not own.
 */

import { useMemo } from "react";
import { CycleSidebarCasesView } from "./CycleSidebarCasesView";
import { useCycleCommands, useCycleDetailQuery, useCyclePendingPromptsQuery, useRerunCase } from "./cycle-queries";

import type { CycleSelection } from "./cycle-workspace-model";

export type CycleSidebarCasesProps = {
  cycleId: string;
  selection: CycleSelection;
} & ({ live?: true; onSelect?: undefined } | { live: false; onSelect: (selection: CycleSelection) => void });

export function CycleSidebarCases(props: CycleSidebarCasesProps) {
  const detailQuery = useCycleDetailQuery(props.cycleId);
  const commands = useCycleCommands(props.cycleId);
  const rerunCase = useRerunCase(props.cycleId);
  const prompts = useCyclePendingPromptsQuery(props.cycleId).data;
  const inputNeeded = useMemo(
    () => new Set((prompts ?? []).flatMap((prompt) => (prompt.scope?.test_execution_id ? [prompt.scope.test_execution_id] : []))),
    [prompts],
  );
  return (
    <CycleSidebarCasesView
      {...props}
      detailQuery={detailQuery}
      inputNeeded={inputNeeded}
      actions={{
        rerunCase: rerunCase.rerun,
        startExecution: (id) => commands.startExecution.mutateAsync(id),
        deleteCase: (id) => commands.deleteCase.mutateAsync(id),
      }}
    />
  );
}
