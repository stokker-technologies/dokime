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
import { AppShellLayout, type FrozenShell } from "./AppShellLayout";
import { ConsoleDrawer } from "./ConsoleDrawer";
import { MenuBar } from "./MenuBar";
import { useRouteCycleScope } from "./route-scope";
import { Sidebar } from "./Sidebar";
import { StatusBar } from "./StatusBar";

/* === Types === */
export type AppShellProps = { children: ReactNode; live?: true } | { children: ReactNode; live: false; frozen: FrozenShell };

/* === Components === */
export function AppShell(props: AppShellProps) {
  useRuntimeStream();
  const frozen = props.live === false ? props.frozen : null;
  return (
    <AppShellLayout
      frozen={frozen}
      menuBar={frozen ? null : <MenuBar />}
      console={frozen ? null : <RoutedConsoleDrawer />}
      sidebar={
        frozen ? <Sidebar selection={frozen.selection} cycleId={frozen.cycleId} live={false} onSelect={frozen.onSelect} /> : <Sidebar />
      }
      statusBar={frozen ? <StatusBar frozen={frozen} live={false} /> : <StatusBar />}
    >
      {frozen ? null : <PluginStartNotifications />}
      {props.children}
    </AppShellLayout>
  );
}

/* === Private Components === */
function RoutedConsoleDrawer() {
  const { cycleId, executionId } = useRouteCycleScope();
  return <ConsoleDrawer cycleId={cycleId} executionId={executionId} />;
}
