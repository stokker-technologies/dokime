/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* What became of the last autosave, stated at the row that caused it. */

import type { AutosaveState } from "@/features/settings/settings-autosave";

type SettingsFieldStatusProps = {
  onRetry: () => void;
  state: AutosaveState;
};

export function SettingsFieldStatus({ onRetry, state }: SettingsFieldStatusProps) {
  if (state.kind === "idle") return null;

  if (state.kind === "error") {
    return (
      <span className="flex items-center gap-2 text-xs text-destructive" role="alert">
        <span className="max-w-60 truncate" title={state.message}>
          {state.message}
        </span>
        <button className="underline underline-offset-2" onClick={onRetry} type="button">
          Retry
        </button>
      </span>
    );
  }

  return (
    <span className="text-xs text-muted-foreground" role="status">
      {state.kind === "saving" ? "Saving" : "Saved"}
    </span>
  );
}
