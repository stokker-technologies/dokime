/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * What the Control card says and offers, on the execution page and the cycle
 * page alike.
 *
 * The card states one line about where its node stands and offers the
 * lifecycle actions that state admits, always in the same order in the same
 * slot. An execution that has not started offers Start and Cancel; one that
 * runs offers Pause or Resume and Stop; one whose last step has completed is
 * finalizing — it saves its evidence and offers nothing until it ends; an
 * ended one offers Re-Run. Stop is the only destructive action. The cycle
 * states the member running now and offers Seal Cycle while it is Active; it
 * names the executions waiting on input and links to them, because their
 * prompts are answered there. Pure, so the mapping is testable away
 * from the queries.
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
import type { TestCycle, TestExecution } from "@/lib/types";
import type { RerunNotices } from "@/stores/rerun-notice-store";
import { executionStatusLabel, isTerminalExecutionStatus, isUnstartedExecution } from "./cycle-workspace-model";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/** One lifecycle action of the execution's Control card, in card order. */
export type ExecutionAction = "Start" | "Cancel" | "Pause" | "Resume" | "Stop" | "Re-Run";

export type ExecutionPhase = "unstarted" | "queued" | "running" | "waiting" | "paused" | "stopping" | "finalizing" | "ended";

/** What the execution's Control card states and offers. */
export type ExecutionControlState = {
  phase: ExecutionPhase;
  /** The state line: what the node does now, the running step's own label. */
  line: string;
  /** Secondary text after the line, muted: a failed report's cause or why the plugin is unavailable. */
  detail: string | null;
  actions: ExecutionAction[];
};

/** The facts the execution's card is decided from. */
export type ExecutionControlInput = {
  /** False only after the live catalog confirms that this template is absent. */
  pluginAvailable?: boolean;
  report?: TestExecution["report"];
  status: TestExecution["status"];
  controls: TestExecution["controls"];
  /** The live step progress, when the runtime streams one for this execution, with its step's label. */
  progress: { label: string; completed: number; total: number } | null;
  /** Whether a prompt of this execution waits for an answer. */
  promptWaiting: boolean;
  /** Whether this page's own Stop is in flight. */
  stopping: boolean;
};

/** The cycle's Control card. */
export type CycleControlState = {
  line: string;
  /** Seal Cycle while the cycle is Active, Retry Seal and Archive once its seal failed, nothing once it seals. */
  actions: CycleAction[];
};

export type CycleAction = "Seal Cycle" | "Retry Seal" | "Archive";

export type CycleControlInput = {
  phase: TestCycle["phase"];
  /** The member running now, with its step's label. */
  running: { ordinal: number; step: string | null } | null;
};

/* === Classes === */

/* === Public Functions === */

/**
 * The runtime owns finalization. Completed records cover only finished steps,
 * and progress counts one step's work; neither establishes that the run ended.
 */
export function executionControlState(input: ExecutionControlInput): ExecutionControlState {
  const { controls, progress, status } = input;
  if (isUnstartedExecution(status)) {
    return status === "created"
      ? { actions: ["Start", "Cancel"], detail: null, line: "Queued · waits for the execution lane", phase: "queued" }
      : { actions: ["Start", "Cancel"], detail: null, line: "Not started · waits for Start", phase: "unstarted" };
  }
  if (isTerminalExecutionStatus(status) && (!controls || input.report?.state === "failed" || input.report?.state === "ready")) {
    if (input.pluginAvailable === false) {
      return {
        actions: [],
        detail:
          "Its plugin is unavailable on this backend. Saved evidence remains readable; start the Python backend that registered it or install its original plugin revision to run it again.",
        line: `Ended · ${executionStatusLabel(status)} · Plugin Unavailable`,
        phase: "ended",
      };
    }
    return {
      actions: ["Re-Run"],
      detail: input.report?.state === "failed" ? input.report.failure : null,
      line: `Ended · ${executionStatusLabel(status)}${input.report?.state === "failed" ? " · Report Failed" : ""}`,
      phase: "ended",
    };
  }
  if (status === "finalizing" || isTerminalExecutionStatus(status)) {
    return { actions: [], detail: null, line: "Finalizing · saving evidence", phase: "finalizing" };
  }
  if (input.stopping || controls?.stopRequested) {
    return { actions: [], detail: null, line: "Stopping · saving evidence", phase: "stopping" };
  }
  const stop: ExecutionAction[] = controls?.canStop ? ["Stop"] : [];
  if (controls?.paused) {
    const resume: ExecutionAction[] = controls.canResume ? ["Resume"] : [];
    return { actions: [...resume, ...stop], detail: null, line: "Paused", phase: "paused" };
  }
  const pause: ExecutionAction[] = controls?.requestId && controls.canPause ? ["Pause"] : [];
  if (input.promptWaiting) {
    return { actions: [...pause, ...stop], detail: null, line: "Waits on the prompt below", phase: "waiting" };
  }
  return {
    actions: [...pause, ...stop],
    detail: null,
    line: progress?.label || "Running",
    phase: "running",
  };
}

/**
 * The cycle's card: the member running now, or that none is, and Seal Cycle
 * while Active. A failed seal offers Retry Seal and Archive; its evidence
 * stays saved either way, and the card states the cause beneath.
 */
export function cycleControlState({ phase, running }: CycleControlInput): CycleControlState {
  if (phase === "sealing") return { actions: [], line: "Sealing · saving evidence" };
  if (phase === "seal_failed") return { actions: ["Retry Seal", "Archive"], line: "Seal failed · evidence kept" };
  if (phase === "sealed") return { actions: [], line: "Sealed · evidence saved" };
  if (!running) return { actions: ["Seal Cycle"], line: "Idle · no execution running" };
  return {
    actions: ["Seal Cycle"],
    line: running.step ? `Execution ${running.ordinal} · ${running.step}` : `Execution ${running.ordinal} running`,
  };
}

/**
 * What Rerun Cycle left out of the cycle it created, one line per reason:
 * "2 cases skipped: a, b, its suite no longer declares this case", then the
 * parameters it dropped from cases it did create.
 */
export function rerunNoticeLines({ skipped, droppedParameters }: RerunNotices): string[] {
  const reasons = new Map<string, string[]>();
  for (const notice of skipped) reasons.set(notice.reason, [...(reasons.get(notice.reason) ?? []), notice.caseId]);
  const lines = [...reasons].map(
    ([reason, ids]) => `${ids.length} ${ids.length === 1 ? "case" : "cases"} skipped: ${ids.join(", ")}, ${reason}`,
  );
  for (const notice of droppedParameters) lines.push(`Parameters dropped from ${notice.caseId}: ${notice.reason}`);
  return lines;
}

/** Where the cycle waits on an operator: "Input needed on #1, #3", or `null`. */
export function inputNeededLabel(ordinals: readonly number[]): string | null {
  return ordinals.length ? `Input needed on ${ordinals.map((ordinal) => `#${ordinal}`).join(", ")}` : null;
}

/* === Private Functions === */

/* === Entry Point === */
