/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The Parameters grid: an execution's materialized case values, the same on
 * the pending page, the running page and the frozen report.
 *
 * Every entry is a label, a figure and a unit on one column template. The
 * figure column is monospace and right-aligned, so digits stack; the unit sits
 * immediately right of its figure, dimmed; the label takes the remaining width
 * and wraps rather than clipping. Figure and unit widths are measured from the
 * rows in characters of the figure font, so both halves carry identical
 * columns. A second half appears only when a half still fits its longest
 * figure and unit beside a readable label; the halves share each row's
 * baseline. No table chrome: no header row, no rules.
 */

/* === External Imports === */

import type { CSSProperties } from "react";

/* === Local Imports === */

import { PathPill } from "@/components/PathPill";
import { isPathLike } from "@/lib/path-display";
import { type ExecutionParameterRow, executionParameterColumns } from "./execution-sections-model";

/* === Types === */

export type ExecutionParametersProps = {
  rows: ExecutionParameterRow[];
};

/* === Constants === */

/** The narrowest label a half keeps before the grid falls back to one half. */
const LABEL_MIN = "12rem";

/** The air between the two halves. */
const HALF_GAP = "2.5rem";

/* === Components === */

export function ExecutionParameters({ rows }: ExecutionParametersProps) {
  const columns = executionParameterColumns(rows);
  const style = {
    "--parameter-value": `${columns.value}ch`,
    "--parameter-unit": `${columns.unit}ch`,
    "--parameter-half": `calc(${LABEL_MIN} + ${columns.value}ch + ${columns.unit}ch + 2rem)`,
    gridTemplateColumns: `repeat(auto-fill, minmax(min(100%, max(var(--parameter-half), calc((100% - ${HALF_GAP}) / 2))), 1fr))`,
    columnGap: HALF_GAP,
  } as CSSProperties;

  return (
    <dl className="grid min-w-0 items-baseline gap-y-2 font-mono text-xs" data-testid="execution-parameters" style={style}>
      {rows.map((row) => (
        <div
          className="grid min-w-0 grid-cols-[minmax(0,1fr)_var(--parameter-value)_var(--parameter-unit)] items-baseline gap-x-2 leading-5"
          data-parameter={row.key}
          key={row.key}
        >
          <dt className="min-w-0 pr-3 font-sans text-muted-foreground [overflow-wrap:anywhere]" title={row.key}>
            {row.label}
          </dt>
          <dd className="min-w-0 text-right tabular-nums [overflow-wrap:anywhere]">
            {isPathLike(row.text, row.key) ? <PathPill align="right" compact copy value={row.text} /> : row.text}
          </dd>
          <dd className="text-muted-foreground">{row.unit ?? ""}</dd>
        </div>
      ))}
    </dl>
  );
}
