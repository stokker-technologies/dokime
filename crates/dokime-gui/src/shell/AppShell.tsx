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
import { useRuntimeStream } from "@/app/live/use-runtime-stream";
import { PluginStartNotifications } from "@/features/settings/plugins/plugin-queries";
import { AppShellLayout } from "./AppShellLayout";
import { ConsoleDrawer } from "./ConsoleDrawer";
import { MenuBar } from "./MenuBar";
import { useRouteCycleScope } from "./route-scope";
import { Sidebar } from "./Sidebar";
import { StatusBar } from "./StatusBar";

/* === Types === */
export type AppShellProps = { children: ReactNode };

/* === Components === */
export function AppShell(props: AppShellProps) {
  useRuntimeStream();
  return (
    <AppShellLayout frozen={null} menuBar={<MenuBar />} console={<RoutedConsoleDrawer />} sidebar={<Sidebar />} statusBar={<StatusBar />}>
      <PluginStartNotifications />
      {props.children}
    </AppShellLayout>
  );
}

/* === Private Components === */
function RoutedConsoleDrawer() {
  const { cycleId, executionId } = useRouteCycleScope();
  return <ConsoleDrawer cycleId={cycleId} executionId={executionId} />;
}
