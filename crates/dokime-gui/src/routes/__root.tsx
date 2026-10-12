/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { createRootRoute, Outlet } from "@tanstack/react-router";

/* === Local Imports === */

import { AppProviders } from "@/app/AppProviders";
import { AppShell } from "@/shell/AppShell";
import { ThemeController } from "@/shell/ThemeController";

/* === Route === */

export const Route = createRootRoute({
  component: RootRoute,
});

/* === Components === */

function RootRoute() {
  return (
    <AppProviders>
      <ThemeController />
      <AppShell>
        <Outlet />
      </AppShell>
    </AppProviders>
  );
}
