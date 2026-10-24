/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * The one settings form: New Cycle, the cycle viewer's Settings and the
 * execution viewer's Settings all draw it.
 *
 * `SettingsFields` lays out the fields a node has — title, description,
 * Continue Automatically — on one grid, each editable or read-only as the
 * node's lifecycle allows. `SettingsDialog` frames it with the dialog every
 * viewer opens from its "…" menu: it starts from the node's current values
 * each time it opens, saves through the caller, and keeps a refusal beside
 * the form instead of a toast.
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
import { type FormEvent, useEffect, useId, useState } from "react";

// Local Imports
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { Textarea } from "@/components/ui/textarea";

/* === Re-Exports === */

/* === Constants === */

export const EMPTY_SETTINGS_TITLE_ERROR = "A title is required.";

/* === Types === */

/** The values the form edits; a field absent here is not shown. */
export type SettingsValues = {
  title: string;
  description?: string;
  continueAutomatically?: boolean;
};

/** Which shown fields are read-only, and why, as a hint under the field. */
export type SettingsReadOnly = Partial<Record<keyof SettingsValues, string>>;

export type SettingsFieldsProps = {
  values: SettingsValues;
  onChange: (patch: Partial<SettingsValues>) => void;
  readOnly?: SettingsReadOnly;
  /** Marks the title invalid after a refused submit. */
  titleInvalid?: boolean;
  /** The fields' id prefix, `new-cycle` gives `new-cycle-title`; unique when absent. */
  idPrefix?: string;
};

export type SettingsDialogProps = {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  title: string;
  description: string;
  /** The node's current values; the form starts from them on every opening. */
  values: SettingsValues;
  readOnly?: SettingsReadOnly;
  /** Saves the changed values; rejects with the refusal to show. */
  onSubmit: (values: SettingsValues) => Promise<unknown>;
  submitLabel?: string;
};

/* === Components === */

export function SettingsFields({ idPrefix, onChange, readOnly = {}, titleInvalid = false, values }: SettingsFieldsProps) {
  const generated = useId();
  const id = idPrefix ?? generated;
  return (
    <div className="grid gap-4">
      <div className="grid gap-2">
        <Label htmlFor={`${id}-title`}>Title</Label>
        <Input
          aria-invalid={titleInvalid}
          disabled={Boolean(readOnly.title)}
          id={`${id}-title`}
          onChange={(event) => onChange({ title: event.target.value })}
          value={values.title}
        />
        <ReadOnlyHint text={readOnly.title} />
      </div>
      {values.description === undefined ? null : (
        <div className="grid gap-2">
          <Label htmlFor={`${id}-description`}>Description</Label>
          <Textarea
            disabled={Boolean(readOnly.description)}
            id={`${id}-description`}
            onChange={(event) => onChange({ description: event.target.value })}
            placeholder="Optional"
            value={values.description}
          />
          <ReadOnlyHint text={readOnly.description} />
        </div>
      )}
      {values.continueAutomatically === undefined ? null : (
        <div className="grid grid-cols-[minmax(0,1fr)_auto] items-center gap-x-4 gap-y-1">
          <Label htmlFor={`${id}-continue-automatically`}>Continue Automatically</Label>
          <Switch
            checked={values.continueAutomatically}
            disabled={Boolean(readOnly.continueAutomatically)}
            id={`${id}-continue-automatically`}
            onCheckedChange={(continueAutomatically) => onChange({ continueAutomatically })}
          />
          <p className="col-span-2 text-xs text-muted-foreground">
            {readOnly.continueAutomatically ?? "After an execution completes, the next unstarted one starts, whatever its verdict."}
          </p>
        </div>
      )}
    </div>
  );
}

export function SettingsDialog({
  description,
  onOpenChange,
  onSubmit,
  open,
  readOnly = {},
  submitLabel = "Save Settings",
  title,
  values,
}: SettingsDialogProps) {
  const [draft, setDraft] = useState<SettingsValues>(values);
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);

  /* Every opening starts from the node as it is now. */
  // biome-ignore lint/correctness/useExhaustiveDependencies: `values` is read when the dialog opens, not tracked while it is open.
  useEffect(() => {
    if (!open) return;
    setDraft(values);
    setError(null);
  }, [open]);

  const submit = (event: FormEvent) => {
    event.preventDefault();
    if (saving) return;
    if (!draft.title.trim()) {
      setError(EMPTY_SETTINGS_TITLE_ERROR);
      return;
    }
    setSaving(true);
    onSubmit({ ...draft, title: draft.title.trim() })
      .then(() => onOpenChange(false))
      .catch((failure: unknown) => setError(failure instanceof Error ? failure.message : String(failure)))
      .finally(() => setSaving(false));
  };
  const editable = Object.keys(values).some((key) => !readOnly[key as keyof SettingsValues]);

  return (
    <Dialog onOpenChange={(next) => !saving && onOpenChange(next)} open={open}>
      <DialogContent>
        <form className="grid gap-4" onSubmit={submit}>
          <DialogHeader>
            <DialogTitle>{title}</DialogTitle>
            <DialogDescription>{description}</DialogDescription>
          </DialogHeader>
          <SettingsFields
            onChange={(patch) => {
              setError(null);
              setDraft((current) => ({ ...current, ...patch }));
            }}
            readOnly={readOnly}
            titleInvalid={error === EMPTY_SETTINGS_TITLE_ERROR}
            values={draft}
          />
          {error ? (
            <p className="text-sm text-destructive" role="alert">
              {error}
            </p>
          ) : null}
          <DialogFooter>
            <Button disabled={saving} onClick={() => onOpenChange(false)} type="button" variant="outline">
              {editable ? "Cancel" : "Close"}
            </Button>
            {editable ? (
              <Button disabled={saving} type="submit">
                {submitLabel}
              </Button>
            ) : null}
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}

/* === Hooks === */

/* === Public Functions === */

/**
 * What a cycle's Settings may change. Its description is written once, when
 * the cycle is created; everything is read-only once it seals or archives.
 */
export function cycleSettingsReadOnly(phase: "active" | "sealing" | "sealed", archived: boolean): SettingsReadOnly {
  const description = "Set when the cycle was created.";
  if (phase !== "active") {
    const closed = "Sealed cycles keep the settings they ran with.";
    return { continueAutomatically: closed, description, title: closed };
  }
  return archived ? { description, title: "Unarchive the cycle to rename it." } : { description };
}

/* === Private Functions === */

function ReadOnlyHint({ text }: { text?: string }) {
  return text ? <p className="text-xs text-muted-foreground">{text}</p> : null;
}
