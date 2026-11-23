// Dokime — SPDX-License-Identifier: Apache-2.0
import type { TestCycle } from "../../lib/types";

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

/**
 * What a cycle's Settings may change. Its description is written once, when
 * the cycle is created; everything is read-only once it seals or archives.
 */
export function cycleSettingsReadOnly(phase: TestCycle["phase"], archived: boolean): SettingsReadOnly {
  const description = "Set when the cycle was created.";
  if (phase !== "active") {
    const closed = "Closed cycles keep the settings they ran with.";
    return { continueAutomatically: closed, description, title: closed };
  }
  return archived ? { description, title: "Unarchive the cycle to rename it." } : { description };
}
