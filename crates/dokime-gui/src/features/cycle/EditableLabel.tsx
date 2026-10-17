/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* A header title the operator can rename in place.
 *
 * The title and its pencil share one cell of the header grid, so the header
 * keeps its tracks whether the pencil shows or not. The pencil turns the label
 * into an input of the same type size: Enter saves through the runtime, Escape
 * drops the draft, and a refusal stays beside the input instead of a toast.
 * `label-editor-model` owns what each key does. Without `onRename` — the
 * frozen report, a sealed or archived subject, a case that never ran — the
 * title is plain text.
 */

/* === External Imports === */

import { Check, Pencil } from "lucide-react";
import { useState } from "react";

/* === Local Imports === */

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { CLOSED_LABEL_DRAFT, type LabelDraft, labelEditorKey, openLabelEdit, refuseLabel, updateLabelText } from "./label-editor-model";

/* === Types === */

export type EditableLabelProps = {
  label: string;
  /** Text before the label that names the node but is not part of it. */
  prefix?: string;
  /** What the pencil renames, for its accessible name: "Cycle" or "Execution". */
  subject: string;
  /** Saves the label; `null` leaves the title read-only. */
  onRename: ((label: string) => Promise<unknown>) | null;
  /** `columns` puts the pencil in a trailing icon column of the bar grid, so it
   *  lines up under the page bar's buttons; `inline` keeps it beside the title. */
  layout?: "inline" | "columns";
};

/* === Constants === */

const TITLE_CLASS = "font-heading text-lg leading-7 font-semibold";

/* === Components === */

export function EditableLabel({ label, prefix = "", subject, onRename, layout = "inline" }: EditableLabelProps) {
  const [draft, setDraft] = useState<LabelDraft>(CLOSED_LABEL_DRAFT);
  const [saving, setSaving] = useState(false);
  const title = `${prefix}${label}`;

  const save = (next: string) => {
    if (!onRename) return;
    setSaving(true);
    onRename(next)
      .then(() => setDraft(CLOSED_LABEL_DRAFT))
      .catch((error: unknown) => setDraft((current) => refuseLabel(current, error instanceof Error ? error.message : String(error))))
      .finally(() => setSaving(false));
  };
  const press = (key: string) => {
    const action = labelEditorKey(draft, key);
    switch (action.kind) {
      case "save":
        save(action.label);
        return true;
      case "cancel":
        setDraft(CLOSED_LABEL_DRAFT);
        return true;
      case "refuse":
        setDraft((current) => refuseLabel(current, action.error));
        return true;
      case "none":
        return false;
    }
  };

  if (draft.mode === "editing" && onRename) {
    return (
      <div className="grid min-w-0 grid-cols-[auto_minmax(0,auto)_auto_minmax(0,auto)] items-center gap-x-2">
        {prefix ? <span className={`${TITLE_CLASS} whitespace-pre`}>{prefix}</span> : <span />}
        <Input
          aria-describedby={draft.error ? "label-editor-error" : undefined}
          aria-invalid={Boolean(draft.error)}
          aria-label={`${subject} Label`}
          autoFocus
          className={`h-[var(--bar-control-height)] min-w-[16ch] ${TITLE_CLASS}`}
          disabled={saving}
          onChange={(event) => setDraft(updateLabelText(draft, event.target.value))}
          onKeyDown={(event) => {
            if (press(event.key)) event.preventDefault();
          }}
          value={draft.text}
        />
        <Button
          aria-label={`Save ${subject} Label`}
          className="size-[var(--bar-control-height)]"
          disabled={saving}
          onClick={() => press("Enter")}
          size="icon"
          title="Save (Enter) · Cancel (Escape)"
          variant="outline"
        >
          <Check />
        </Button>
        {draft.error ? (
          <p aria-live="polite" className="min-w-0 truncate text-xs text-destructive" id="label-editor-error" title={draft.error}>
            {draft.error}
          </p>
        ) : (
          <span />
        )}
      </div>
    );
  }

  return (
    <div
      className={
        layout === "columns"
          ? "grid min-w-0 grid-cols-[minmax(0,1fr)_var(--bar-control-height)] items-center gap-x-[var(--bar-gap)]"
          : "grid min-w-0 grid-cols-[minmax(0,auto)_auto] items-center gap-x-2"
      }
    >
      <h1 className={`min-w-0 truncate ${TITLE_CLASS}`} title={title}>
        {title}
      </h1>
      {onRename ? (
        <Button
          aria-label={`Rename ${subject}`}
          className="size-[var(--bar-control-height)]"
          onClick={() => setDraft(openLabelEdit(label))}
          size="icon"
          title={`Rename ${subject}`}
          variant="outline"
        >
          <Pencil />
        </Button>
      ) : (
        <span />
      )}
    </div>
  );
}
