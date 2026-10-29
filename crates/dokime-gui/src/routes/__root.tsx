/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import { createRootRoute, Outlet } from "@tanstack/react-router";

import { AppProviders } from "@/app/AppProviders";
import { AppShell } from "@/shell/AppShell";
import { ThemeController } from "@/shell/ThemeController";

export const Route = createRootRoute({
  component: RootRoute,
});

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
