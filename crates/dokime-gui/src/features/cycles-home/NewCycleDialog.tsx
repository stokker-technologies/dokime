/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The one way a cycle comes into being: a title, an optional description and
   Continue Automatically, on the same settings form the cycle's own Settings
   dialog draws. The cycle starts empty; the caller decides where the operator
   goes next. */

/* === External Imports === */

import { type FormEvent, useEffect, useState } from "react";

/* === Local Imports === */

import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { SettingsFields } from "@/features/cycle/SettingsForm";
import { useCreateCycleMutation } from "@/features/cycles-home/cycles-queries";
import { initialNewCycleForm, type NewCycleForm, newCycleFormError, newCycleRequest } from "@/features/cycles-home/new-cycle-form";

/* === Types === */

type NewCycleDialogProps = {
  onCreated: (cycleId: string) => void;
  onOpenChange: (open: boolean) => void;
  open: boolean;
};

/* === Components === */

export function NewCycleDialog({ onCreated, onOpenChange, open }: NewCycleDialogProps) {
  const create = useCreateCycleMutation();
  const [form, setForm] = useState<NewCycleForm>(() => initialNewCycleForm(new Date()));
  const [error, setError] = useState<string | null>(null);

  /* Every opening starts over, titled by the moment it opened. */
  useEffect(() => {
    if (!open) return;
    setForm(initialNewCycleForm(new Date()));
    setError(null);
  }, [open]);

  function change(patch: Partial<NewCycleForm>) {
    setError(null);
    setForm((current) => ({ ...current, ...patch }));
  }

  async function submit(event: FormEvent) {
    event.preventDefault();
    if (create.isPending) return;
    const refusal = newCycleFormError(form);
    if (refusal) {
      setError(refusal);
      return;
    }
    try {
      const { cycleId } = await create.mutateAsync(newCycleRequest(form));
      onOpenChange(false);
      onCreated(cycleId);
    } catch (failure: unknown) {
      setError(failure instanceof Error ? failure.message : String(failure));
    }
  }

  return (
    <Dialog onOpenChange={(next) => !create.isPending && onOpenChange(next)} open={open}>
      <DialogContent>
        <form className="grid gap-4" onSubmit={(event) => void submit(event)}>
          <DialogHeader>
            <DialogTitle>New Cycle</DialogTitle>
            <DialogDescription>The cycle starts empty. Add cases to it from the Library.</DialogDescription>
          </DialogHeader>
          <SettingsFields
            onChange={(patch) => change(patch)}
            titleInvalid={error !== null && !form.title.trim()}
            values={{ continueAutomatically: form.continueAutomatically, description: form.description, title: form.title }}
          />
          {error ? (
            <p className="text-sm text-destructive" role="alert">
              {error}
            </p>
          ) : null}
          <DialogFooter>
            <Button disabled={create.isPending} onClick={() => onOpenChange(false)} type="button" variant="outline">
              Cancel
            </Button>
            <Button disabled={create.isPending} type="submit">
              Create Cycle
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
