/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* One Full-Width Content Column For Every Settings Section. */

import type { ReactNode } from "react";

type SettingsGridProps = {
  children: ReactNode;
};

export function SettingsGrid({ children }: SettingsGridProps) {
  return <div className="grid w-full min-w-0 grid-cols-12 content-start gap-4 [&>*]:col-span-12">{children}</div>;
}
