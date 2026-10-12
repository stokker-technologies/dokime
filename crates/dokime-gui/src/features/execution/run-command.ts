/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Types === */

export type RunCommandState =
  | { kind: "start"; runtimeReady: boolean }
  | { kind: "running"; canRestart: boolean; paused: boolean; runtimeReady: boolean }
  | { kind: "stopping"; runtimeReady: boolean }
  | { kind: "rerun"; runtimeReady: boolean }
  | { kind: "none" };

type RunCommandInput = {
  selectedRuntimeReady: boolean;
  showQueuedStart: boolean;
  showRerun: boolean;
  showResume: boolean;
  showRunningStop: boolean;
  showStopping: boolean;
};

/* === Public Functions === */

export function buildRunCommandState({
  selectedRuntimeReady,
  showQueuedStart,
  showRerun,
  showResume,
  showRunningStop,
  showStopping,
}: RunCommandInput): RunCommandState {
  if (showQueuedStart) {
    return { kind: "start", runtimeReady: selectedRuntimeReady };
  }
  if (showStopping) {
    return { kind: "stopping", runtimeReady: selectedRuntimeReady };
  }
  if (showRunningStop || showResume) {
    return {
      kind: "running",
      canRestart: showRerun || showRunningStop || showResume,
      paused: showResume,
      runtimeReady: selectedRuntimeReady,
    };
  }
  if (showRerun) {
    return { kind: "rerun", runtimeReady: selectedRuntimeReady };
  }
  return { kind: "none" };
}
