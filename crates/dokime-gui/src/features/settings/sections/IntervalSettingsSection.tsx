/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The Interval Window and Lag Bound the Built-In Host Starts With. */

/* === External Imports === */

import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useId, useState } from "react";

/* === Local Imports === */

import { ErrorCard } from "@/components/ErrorCard";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { commands } from "@/lib/desktop/commands";
import type { IntervalSettings } from "@/lib/desktop/settings";
import { formatUnitLabel } from "@/lib/format";
import { failureOf } from "@/lib/notification-details";
import { SettingsCard } from "../SettingsCard";
import { SettingsRow } from "../SettingsRow";

/* === Constants === */

const QUERY_KEY = ["settings", "interval-settings"];
const MAX_WINDOW_MS = Number.MAX_SAFE_INTEGER;
const MAX_LAG_WINDOWS = 4_294_967_295;

/* === Types === */

type Draft = { window: string; lag: string };

/* === Components === */

export function IntervalSettingsSection() {
  const queryClient = useQueryClient();
  const windowId = useId();
  const lagId = useId();
  const [draft, setDraft] = useState<Draft | null>(null);
  const query = useQuery({ queryKey: QUERY_KEY, queryFn: () => commands.desktopIntervalSettingsRead() });
  const mutation = useMutation({
    mutationFn: (settings: IntervalSettings) => commands.desktopIntervalSettingsWrite(settings),
    onSuccess: (settings) => {
      queryClient.setQueryData(QUERY_KEY, settings);
      setDraft(null);
    },
  });
  const saved = query.data;
  const values = draft ?? (saved ? { window: String(saved.interval_window_ms), lag: String(saved.interval_lag_windows) } : null);
  const windowError = values ? wholeNumberError(values.window, MAX_WINDOW_MS) : null;
  const lagError = values ? wholeNumberError(values.lag, MAX_LAG_WINDOWS) : null;
  const changed =
    values !== null &&
    saved !== undefined &&
    (values.window.trim() !== String(saved.interval_window_ms) || values.lag.trim() !== String(saved.interval_lag_windows));
  const windowLabel = formatUnitLabel("Interval Window", "ms");
  const lagLabel = formatUnitLabel("Interval Lag", "windows");

  return (
    <SettingsCard
      title="Interval Sampling"
      description="This desktop's built-in host places interval streams with these values. Saved values apply when the built-in host next starts; an attached host keeps its own configuration."
    >
      {query.isPending ? (
        <p role="status" className="px-4 py-3 text-sm text-muted-foreground">
          Loading Interval Settings…
        </p>
      ) : null}
      {query.error ? <ErrorCard failure={failureOf(query.error, "Interval Settings Unavailable")} /> : null}
      {values ? (
        <>
          <SettingsRow
            htmlFor={windowId}
            label={windowLabel}
            description="Sampling time one window covers. A stream's own window setting overrides it."
            error={windowError ?? undefined}
          >
            <Input
              id={windowId}
              aria-label={windowLabel}
              aria-invalid={Boolean(windowError)}
              className="text-right tabular-nums"
              inputMode="numeric"
              value={values.window}
              onChange={(event) => setDraft({ ...values, window: event.target.value })}
            />
          </SettingsRow>
          <SettingsRow
            htmlFor={lagId}
            label={lagLabel}
            description="Window lengths a stream may fall behind before its execution is interrupted."
            error={lagError ?? undefined}
          >
            <Input
              id={lagId}
              aria-label={lagLabel}
              aria-invalid={Boolean(lagError)}
              className="text-right tabular-nums"
              inputMode="numeric"
              value={values.lag}
              onChange={(event) => setDraft({ ...values, lag: event.target.value })}
            />
          </SettingsRow>
          <SettingsRow
            label="Save Interval Settings"
            error={mutation.error ? failureOf(mutation.error, "Save Interval Settings").message : undefined}
            failure={mutation.error ?? undefined}
            status={mutation.isPending ? <span role="status">Saving</span> : null}
          >
            <Button
              disabled={!changed || Boolean(windowError) || Boolean(lagError) || mutation.isPending}
              onClick={() =>
                mutation.mutate({ interval_window_ms: Number(values.window.trim()), interval_lag_windows: Number(values.lag.trim()) })
              }
              size="sm"
            >
              Save
            </Button>
          </SettingsRow>
        </>
      ) : null}
    </SettingsCard>
  );
}

/* === Private Functions === */

function wholeNumberError(text: string, max: number): string | null {
  const trimmed = text.trim();
  const value = Number(trimmed);
  return /^\d+$/.test(trimmed) && Number.isSafeInteger(value) && value >= 1 && value <= max
    ? null
    : `Enter a whole number from 1 to ${max}.`;
}
