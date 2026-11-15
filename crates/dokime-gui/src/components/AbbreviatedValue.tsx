/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Local Imports === */

import { formatCompactDecimal } from "@/lib/format";

/* === Types === */

type AbbreviatedValueProps = {
  /** The complete value, shown on hover and copied. */
  exact: string;
  /** The shown text; abbreviated from `exact` when absent. */
  value?: string;
  className?: string;
};

/* === Components === */

/** A value that may be shown abbreviated while its title and copy keep every digit. */
export function AbbreviatedValue({ exact, value = formatCompactDecimal(exact), className }: AbbreviatedValueProps) {
  return (
    <span
      className={className}
      data-exact-value={exact}
      title={exact}
      onCopy={
        exact === value
          ? undefined
          : (event) => {
              event.clipboardData.setData("text/plain", exact);
              event.preventDefault();
            }
      }
    >
      {value}
    </span>
  );
}
