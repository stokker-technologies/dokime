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

import type { TestExecution } from "@/lib/types";
import { executionStatusLabel, isTerminalExecutionStatus, isUnstartedExecution } from "./cycle-workspace-model";

/** One lifecycle action of the execution's Control card, in card order. */
export type ExecutionAction = "Start" | "Cancel" | "Pause" | "Resume" | "Stop" | "Re-Run";

export type ExecutionPhase = "unstarted" | "queued" | "running" | "waiting" | "paused" | "stopping" | "finalizing" | "ended";

/** What the execution's Control card states and offers. */
export type ExecutionControlState = {
  phase: ExecutionPhase;
  /** The state line: what the node does now, the running step's own label. */
  line: string;
  /** Secondary text after the line, muted: the running step's id. */
  detail: string | null;
  actions: ExecutionAction[];
};

/** The facts the execution's card is decided from. */
export type ExecutionControlInput = {
  status: TestExecution["status"];
  controls: TestExecution["controls"];
  /** The live step progress, when the runtime streams one for this execution. */
  progress: { label: string; id: string; completed: number; total: number } | null;
  /** Whether a prompt of this execution waits for an answer. */
  promptWaiting: boolean;
  /** Whether this page's own Stop is in flight. */
  stopping: boolean;
};

/** The cycle's Control card. */
export type CycleControlState = {
  line: string;
  detail: string | null;
  /** Seal Cycle while the cycle is Active, nothing once it seals. */
  actions: "Seal Cycle"[];
};

export type CycleControlInput = {
  phase: "active" | "sealing" | "sealed";
  /** The member running now, with its step's label and id. */
  running: { ordinal: number; step: string | null; stepId: string | null } | null;
};

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
  if (isTerminalExecutionStatus(status) && !controls) {
    return { actions: ["Re-Run"], detail: null, line: `Ended · ${executionStatusLabel(status)}`, phase: "ended" };
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
    detail: progress && progress.id !== progress.label ? progress.id : null,
    line: progress?.label ?? "Running",
    phase: "running",
  };
}

/** The cycle's card: the member running now, or that none is, and Seal Cycle while Active. */
export function cycleControlState({ phase, running }: CycleControlInput): CycleControlState {
  if (phase === "sealing") return { actions: [], detail: null, line: "Sealing · saving evidence" };
  if (phase === "sealed") return { actions: [], detail: null, line: "Sealed · evidence saved" };
  if (!running) return { actions: ["Seal Cycle"], detail: null, line: "Idle · no execution running" };
  return {
    actions: ["Seal Cycle"],
    detail: running.stepId && running.stepId !== running.step ? running.stepId : null,
    line: running.step ? `Execution ${running.ordinal} · ${running.step}` : `Execution ${running.ordinal} running`,
  };
}

/** Where the cycle waits on an operator: "Input needed on #1, #3", or `null`. */
export function inputNeededLabel(ordinals: readonly number[]): string | null {
  return ordinals.length ? `Input needed on ${ordinals.map((ordinal) => `#${ordinal}`).join(", ")}` : null;
}
