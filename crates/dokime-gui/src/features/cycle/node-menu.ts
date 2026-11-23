// Dokime — SPDX-License-Identifier: Apache-2.0
/** Archive or Unarchive, as the node's menu offers it. */
export type ArchiveAction = {
  archived: boolean;
  onToggle: () => void;
  /** Why the node cannot be archived yet; archiving waits for a terminal run. */
  refusal?: string;
};

/** The cycle page's "…" menu. */
export type CycleMenu = {
  kind: "cycle";
  archive: ArchiveAction;
  followActive: boolean;
  onAddCases: () => void;
  /** Plans a new cycle with this one's cases against the installed plugin, and opens it. */
  onRerunCycle: () => void;
  /** Opens the cycle's Settings: title, description, Continue Automatically. */
  onSettings: () => void;
  onToggleFollowActive: () => void;
  onSeal: (() => void) | null;
  sealFailed: boolean;
};

/** The execution page's "…" menu. */
export type ExecutionMenu = {
  kind: "execution";
  onStart: (() => void) | null;
  startRefusal?: string;
  onDeleteCase: (() => void) | null;
  deleteRefusal?: string;
  /** `null` for a case that has not run, which has no execution to archive. */
  archive: ArchiveAction | null;
  /** Re-Run the execution's case; `null` while the execution is not sealed. */
  onRerun: (() => void) | null;
  /** Why the runtime refuses the re-run, when it does. */
  rerunRefusal?: string;
  /** Opens the execution's Settings; `null` for a case that has not run. */
  onSettings: (() => void) | null;
};
