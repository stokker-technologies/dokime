/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The New Cycle form as plain values: what it opens with, what it refuses and
   the request it submits. Creating a cycle is always this explicit step; cases
   join it afterwards from the Library. */

/* === Local Imports === */

import type { NewCycleRequest } from "@/lib/api/native-planning";
import { defaultCycleLabel } from "@/lib/time-format";

/* === Types === */

export type NewCycleForm = {
  title: string;
  description: string;
  continueOnFailure: boolean;
};

/* === Constants === */

export const EMPTY_TITLE_ERROR = "A cycle needs a title.";

/* === Public Functions === */

/** The form as it opens at `now`: titled by that moment, continuing past failures. */
export function initialNewCycleForm(now: Date): NewCycleForm {
  return { continueOnFailure: true, description: "", title: defaultCycleLabel(now) };
}

/** Why the form cannot be submitted, or `null` when it can. */
export function newCycleFormError(form: NewCycleForm): string | null {
  return form.title.trim() ? null : EMPTY_TITLE_ERROR;
}

/** The CreateCycle request: trimmed text, a blank description omitted. */
export function newCycleRequest(form: NewCycleForm): NewCycleRequest {
  const description = form.description.trim();
  return {
    continueOnFailure: form.continueOnFailure,
    description: description || undefined,
    label: form.title.trim(),
  };
}
