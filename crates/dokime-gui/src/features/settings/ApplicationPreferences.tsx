/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/** Scoped Preferences With Inline Autosave. */

import type { ApplicationSettings, SettingsScope, SettingsSnapshot } from "@generated/application-settings";
import { useEffect, useRef, useState } from "react";
import { NotificationMessage } from "@/components/NotificationMessage";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Switch } from "@/components/ui/switch";
import { toast } from "@/lib/notifications";
import { preferencesError, usePreferences, useSavePreferences } from "./application-preferences";
import { SettingsCard } from "./SettingsCard";
import { SettingsRow } from "./SettingsRow";
import { createDebouncedWriter } from "./settings-autosave";

type PreferencesSection = "appearance" | "remote";
type PreferenceGroup = keyof ApplicationSettings;
type PreferenceField = {
  group: PreferenceGroup;
  key: string;
  label: string;
  help: string;
  advanced?: boolean;
  unit?: string;
  max?: number;
  options?: string[];
  kind?: "boolean" | "list";
};

const FIELDS: PreferenceField[] = [
  {
    group: "appearance",
    key: "theme_mode",
    label: "Theme Mode",
    help: "Choose the desktop color mode.",
    options: ["system", "light", "dark"],
  },
  {
    group: "appearance",
    key: "theme_family",
    label: "Theme Family",
    help: "Use a theme family, or leave empty for the organization default.",
  },
  {
    group: "appearance",
    key: "density",
    label: "Density",
    help: "Set the space around desktop controls.",
    options: ["comfortable", "compact", "touch"],
  },
  {
    group: "report",
    key: "plot_points",
    label: "Report Points",
    help: "Maximum points in newly generated plots.",
    unit: "points",
    max: 100000,
  },
  {
    group: "report",
    key: "plot_downsample",
    label: "Report Downsampling",
    help: "Reduction method for newly generated plots.",
    options: ["auto", "raw", "min_max", "lttb"],
  },
  {
    group: "client",
    key: "connect_timeout_ms",
    label: "Connect Timeout (ms)",
    help: "Time allowed to establish a new connection.",
    unit: "ms",
    advanced: true,
  },
  {
    group: "client",
    key: "request_timeout_ms",
    label: "Request Timeout (ms)",
    help: "Time allowed for a new request.",
    unit: "ms",
    advanced: true,
  },
  {
    group: "telemetry",
    key: "interval_ms",
    label: "Telemetry Interval (ms)",
    help: "Delay between telemetry samples.",
    unit: "ms",
    advanced: true,
  },
  {
    group: "listener",
    key: "enabled",
    label: "Listener Enabled",
    help: "Allow remote clients after restarting the listener.",
    kind: "boolean",
  },
  { group: "listener", key: "address", label: "Listener Address", help: "Host and port to bind, for example 0.0.0.0:8765." },
  { group: "listener", key: "tls_enabled", label: "Listener TLS", help: "Encrypt traffic using this host’s certificate.", kind: "boolean" },
  {
    group: "listener",
    key: "authentication_enabled",
    label: "Listener Authentication",
    help: "Require an access key from remote clients.",
    kind: "boolean",
  },
  {
    group: "listener",
    key: "certificate_names",
    label: "Certificate Names",
    help: "DNS names or IP addresses separated by commas. Regenerate the certificate to apply.",
    kind: "list",
    advanced: true,
  },
];
const SELECT_CLASS =
  "h-[var(--control-height)] w-full rounded-lg border border-input bg-background px-3 text-sm focus-visible:outline-2 focus-visible:outline-ring";

export function ApplicationPreferences({
  advanced = false,
  localOnly = false,
  section,
  onPendingChange,
}: {
  onPendingChange?: (pending: boolean) => void;
  advanced?: boolean;
  localOnly?: boolean;
  section?: PreferencesSection;
}) {
  return (
    <>
      {section !== "remote" && <ScopedPreferences scope="local" advanced={advanced} section={section} />}
      {!localOnly && <ScopedPreferences scope="backend" advanced={advanced} section={section} onPendingChange={onPendingChange} />}
    </>
  );
}

