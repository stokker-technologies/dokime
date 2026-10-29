/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* What became of the last autosave, stated at the row that caused it. */

/* === External Imports === */

/* === Local Imports === */

import { NotificationMessage } from "@/components/NotificationMessage";
import type { AutosaveState } from "@/features/settings/settings-autosave";

/* === Types === */

type SettingsFieldStatusProps = {
  onRetry: () => void;
  state: AutosaveState;
};

/* === Components === */

export function SettingsFieldStatus({ onRetry, state }: SettingsFieldStatusProps) {
  return (
    <span className="inline-flex min-h-5 items-center gap-2 text-xs">
      <NotificationMessage
        source="Settings"
        title={state.kind === "error" ? "Settings Could Not Save" : "Settings Updated"}
        tone={state.kind === "error" ? "error" : state.kind === "saved" ? "success" : "info"}
      >
        {state.kind === "idle"
          ? ""
          : state.kind === "error"
            ? state.message
            : state.kind === "saving"
              ? "Saving settings."
              : "Settings saved."}
      </NotificationMessage>
      {state.kind === "error" && (
        <button className="underline underline-offset-2" onClick={onRetry} type="button">
          Retry
        </button>
      )}
    </span>
  );
}
