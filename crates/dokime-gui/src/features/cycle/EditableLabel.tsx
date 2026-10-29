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
 * The title and its pencil sit inline in one cell of the header grid, so the header
 * keeps its tracks whether the pencil shows or not. The pencil turns the label
 * into an input of the same type size: Enter saves through the runtime, Escape
 * drops the draft, and a refusal goes to notifications while retaining the input.
 * `label-editor-model` owns what each key does. Without `onRename` — the
 * frozen report, a sealed or archived subject, a case that never ran — the
 * title is plain text.
 */

/* === External Imports === */

import { Check, Pencil } from "lucide-react";
import { useState } from "react";

/* === Local Imports === */

import { NotificationMessage } from "@/components/NotificationMessage";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { CLOSED_LABEL_DRAFT, type LabelDraft, labelEditorKey, openLabelEdit, refuseLabel, updateLabelText } from "./label-editor-model";

/* === Types === */

export type EditableLabelProps = {
  label: string;
  initiallyEditing?: boolean;
  /** Text before the label that names the node but is not part of it. */
  prefix?: string;
  /** What the pencil renames, for its accessible name: "Cycle" or "Execution". */
  subject: string;
  /** Saves the label; `null` leaves the title read-only. */
  onRename: ((label: string) => Promise<unknown>) | null;
};

/* === Constants === */

const TITLE_CLASS = "font-heading text-lg leading-7 font-semibold";

/* === Components === */

export function EditableLabel({ label, prefix = "", subject, onRename, initiallyEditing = false }: EditableLabelProps) {
  const [draft, setDraft] = useState<LabelDraft>(() => (initiallyEditing ? openLabelEdit(label) : CLOSED_LABEL_DRAFT));
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
          <NotificationMessage source="Label" title="Label Could Not Save">
            {draft.error}
          </NotificationMessage>
        ) : (
          <span />
        )}
      </div>
    );
  }

  return (
    <div className="grid min-w-0 grid-cols-[minmax(0,auto)_auto] items-center gap-x-2">
      <h2 className={`min-w-0 truncate ${TITLE_CLASS}`} title={title}>
        {title}
      </h2>
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
