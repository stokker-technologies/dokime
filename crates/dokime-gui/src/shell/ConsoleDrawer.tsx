/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The console: what the backend said, in the order it said it.
 *
 * A drawer above the status bar, spanning the main column, so the evidence
 * keeps its shape while the operator reads the tail. The rows are records —
 * the same projection the evidence pane renders as cards — shown as the dense
 * mono grid a log wants: time, level, source, message.
 */

/* === External Imports === */
import { useStore } from "zustand";

/* === Local Imports === */
import { useUiStore } from "@/stores/ui-store";
import { LiveConsoleView, resolvedConsoleScope } from "@/shell/ConsoleDrawerView";
import { consoleRecordsStore } from "./console-buffer";
import { useConsoleRecords } from "./console-records";

/* === Components === */
export function ConsoleDrawer({ cycleId, executionId }: { cycleId: string | null; executionId: string | null }) {
  const drawer = useUiStore((state) => state.console);
  const setConsoleState = useUiStore((state) => state.setConsoleState);
  const filter = useStore(consoleRecordsStore, (state) => state.filter);
  const setFilter = useStore(consoleRecordsStore, (state) => state.setFilter);
  const scope = resolvedConsoleScope(drawer.scope, cycleId, executionId);
  const streamed = useConsoleRecords({
    cycleId: scope === "instance" ? null : cycleId,
    enabled: drawer.open,
    executionId: scope === "execution" ? executionId : null,
    level: drawer.level,
  });
  return (
    <LiveConsoleView
      cycleId={cycleId}
      executionId={executionId}
      streamed={streamed}
      drawer={drawer}
      setConsoleState={setConsoleState}
      filter={filter}
      setFilter={setFilter}
    />
  );
}
