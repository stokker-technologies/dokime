/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Where the backend stores its data, and how often storage is maintained.
 *
 * Every value here saves itself: the directory when the operator picks one,
 * the intervals and the retention when their field settles. Each row says what
 * became of its own write, and the card says once that the backend applies
 * them on restart.
 */

/* === External Imports === */

import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useId, useRef, useState } from "react";

/* === Local Imports === */

import { useNativeSessionOptional } from "@/app/native-session/NativeSessionProvider";
import type { ConnectedSession } from "@/app/native-session/session";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { SettingsCard } from "@/features/settings/SettingsCard";
import { SettingsFieldStatus } from "@/features/settings/SettingsFieldStatus";
import { SettingsPathField } from "@/features/settings/SettingsPathField";
import { SettingsRow } from "@/features/settings/SettingsRow";
import { type Autosave, useAutosave } from "@/features/settings/settings-autosave";
import {
  readStoragePolicy,
  type StoragePolicy,
  updateStoragePolicy,
} from "@/lib/api/native-storage-settings";
import {
  applicationSettingsError,
  useApplicationSettings,
  useSaveDataRoot,
} from "@/lib/application-settings";
import { useBrand } from "@/lib/brand";
import { commands } from "@/lib/desktop/commands";
import { isTauriDesktop } from "@/lib/desktop-open";

/* === Components === */

export function GeneralSection() {
  const brand = useBrand();
  if (!isTauriDesktop()) {
    return (
      <SettingsCard title="Output">
        <SettingsRow
          description="Execution storage belongs to the connected host."
          label="Execution Directory"
        >
          <span className="text-sm text-muted-foreground">
            Open Settings in the {brand.label} desktop application to choose it.
          </span>
        </SettingsRow>
      </SettingsCard>
    );
  }
  return <DesktopOutputSettings />;
}

export function DesktopOutputSettings() {
  const session = useNativeSessionOptional();
  const settings = useApplicationSettings();
  const save = useSaveDataRoot();
  const pick = useMutation({
    mutationFn: async () => {
      const directory = await commands.desktopPickExecutionDirectory();
      if (directory !== null) await save.mutateAsync(directory);
    },
  });
  const view = settings.data;
  const error = settings.error ?? pick.error ?? save.error;
  const canEdit =
    session?.snapshot.connection.backend?.methods.includes("/dokime.v1.Settings/UpdateDataRoot") ??
    false;
  const openable = session?.snapshot.ownership === "owned_local";
  const busy = pick.isPending || save.isPending;

  return (
    <>
      <SettingsCard
        description="Choose the data directory the backend uses when it next starts."
        title="Output"
      >
        {!session ? (
          <SettingsRow label="Status">
            <span className="text-sm text-muted-foreground" role="status">
              Settings are available once the backend is connected.
            </span>
          </SettingsRow>
        ) : null}
        {settings.isPending && session ? (
          <SettingsRow label="Status">
            <span className="text-sm text-muted-foreground" role="status">
              Loading application settings…
            </span>
          </SettingsRow>
        ) : null}
        {error ? (
          <SettingsRow
            label="Status"
            status={
              settings.isError ? (
                <Button onClick={() => void settings.refetch()} size="sm" variant="outline">
                  Try Again
                </Button>
              ) : null
            }
          >
            <span className="text-sm text-destructive" role="alert">
              {applicationSettingsError(error)}
            </span>
          </SettingsRow>
        ) : null}

        {view ? (
          <>
            <SettingsRow label="Current Data Directory">
              <SettingsPathField openable={openable} value={view.effectiveDataRoot} />
            </SettingsRow>
            <SettingsRow label="Saved Directory">
              <SettingsPathField
                openable={openable}
                placeholder={`Application default: ${view.defaultDataRoot}`}
                value={view.configuredDataRoot}
              />
            </SettingsRow>
            <SettingsRow
              description={
                canEdit
                  ? "Existing executions stay in their original directory; no files are moved."
                  : "Administrator access is required to change the data directory."
              }
              label="Choose A Directory"
              status={
                busy ? (
                  <span className="text-xs text-muted-foreground" role="status">
                    Saving
                  </span>
                ) : save.isSuccess ? (
                  <span className="text-xs text-muted-foreground" role="status">
                    Saved
                  </span>
                ) : null
              }
            >
              <div className="flex flex-wrap justify-end gap-2">
                {view.configuredDataRoot !== null ? (
                  <Button
                    disabled={!canEdit || busy}
                    onClick={() => save.mutate(null)}
                    variant="ghost"
                  >
                    Use Default
                  </Button>
                ) : null}
                <Button disabled={!canEdit || busy} onClick={() => pick.mutate()} variant="outline">
                  {pick.isPending ? "Choosing…" : "Change Folder…"}
                </Button>
              </div>
            </SettingsRow>
          </>
        ) : null}
      </SettingsCard>
      <BackendPolicySettings />
    </>
  );
}

/* === Private Components === */

function BackendPolicySettings() {
  const session = useNativeSessionOptional();
  return session ? (
    <ConnectedPolicySettings key={session.identity.generation} session={session} />
  ) : null;
}

