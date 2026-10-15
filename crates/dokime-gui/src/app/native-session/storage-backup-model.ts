/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/*
 * The startup failure screen's way out of storage this build cannot open.
 *
 * When the owned local backend fails to start, the screen offers to archive
 * its data directory into a zip and then reset and restart it. The shell
 * reports stopping, archiving with the bytes written, resetting and
 * restarting; a failure after the archive was written still names the kept
 * archive. The module has no runtime imports, so a node test can load it
 * directly.
 */

/* === Types === */

export type BackupResetContext = {
  /** Whether this desktop can start a local backend of its own. */
  localBackendAvailable: boolean;
  /** Whether the last error is a failed start of that owned backend. */
  failedStart: boolean;
  /** Whether a selection or reset is already in flight. */
  busy: boolean;
};

export type BackupStep = {
  phase: string;
  label: string;
  state: "done" | "active" | "waiting";
  /** The archive bytes written so far, on the archiving step. */
  detail: string | null;
};

/** A failure after the backup archive was written: the kept archive and the
 * reset or restart error that followed it. */
export type BackupFailure = {
  archive: string;
  error: unknown;
};

/* === Public Constants === */

export const BACKUP_RESET_LABEL = "Back Up And Reset Local Storage";

/** The steps a backup reset reports, in order. Every backend startup phase
 * after them belongs to the last step. */
export const BACKUP_RESET_PHASES = ["stopping", "archiving", "deleting", "starting"] as const;

/* === Public Functions === */

/** Whether the failure screen offers the backup reset at all. */
export function backupResetAvailable(context: BackupResetContext): boolean {
  return context.localBackendAvailable && context.failedStart && !context.busy;
}

/** The backup reset's steps for the phase the shell last reported. */
export function backupSteps(phase: string | null, archivedBytes: number | null): BackupStep[] {
  const phases: readonly string[] = BACKUP_RESET_PHASES;
  const index = phase === null ? 0 : phases.includes(phase) ? phases.indexOf(phase) : 3;
  return BACKUP_RESET_PHASES.map((name, position) => ({
    phase: name,
    label: stepLabel(name),
    state: position < index ? "done" : position === index ? "active" : "waiting",
    detail:
      name === "archiving" && position <= index && archivedBytes !== null
        ? `${formatBytes(archivedBytes)} written`
        : null,
  }));
}

/** A byte count in the largest binary unit that keeps it at or above one. */
export function formatBytes(bytes: number): string {
  const units = ["B", "KiB", "MiB", "GiB", "TiB"];
  let value = bytes;
  let unit = 0;
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024;
    unit += 1;
  }
  return unit === 0 ? `${value} B` : `${value.toFixed(1)} ${units[unit]}`;
}

/** The kept archive of a backup reset that failed after archiving, or `null`
 * for every other error. */
export function backupFailure(error: unknown): BackupFailure | null {
  if (!error || typeof error !== "object" || !("kind" in error) || error.kind !== "archived")
    return null;
  if (!("archive" in error) || typeof error.archive !== "string") return null;
  return { archive: error.archive, error: "error" in error ? error.error : undefined };
}

/* === Private Functions === */

function stepLabel(phase: (typeof BACKUP_RESET_PHASES)[number]): string {
  switch (phase) {
    case "stopping":
      return "Stopping Backend";
    case "archiving":
      return "Archiving Local Storage";
    case "deleting":
      return "Resetting Local Storage";
    case "starting":
      return "Restarting Backend";
  }
}
