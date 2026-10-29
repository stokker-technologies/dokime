/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import type { ReactNode } from "react";

import { cn } from "@/lib/utils";

type DocumentGridProps = {
  children: ReactNode;
  className?: string;
};

export function DocumentGrid({ children, className }: DocumentGridProps) {
  return <div className={cn("document-grid grid min-w-0 grid-cols-1 gap-3 md:grid-cols-12", className)}>{children}</div>;
}