function ConnectedPolicySettings({ session }: { session: ConnectedSession }) {
  const client = useQueryClient();
  const queryKey = ["settings", "storage-policy", session.identity.generation];
  const query = useQuery({
    queryKey,
    queryFn: () => readStoragePolicy(session.identity),
    retry: false,
  });
  const pending = useRef<Promise<unknown>>(Promise.resolve());
  const save = (patch: Partial<StoragePolicy>) => {
    const operation = pending.current
      .catch(() => undefined)
      .then(async () => {
        if (!session.identity.isCurrent()) throw new Error("Backend connection changed");
        await client.cancelQueries({ queryKey });
        const current = await readStoragePolicy(session.identity);
        const saved = await updateStoragePolicy(session.identity, current.revision, {
          ...current.configured,
          ...patch,
        });
        if (session.identity.isCurrent()) client.setQueryData(queryKey, saved);
        return saved;
      });
    pending.current = operation;
    return operation;
  };
  const interval = useAutosave<number>(
    (saveIntervalMs) => save({ saveIntervalMs }),
    applicationSettingsError,
  );
  const compaction = useAutosave<number>(
    (hours) => save({ compactionIntervalMs: hours * 3_600_000 }),
    applicationSettingsError,
  );
  const retention = useAutosave<number | null>(
    (days) => save({ localRetentionMs: days === null ? null : days * 86_400_000 }),
    applicationSettingsError,
  );
  const view = query.data;
  const canEdit =
    session.snapshot.connection.backend?.methods.includes(
      "/dokime.v1.Settings/UpdateStoragePolicy",
    ) ?? false;
  return (
    <SettingsCard
      title="Backend Storage"
      description="Policy belongs to the connected backend and applies when that backend restarts."
    >
      {query.isPending ? (
        <SettingsRow label="Status">
          <span role="status">Loading storage policy…</span>
        </SettingsRow>
      ) : null}
      {query.error ? (
        <SettingsRow
          label="Status"
          status={
            <Button onClick={() => void query.refetch()} size="sm" variant="outline">
              Try Again
            </Button>
          }
        >
          <span role="alert">{applicationSettingsError(query.error)}</span>
        </SettingsRow>
      ) : null}
      {!canEdit ? (
        <SettingsRow label="Access">
          <span>Administrator access is required to change storage policy.</span>
        </SettingsRow>
      ) : null}
      {view ? (
        <fieldset className="divide-y" disabled={!canEdit}>
          <IntervalRow
            autosave={interval}
            description="How often running execution data is saved."
            label="Save Interval (Seconds)"
            max={86_400}
            min={0.001}
            parse={(text) => {
              const seconds = Number(text);
              const value = Math.round(seconds * 1000);
              return Number.isSafeInteger(value) &&
                value >= 1 &&
                value <= 86_400_000 &&
                seconds === value / 1000
                ? value
                : null;
            }}
            step="any"
            value={String(view.configured.saveIntervalMs / 1000)}
          />
          <IntervalRow
            autosave={compaction}
            description="Combines small storage files in the background; saves wait while it runs."
            label="Compaction Interval (Hours)"
            max={744}
            min={1}
            parse={(text) => {
              const value = Number(text);
              return Number.isSafeInteger(value) && value >= 1 && value <= 744 ? value : null;
            }}
            step={1}
            value={String(view.configured.compactionIntervalMs / 3_600_000)}
          />
          <RetentionRow
            autosave={retention}
            days={
              view.configured.localRetentionMs === null
                ? null
                : view.configured.localRetentionMs / 86_400_000
            }
          />
        </fieldset>
      ) : null}
      {view?.restartRequired ? (
        <SettingsRow label="Pending Restart">
          <span role="status">
            Saved. Restart the connected backend to apply this policy. Its current policy remains
            active until then.
          </span>
        </SettingsRow>
      ) : null}
    </SettingsCard>
  );
}

function IntervalRow({
  autosave,
  description,
  label,
  max,
  min,
  parse,
  step,
  value,
}: {
  autosave: Autosave<number>;
  description: string;
  label: string;
  max: number;
  min: number;
  /** Null for a value the backend would reject, which is never written. */
  parse: (text: string) => number | null;
  step: number | "any";
  value: string;
}) {
  const id = useId();
  const [text, setText] = useState(value);
  const parsed = parse(text);

  return (
    <SettingsRow
      description={parsed === null ? "Enter a value inside the allowed range." : description}
      htmlFor={id}
      label={label}
      status={<SettingsFieldStatus onRetry={autosave.retry} state={autosave.state} />}
    >
      <Input
        aria-invalid={parsed === null || undefined}
        className="max-w-40"
        id={id}
        max={max}
        min={min}
        onChange={(event) => {
          setText(event.target.value);
          const next = parse(event.target.value);
          if (next !== null) autosave.change(next);
        }}
        step={step}
        type="number"
        value={text}
      />
    </SettingsRow>
  );
}

function RetentionRow({
  autosave,
  days,
}: {
  autosave: Autosave<number | null>;
  days: number | null;
}) {
  const id = useId();
  const [selected, setSelected] = useState(days === null ? "all" : String(days));

  return (
    <SettingsRow
      description="Old local copies go only when an identical verified remote copy exists."
      htmlFor={id}
      label="Local Cache Retention"
      status={<SettingsFieldStatus onRetry={autosave.retry} state={autosave.state} />}
    >
      <select
        className="h-[var(--control-height)] w-56 rounded-md border bg-background px-2 text-sm"
        id={id}
        onChange={(event) => {
          setSelected(event.target.value);
          autosave.commit(event.target.value === "all" ? null : Number(event.target.value));
        }}
        value={selected}
      >
        <option value="all">Keep All Local Data</option>
        <option value="7">One Week</option>
        <option value="30">One Month</option>
        <option value="90">Three Months</option>
        {days !== null && ![7, 30, 90].includes(days) ? (
          <option value={String(days)}>{days} Days</option>
        ) : null}
      </select>
    </SettingsRow>
  );
}
