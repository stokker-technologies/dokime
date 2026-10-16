/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The one way a cycle comes into being: a title, an optional description and
   its failure policy. The cycle starts empty; the caller decides where the
   operator goes next. */

/* === External Imports === */

import { type FormEvent, useEffect, useState } from "react";

/* === Local Imports === */

import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { Textarea } from "@/components/ui/textarea";
import { useCreateCycleMutation } from "@/features/cycles-home/cycles-queries";
import {
  initialNewCycleForm,
  type NewCycleForm,
  newCycleFormError,
  newCycleRequest,
} from "@/features/cycles-home/new-cycle-form";

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
            <DialogDescription>
              The cycle starts empty. Add cases to it from the Library.
            </DialogDescription>
          </DialogHeader>
          <div className="grid gap-2">
            <Label htmlFor="new-cycle-title">Title</Label>
            <Input
              aria-invalid={error !== null && !form.title.trim()}
              id="new-cycle-title"
              onChange={(event) => change({ title: event.target.value })}
              value={form.title}
            />
          </div>
          <div className="grid gap-2">
            <Label htmlFor="new-cycle-description">Description</Label>
            <Textarea
              id="new-cycle-description"
              onChange={(event) => change({ description: event.target.value })}
              placeholder="Optional"
              value={form.description}
            />
          </div>
          <div className="grid grid-cols-[minmax(0,1fr)_auto] items-center gap-4">
            <Label htmlFor="new-cycle-continue-on-failure">Continue after a failed case</Label>
            <Switch
              checked={form.continueOnFailure}
              id="new-cycle-continue-on-failure"
              onCheckedChange={(continueOnFailure) => change({ continueOnFailure })}
            />
          </div>
          {error ? (
            <p className="text-sm text-destructive" role="alert">
              {error}
            </p>
          ) : null}
          <DialogFooter>
            <Button
              disabled={create.isPending}
              onClick={() => onOpenChange(false)}
              type="button"
              variant="outline"
            >
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
