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
 *
 * With `live={false}` the same component is the report's record table: the
 * level filter survives, because a threshold is a way of reading a file, and
 * everything that would suggest a running backend does not.
 */

/* === External Imports === */

/* === Local Imports === */
import type { HubRecord } from "@/lib/types";
import { useUiStore } from "@/stores/ui-store";
import { useConsoleRecords } from "./console-records";
import { FrozenConsole, LiveConsoleView, resolvedConsoleScope } from "./ConsoleDrawerView";

/* === Types === */
export type ConsoleDrawerProps =
  | {
      /** The cycle the route carries, or null away from a cycle. */
      cycleId: string | null;
      /** The attempt the operator selected, or null. */
      executionId: string | null;
      live?: true;
    }
  | {
      live: false;
      /** The attempt's records, already frozen into the file. */
      records: HubRecord[];
    };

/* === Components === */
export function ConsoleDrawer(props: ConsoleDrawerProps) {
  if (props.live === false) {
    return <FrozenConsole records={props.records} />;
  }
  return <ConnectedConsole cycleId={props.cycleId} executionId={props.executionId} />;
}

/* === Private Components === */
function ConnectedConsole({ cycleId, executionId }: { cycleId: string | null; executionId: string | null }) {
  const drawer = useUiStore((state) => state.console);
  const scope = resolvedConsoleScope(drawer.scope, cycleId, executionId);
  const streamed = useConsoleRecords({
    cycleId: scope === "instance" ? null : cycleId,
    enabled: drawer.open,
    executionId: scope === "execution" ? executionId : null,
    level: drawer.level,
  });
  return <LiveConsoleView cycleId={cycleId} executionId={executionId} streamed={streamed} />;
}
