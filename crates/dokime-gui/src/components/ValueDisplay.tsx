/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Local Imports === */

import { PathPill } from "@/components/PathPill";
import { formatValue } from "@/lib/dashboard";
import { isPathLike } from "@/lib/path-display";

/* === Types === */

type ValueDisplayProps = {
  value: unknown;
  label?: string;
  unit?: string | null;
  align?: "left" | "right";
  copyPath?: boolean;
  compactPath?: boolean;
};

/* === Components === */

export function ValueDisplay({ value, label = "", unit, align = "left", copyPath = true, compactPath = false }: ValueDisplayProps) {
  const formatted = formatValue(value, unit, label);
  if (isPathLike(formatted, label)) {
    return <PathPill align={align} compact={compactPath} copy={copyPath} value={formatted} />;
  }
  return <>{formatted}</>;
}
