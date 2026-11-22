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

import { failureOf } from "@/lib/notification-details";
import { ErrorCard } from "@/components/ErrorCard";

import type { AutosaveState } from "@/features/settings/settings-autosave";

/* === Types === */

type SettingsFieldStatusProps = {
  onRetry: () => void;
  state: AutosaveState;
};

/* === Components === */

export function SettingsFieldStatus({ onRetry, state }: SettingsFieldStatusProps) {
  if (state.kind === "error")
    return (
      <ErrorCard
        notificationContext="action"
        failure={failureOf(state.failure ?? state.message, "Settings Could Not Be Saved")}
        onRetry={onRetry}
        retryLabel="Save Settings"
      />
    );
  return (
    <span role="status" className="inline-flex min-h-5 items-center text-xs text-muted-foreground">
      {state.kind === "idle" ? "" : state.kind === "saving" ? "Saving Settings…" : "Settings Saved"}
    </span>
  );
}