function ScopedPreferences({
  scope,
  advanced,
  section,
  onPendingChange,
}: {
  scope: SettingsScope;
  advanced: boolean;
  section?: PreferencesSection;
  onPendingChange?: (pending: boolean) => void;
}) {
  const query = usePreferences(scope);
  return (
    <SettingsCard title={scope === "local" ? "Local Preferences" : "Backend Preferences"}>
      {query.isPending ? (
        <NotificationMessage source="Settings" tone="info">
          Loading settings.
        </NotificationMessage>
      ) : query.error ? (
        <div className="grid justify-items-start gap-3 p-4 text-sm">
          <NotificationMessage>{preferencesError(query.error)}</NotificationMessage>
          <Button variant="outline" onClick={() => void query.refetch()}>
            Retry
          </Button>
        </div>
      ) : query.data ? (
        <PreferencesForm snapshot={query.data} scope={scope} advanced={advanced} section={section} onPendingChange={onPendingChange} />
      ) : null}
    </SettingsCard>
  );
}

function PreferencesForm({
  snapshot,
  scope,
  advanced,
  section,
  onPendingChange,
}: {
  onPendingChange?: (pending: boolean) => void;
  snapshot: SettingsSnapshot;
  scope: SettingsScope;
  advanced: boolean;
  section?: PreferencesSection;
}) {
  const [draft, setDraft] = useState<ApplicationSettings>(() => structuredClone(snapshot.settings));
  const [edited, setEdited] = useState<Set<string>>(new Set());
  const [rawText, setRawText] = useState<Record<string, string>>({});
  const [invalid, setInvalid] = useState<Record<string, string>>({});
  const [state, setState] = useState<"saved" | "saving" | "error">("saved");
  const [failure, setFailure] = useState<unknown>(null);
  const save = useSavePreferences(scope);
  const query = usePreferences(scope);
  const baseline = useRef(snapshot);
  const writing = useRef(false);
  const version = useRef(0);
  const saveRef = useRef(save.mutateAsync);
  saveRef.current = save.mutateAsync;
  useEffect(() => {
    if (edited.size === 0 && !writing.current) {
      baseline.current = snapshot;
      setDraft(structuredClone(snapshot.settings));
    }
  }, [snapshot, edited.size]);
  useEffect(() => {
    onPendingChange?.(edited.size > 0 || Boolean(failure));
  }, [edited.size, failure, onPendingChange]);
  const writer = useRef<ReturnType<typeof createDebouncedWriter<{ settings: ApplicationSettings; version: number }>> | null>(null);
  if (!writer.current)
    writer.current = createDebouncedWriter(
      async (input) => {
        writing.current = true;
        try {
          const previous = baseline.current;
          const settings =
            scope === "local"
              ? { ...previous.settings, appearance: input.settings.appearance, client: input.settings.client }
              : {
                  ...previous.settings,
                  report: input.settings.report,
                  telemetry: input.settings.telemetry,
                  listener: input.settings.listener,
                };
          const saved = await saveRef.current({ previous, settings });
          baseline.current = saved;
          toast.success("Settings Saved", { source: "Settings" });
          if (version.current === input.version) setEdited(new Set());
        } catch (error) {
          // A Newer Edit Supersedes A Rejected Draft Without Advancing Its Revision.
          if (validationFailure(error) && version.current !== input.version) return;
          setFailure(error);
          throw error;
        } finally {
          writing.current = false;
        }
      },
      (next) => setState(next.kind === "idle" ? "saved" : next.kind),
    );

  useEffect(() => {
    if (!edited.size || failure) return;
    if (Object.keys(invalid).length) {
      writer.current?.cancel();
      return;
    }
    writer.current?.change({ settings: draft, version: version.current });
  }, [draft, edited, invalid, failure]);
  // Leaving A Section Flushes Valid Edits Instead Of Dropping Them.
  useEffect(() => () => writer.current?.flush(), []);

  const fields = FIELDS.filter((field) => {
    const local = field.group === "appearance" || field.group === "client";
    return (
      local === (scope === "local") &&
      (section === "remote"
        ? field.group === "listener"
        : section === "appearance"
          ? field.group === "appearance"
          : field.group !== "listener")
    );
  });
  function row(field: PreferenceField) {
    const name = `${field.group}.${field.key}`;
    const value = (draft[field.group] as unknown as Record<string, string | number | boolean | string[] | null>)[field.key];
    const error = invalid[name] ?? (edited.has(name) && (failure || save.error) ? preferencesError(failure ?? save.error) : null);
    const change = (next: string | number | boolean | string[] | null) => {
      if (validationFailure(failure ?? save.error)) {
        writer.current?.reset();
        save.reset();
        setFailure(null);
      }
      version.current++;
      setEdited((current) => new Set(current).add(name));
      setState("saving");
      setInvalid((current) => {
        const updated = { ...current };
        if (field.unit && (!Number.isSafeInteger(next) || Number(next) <= 0 || Number(next) > (field.max ?? Number.MAX_SAFE_INTEGER)))
          updated[name] = `Enter a whole number from 1 to ${field.max ?? Number.MAX_SAFE_INTEGER}.`;
        else if (
          field.group === "listener" &&
          field.key === "address" &&
          (typeof next !== "string" ||
            !/^.+:\d+$/.test(next.trim()) ||
            Number(next.slice(next.lastIndexOf(":") + 1)) < 1 ||
            Number(next.slice(next.lastIndexOf(":") + 1)) > 65535)
        )
          updated[name] = "Enter a host and port from 1 to 65535.";
        else delete updated[name];
        return updated;
      });
      setDraft((current) => ({ ...current, [field.group]: { ...current[field.group], [field.key]: next } }));
    };
    return (
      <SettingsRow
        key={name}
        label={field.label}
        description={field.help}
        htmlFor={`${scope}-${name}`}
        error={error}
        status={<span data-state={error ? "error" : edited.has(name) ? state : "saved"} className="inline-block min-h-5" />}
      >
        {field.kind === "boolean" ? (
          <Switch id={`${scope}-${name}`} aria-label={field.label} checked={Boolean(value)} onCheckedChange={change} />
        ) : field.options ? (
          <select
            id={`${scope}-${name}`}
            className={SELECT_CLASS}
            aria-label={field.label}
            value={String(value)}
            onChange={(event) => change(event.target.value)}
          >
            {field.options.map((option) => (
              <option key={option} value={option}>
                {option === "lttb" ? "LTTB" : option.replaceAll("_", " ").replace(/\b[a-z]/g, (letter) => letter.toUpperCase())}
              </option>
            ))}
          </select>
        ) : (
          <div className="flex w-full items-center gap-2">
            <Input
              id={`${scope}-${name}`}
              aria-label={field.label}
              aria-invalid={Boolean(error)}
              className={field.unit ? "text-right tabular-nums" : undefined}
              type={field.unit ? "number" : "text"}
              min={field.unit ? 1 : undefined}
              max={field.max}
              step={field.unit ? 1 : undefined}
              value={
                typeof value === "number" && !Number.isFinite(value)
                  ? ""
                  : Array.isArray(value)
                    ? (rawText[name] ?? value.join(", "))
                    : typeof value === "boolean"
                      ? ""
                      : (value ?? "")
              }
              onChange={(event) => {
                if (field.kind === "list") setRawText((current) => ({ ...current, [name]: event.target.value }));
                change(
                  field.unit
                    ? event.target.valueAsNumber
                    : field.kind === "list"
                      ? event.target.value.split(/[\s,]+/).filter(Boolean)
                      : event.target.value || null,
                );
              }}
            />
            {field.unit && <span className="text-xs text-muted-foreground">{field.unit}</span>}
          </div>
        )}
      </SettingsRow>
    );
  }
  return (
    <div>
      {fields.filter((field) => !field.advanced).map(row)}
      {advanced && fields.some((field) => field.advanced) && (
        <>
          <h3 className="bg-muted/20 px-4 py-3 text-xs font-semibold">Advanced</h3>
          {fields.filter((field) => field.advanced).map(row)}
        </>
      )}
      {(failure || save.error) && (
        <div className="flex items-center justify-between gap-3 px-4 py-3">
          <NotificationMessage>{preferencesError(failure ?? save.error)}</NotificationMessage>
          <Button
            variant="outline"
            onClick={async () => {
              // A conflict must be reviewed against the latest revision before another edit.
              const fresh = await readFreshSnapshot();
              if (fresh) {
                writer.current?.reset();
                baseline.current = fresh;
                setDraft(structuredClone(fresh.settings));
                setEdited(new Set());
                setRawText({});
                setInvalid({});
                setFailure(null);
                save.reset();
                setState("saved");
              }
            }}
          >
            Retry
          </Button>
        </div>
      )}
    </div>
  );

  async function readFreshSnapshot() {
    const fresh = await query.refetch();
    if (fresh.error) {
      setFailure(fresh.error);
      return null;
    }
    return fresh.data ?? null;
  }
}

function validationFailure(error: unknown): boolean {
  return typeof error === "object" && error !== null && "kind" in error && error.kind === "validation";
}
