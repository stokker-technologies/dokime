/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import type { ReactNode } from "react";

/* === Local Imports === */

import { cn } from "@/lib/utils";

/* === Types === */

type DocumentGridProps = {
  children: ReactNode;
  className?: string;
};

/* === Components === */

export function DocumentGrid({ children, className }: DocumentGridProps) {
  return <div className={cn("document-grid grid min-w-0 grid-cols-1 gap-3 md:grid-cols-12", className)}>{children}</div>;
}
