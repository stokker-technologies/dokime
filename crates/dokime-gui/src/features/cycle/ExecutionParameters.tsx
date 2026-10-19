/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The Parameters section's grid: the execution's materialized case values.
 *
 * Key, value and unit on three shared columns, repeated twice side by side on
 * a wide viewport. Every entry is laid on the same column template rather than
 * a box of its own, so keys, values and units line up down the page and share
 * a baseline across a row. No table chrome: no header row, no rules.
 */

/* === Local Imports === */

import { ValueDisplay } from "@/components/ValueDisplay";
import type { ExecutionParameterRow } from "./execution-sections-model";

/* === Types === */

export type ExecutionParametersProps = {
  rows: ExecutionParameterRow[];
};

/* === Components === */

export function ExecutionParameters({ rows }: ExecutionParametersProps) {
  return (
    <dl className="grid min-w-0 grid-cols-[minmax(0,auto)_minmax(0,1fr)_auto] items-baseline gap-x-3 gap-y-1.5 text-sm lg:grid-cols-[repeat(2,minmax(0,auto)_minmax(0,1fr)_auto)] lg:gap-x-4">
      {rows.map((row) => (
        <div className="contents" key={row.key}>
          <dt className="truncate text-xs text-muted-foreground" title={row.key}>
            {row.label}
          </dt>
          <dd className="min-w-0 font-mono text-xs leading-5 [overflow-wrap:anywhere]">
            <ValueDisplay compactPath label={row.key} value={row.value} />
          </dd>
          <dd className="text-xs text-muted-foreground">{row.unit ?? ""}</dd>
        </div>
      ))}
    </dl>
  );
}
