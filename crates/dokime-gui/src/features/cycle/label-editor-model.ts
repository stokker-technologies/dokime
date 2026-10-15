/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/*
 * The inline label editor as plain state.
 *
 * The cycle and execution headers each name their node with an operator label.
 * The pencil beside it opens a draft of that label in place: Enter saves the
 * trimmed text, Escape drops the draft, and a refusal keeps the draft open with
 * the reason beside it. Saving an unchanged label closes the draft without a
 * call. The module has no imports, so a node test can load it directly.
 */

/* === Types === */

export type LabelDraft =
  | { mode: "closed" }
  | { mode: "editing"; original: string; text: string; error: string | null };

/** What a key does to an open draft: save it, drop it, refuse it, or nothing. */
export type LabelKeyAction =
  | { kind: "save"; label: string }
  | { kind: "cancel" }
  | { kind: "refuse"; error: string }
  | { kind: "none" };

/* === Constants === */

export const CLOSED_LABEL_DRAFT: LabelDraft = { mode: "closed" };

export const EMPTY_LABEL_ERROR = "A label cannot be empty.";

/* === Public Functions === */

/** Opens a draft holding the current label. */
export function openLabelEdit(label: string): LabelDraft {
  return { mode: "editing", original: label, text: label, error: null };
}

/** Typing replaces the text and clears an earlier refusal. */
export function updateLabelText(draft: LabelDraft, text: string): LabelDraft {
  return draft.mode === "editing" ? { ...draft, text, error: null } : draft;
}

/** A refused save keeps the draft open and states why. */
export function refuseLabel(draft: LabelDraft, error: string): LabelDraft {
  return draft.mode === "editing" ? { ...draft, error } : draft;
}

/** Enter saves, Escape cancels; every other key types. */
export function labelEditorKey(draft: LabelDraft, key: string): LabelKeyAction {
  if (draft.mode !== "editing") return { kind: "none" };
  if (key === "Escape") return { kind: "cancel" };
  if (key !== "Enter") return { kind: "none" };
  const label = draft.text.trim();
  if (!label) return { kind: "refuse", error: EMPTY_LABEL_ERROR };
  if (label === draft.original.trim()) return { kind: "cancel" };
  return { kind: "save", label };
}
