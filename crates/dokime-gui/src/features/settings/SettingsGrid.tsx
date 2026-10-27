/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* One Full-Width Content Column For Every Settings Section. */

/* === External Imports === */

import type { ReactNode } from "react";

/* === Types === */

type SettingsGridProps = {
  children: ReactNode;
};

/* === Components === */

export function SettingsGrid({ children }: SettingsGridProps) {
  return <div className="grid w-full min-w-0 grid-cols-12 content-start gap-4 [&>*]:col-span-12">{children}</div>;
}
