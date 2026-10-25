/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The pre-connection screen: startup, and the attach form.
 *
 * One centred column on the paper ground. The mark appears once with the
 * product name beneath it, then one sentence of state, then the startup phases
 * the backend reports as a quiet vertical step list. A failed local start
 * keeps its headline and adds the backend's exit, log file and diagnostics,
 * and offers to back up and reset the local storage it could not open. Cancel and Desktop
 * Settings are one balanced pair of equal-width secondary buttons. The attach
 * form keeps the same column width and puts its fields on a two-column
 * label/value grid so nothing inside the centred box is left ragged.
 */

/* === External Imports === */

import { Check } from "lucide-react";
import { type FormEvent, type ReactNode, useEffect, useId, useState, useSyncExternalStore } from "react";

/* === Local Imports === */

import { PathPill } from "@/components/PathPill";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { defaultBrand } from "@/lib/brand";
import { commands } from "@/lib/desktop/commands";
import type { BackendProfiles, ConnectionSecrets, ConnectionTarget } from "@/lib/desktop/connection";
import type { DesktopLaunch } from "@/lib/desktop/session";
import { BrandMark } from "@/shell/BrandMark";
import { useNativeSessionController } from "./context";
import { connectionError, startupDiagnostics } from "./errors";
import { StartupDiagnostics } from "./StartupDiagnostics";
import { BACKUP_RESET_LABEL, backupFailure, backupResetAvailable, backupSteps } from "./storage-backup-model";

/* === Constants === */

/** How often to poll `desktop_session_status` for startup progress while an
 * owned local backend is starting. `desktop_session_select` is one invoke
 * that only resolves once the whole backend is ready, so this poll is the
 * only way to see it move before then. */
const PHASE_POLL_INTERVAL_MS = 300;

/** The startup phases `dokime server --dev-control` reports, in the order it reports
 * them. The wire carries only the phase it is in, so the order lives here. */
const STARTUP_PHASES = ["storage", "plugins", "requests"] as const;

/** The steps a local storage reset reports. The shell reports the first three
 * itself; every backend startup phase after them belongs to "Starting". */
const RESET_PHASES = ["stopping", "deleting", "starting"] as const;

/* === Components === */

export function ConnectionScreen({ launch, launchError }: { launch: DesktopLaunch | null; launchError?: unknown }) {
  const profileId = useId();
  const caId = useId();
  const secretId = useId();
  const endpointId = useId();
  const identityId = useId();
  const tokenId = useId();
  const owner = useNativeSessionController();
  const state = useSyncExternalStore(owner.subscribe, owner.snapshot);
  const [profiles, setProfiles] = useState<BackendProfiles | null>(null);
  const [profile, setProfile] = useState("");
  const [profileError, setProfileError] = useState<string | null>(null);
  const [endpoint, setEndpoint] = useState("");
  const [identity, setIdentity] = useState("");
  const [secret, setSecret] = useState<"none" | "saved" | "bearer">("saved");
  const [token, setToken] = useState("");
  const [remember, setRemember] = useState(false);
  const [ca, setCa] = useState("");
  const [phase, setPhase] = useState<string | null>(null);
  const [archived, setArchived] = useState<number | null>(null);
  const [backupError, setBackupError] = useState<string | null>(null);
  const busy = state.phase === "connecting";
  const reset = state.phase === "connecting" && state.reset;
  const archive = state.phase === "connecting" ? state.archive : null;
  const error = state.phase === "failed" ? state.error : (launchError ?? launch?.error);
  const diagnostics = startupDiagnostics(error);
  const keptArchive = backupFailure(error)?.archive ?? null;
  const offerBackup = backupResetAvailable({
    localBackendAvailable: Boolean(launch?.localBackendAvailable),
    failedStart: diagnostics !== null,
    busy,
  });

  useEffect(() => {
    let active = true;
    Promise.all([commands.desktopProfilesList(), commands.desktopSessionLaunch()])
      .then(([document, configuration]) => {
        if (!active) return;
        setProfiles(document);
        const selected = state.selection ?? configuration.selection;
        setProfile(selected?.kind === "profile" ? selected.name : "");
      })
      .catch((error) => {
        if (active) setProfileError(error instanceof Error ? error.message : String(error));
      });
    return () => {
      active = false;
    };
  }, [state.selection]);

  useEffect(() => {
    if (!busy) {
      setPhase(null);
      setArchived(null);
      return;
    }
    let active = true;
    const poll = () => {
      commands
        .desktopSessionStatus()
        .then((status) => {
          if (!active) return;
          setPhase(status.phase);
          setArchived(status.archivedBytes ?? null);
        })
        // No backend is selected yet, or the native context has not settled:
        // either way, the static state sentence below still applies.
        .catch(() => {});
    };
    poll();
    const interval = window.setInterval(poll, PHASE_POLL_INTERVAL_MS);
    return () => {
      active = false;
      window.clearInterval(interval);
    };
  }, [busy]);

  // The shell names the archive, beside the data root or where the operator
  // picks; a cancelled pick leaves the failure and its diagnostics in place.
  async function backupReset(choose: boolean) {
    setBackupError(null);
    try {
      const destination = await commands.desktopStorageBackupDestination(choose);
      if (destination) void owner.backupResetLocalStorage(destination);
    } catch (error) {
      setBackupError(connectionError(error));
    }
  }

  function submit(event: FormEvent) {
    event.preventDefault();
    const value = endpoint.trim();
    const target: ConnectionTarget = { kind: "remote", endpoint: value, caCertificatePem: ca.trim() || null };
    const secrets: ConnectionSecrets = secret === "bearer" ? { kind: "bearer", token, remember } : { kind: secret };
    setToken("");
    void owner.select({
      kind: "attach",
      request: { target, secrets, expectedBackendId: identity.trim() || null },
    });
  }

  return (
    <main className="grid min-h-screen place-items-center px-6 py-12">
      <div className="grid w-full max-w-md justify-items-center gap-6 text-center">
        <BrandMark className="h-10" />
        {profiles && (profiles.profiles.length > 0 || profile) && (
          <div className="grid w-full gap-2 text-left">
            <label htmlFor={profileId}>Saved Backend</label>
            <select
              id={profileId}
              value={profile}
              disabled={busy}
              onChange={(event) => setProfile(event.target.value)}
              className="rounded-md border bg-background p-2"
            >
              <option value="">Select A Backend</option>
              {profile && !profiles.profiles.some((entry) => entry.name === profile) ? (
                <option value={profile}>{profile} (Unavailable)</option>
              ) : null}
              {profiles.profiles.map((entry) => (
                <option key={entry.name} value={entry.name}>
                  {entry.name}
                </option>
              ))}
            </select>
            <Button
              disabled={busy || !profile}
              onClick={() => {
                void owner.select({ kind: "profile", name: profile });
              }}
            >
              Connect To Saved Backend
            </Button>
          </div>
        )}
        {profileError && <p role="alert">{profileError}</p>}
        <h1 className="font-heading text-3xl leading-none font-semibold">{defaultBrand.label}</h1>

        <p className="text-sm text-muted-foreground" {...(busy ? { role: "status" } : {})}>
          {stateSentence({ busy, error: Boolean(error), launch, reset, archive })}
        </p>

        {busy ? (
          <>
            {archive ? (
              <>
                <BackupSteps archived={archived} current={phase} />
                <PathPill value={archive} />
              </>
            ) : reset ? (
              <ResetSteps current={phase} />
            ) : (
              <StartupSteps current={phase} />
            )}
            <div className="grid w-full gap-3">
              <Button className="w-full" onClick={() => void owner.disconnect()} variant="outline">
                Cancel
              </Button>
            </div>
          </>
        ) : (
          <>
            {Boolean(error) && (
              <p className="text-sm text-destructive" role="alert">
                {connectionError(error)}
              </p>
            )}
            {diagnostics && <StartupDiagnostics diagnostics={diagnostics} />}
            {keptArchive && (
              <div className="grid w-full grid-cols-[10rem_minmax(0,1fr)] items-center gap-3 text-left">
                <span className="text-sm text-muted-foreground">Backup Kept</span>
                <PathPill value={keptArchive} />
              </div>
            )}
            {offerBackup && (
              <div className="grid w-full gap-2">
                <div className="grid w-full grid-cols-2 gap-3">
                  <Button className="w-full" onClick={() => void backupReset(false)} variant="destructive">
                    {BACKUP_RESET_LABEL}
                  </Button>
                  <Button className="w-full" onClick={() => void backupReset(true)} variant="outline">
                    Choose Backup Location
                  </Button>
                </div>
                <p className="text-xs text-muted-foreground">
                  Archives the local data directory into a zip beside it, then empties it and starts the backend again.
                </p>
                {backupError && (
                  <p className="text-sm text-destructive" role="alert">
                    {backupError}
                  </p>
                )}
              </div>
            )}
            {!launch && !error ? (
              <p role="status">Loading desktop settings…</p>
            ) : (
              <>
                {launch ? (
                  <div className="grid w-full gap-2">
                    <Button
                      className="w-full"
                      disabled={!launch.localBackendAvailable}
                      onClick={() => void owner.select({ kind: "owned_local" })}
                    >
                      Start Local Backend
                    </Button>
                    <p className="text-xs text-muted-foreground">
                      {launch.localBackendAvailable
                        ? "This desktop owns the backend and closes it on exit."
                        : `No ${defaultBrand.label} backend was found beside this application. Reinstall ${defaultBrand.label} with the desktop installer, or attach to a backend below.`}
                    </p>
                  </div>
                ) : null}

                <form className="grid w-full gap-3 text-left" onSubmit={submit}>
                  <h2 className="text-center font-heading text-lg font-semibold">Attach To A Backend</h2>
                  <Field htmlFor={endpointId} label="Endpoint">
                    <Input
                      id={endpointId}
                      onChange={(event) => setEndpoint(event.target.value)}
                      placeholder="https://backend.example:50051"
                      required
                      type="url"
                      value={endpoint}
                    />
                  </Field>
                  <Field htmlFor={identityId} label="Expected Backend ID (Optional)">
                    <Input id={identityId} onChange={(event) => setIdentity(event.target.value)} value={identity} />
                  </Field>
                  <Field htmlFor={secretId} label="Secrets">
                    <select
                      className="h-[var(--control-height)] w-full rounded-lg border border-input bg-background px-3 text-sm"
                      id={secretId}
                      onChange={(event) => {
                        setSecret(event.target.value as typeof secret);
                        setToken("");
                      }}
                      value={secret}
                    >
                      <option value="saved">Saved in System Keyring</option>
                      <option value="bearer">Access Key</option>
                      <option value="none">No Secrets</option>
                    </select>
                  </Field>
                  {secret === "bearer" && (
                    <>
                      <Field htmlFor={tokenId} label="Access Key">
                        <Input
                          autoComplete="off"
                          id={tokenId}
                          onChange={(event) => setToken(event.target.value)}
                          required
                          type="password"
                          value={token}
                        />
                      </Field>
                      <label className="col-span-full flex items-center justify-center gap-2 text-sm">
                        <input checked={remember} onChange={(event) => setRemember(event.target.checked)} type="checkbox" />
                        Save Key in System Keyring
                      </label>
                    </>
                  )}
                  <details className="text-sm">
                    <summary className="cursor-pointer text-center text-muted-foreground">Custom TLS Authority</summary>
                    <div className="mt-3">
                      <Field htmlFor={caId} label="CA Certificate (PEM)">
                        <textarea
                          className="min-h-24 w-full rounded-lg border border-input bg-background p-2 font-mono text-xs"
                          id={caId}
                          onChange={(event) => setCa(event.target.value)}
                          value={ca}
                        />
                      </Field>
                    </div>
                  </details>
                  <p className="text-center text-xs text-muted-foreground">
                    HTTPS verifies the server certificate. Explicit HTTP sends keys and data without encryption; use it only in an isolated
                    lab. Choose No Secrets only when the listener does not require a key. Attached backends continue after disconnect.
                  </p>
                  <Button className="w-full" type="submit">
                    Connect
                  </Button>
                </form>
              </>
            )}
          </>
        )}
      </div>
    </main>
  );
}

/* === Private Components === */

/** One label/value row on the shared two-column grid. */
function Field({ children, htmlFor, label }: { children: ReactNode; htmlFor: string; label: string }) {
  return (
    <div className="grid grid-cols-[10rem_minmax(0,1fr)] items-center gap-3">
      <label className="text-sm text-muted-foreground" htmlFor={htmlFor}>
        {label}
      </label>
      {children}
    </div>
  );
}

/** The startup phases the backend reports, as a quiet vertical step list. */
function StartupSteps({ current }: { current: string | null }) {
  const phases: string[] = [...STARTUP_PHASES];
  // A phase this build does not know (a newer CLI) is shown where it is
  // rather than dropped, so the list never contradicts the backend.
  if (current && !phases.includes(current)) phases.push(current);
  const index = current ? phases.indexOf(current) : -1;

  return (
    <ol aria-label="Startup Phases" className="grid w-full gap-2 text-left">
      {phases.map((name, position) => (
        <Step active={position === index} done={index >= 0 && position < index} key={name} label={phaseLabel(name)} />
      ))}
    </ol>
  );
}

/** A local storage reset's steps: stop, delete, then start again. */
function ResetSteps({ current }: { current: string | null }) {
  const phases: readonly string[] = RESET_PHASES;
  // Any phase past the shell's own is the backend starting on the empty root.
  const index = current === null ? 0 : phases.includes(current) ? phases.indexOf(current) : 2;
  return (
    <ol aria-label="Reset Steps" className="grid w-full gap-2 text-left">
      {phases.map((name, position) => (
        <Step active={position === index} done={position < index} key={name} label={phaseLabel(name)} />
      ))}
    </ol>
  );
}

/** A backup reset's steps: stop, archive with its bytes written, reset, restart. */
function BackupSteps({ archived, current }: { archived: number | null; current: string | null }) {
  return (
    <ol aria-label="Backup Steps" className="grid w-full gap-2 text-left">
      {backupSteps(current, archived).map((step) => (
        <Step active={step.state === "active"} detail={step.detail} done={step.state === "done"} key={step.phase} label={step.label} />
      ))}
    </ol>
  );
}

/** One row of a vertical step list. */
function Step({ active, detail = null, done, label }: { active: boolean; detail?: string | null; done: boolean; label: string }) {
  return (
    <li
      className="grid grid-cols-[1.25rem_minmax(0,1fr)] items-center gap-3 text-sm"
      data-state={done ? "done" : active ? "active" : "waiting"}
    >
      <span className="grid size-5 place-items-center">
        {done ? (
          <Check aria-hidden className="text-primary" size={14} />
        ) : (
          <span aria-hidden className={active ? "size-2 rounded-full bg-primary" : "size-2 rounded-full bg-border"} />
        )}
      </span>
      <span className={active ? "font-medium text-foreground" : "text-muted-foreground"}>
        {label}
        {detail && <span className="ml-2 font-mono text-xs text-muted-foreground tabular-nums">{detail}</span>}
      </span>
    </li>
  );
}

/* === Private Functions === */

/** Human label for a `dokime server --dev-control` startup phase name. The wire value
 * is free-form — an unrecognized name (a newer CLI, an older GUI) still shows
 * something instead of nothing. */
function phaseLabel(phase: string): string {
  switch (phase) {
    case "storage":
      return "Opening Storage";
    case "plugins":
      return "Loading Plugins";
    case "requests":
      return "Accepting Requests";
    case "sidecars":
      return "Starting Sidecars";
    case "stopping":
      return "Stopping Backend";
    case "deleting":
      return "Deleting Local Storage";
    case "starting":
      return "Starting Backend";
    default:
      return phase;
  }
}

/** One sentence naming what the screen is doing. */
function stateSentence({
  busy,
  error,
  launch,
  reset,
  archive,
}: {
  busy: boolean;
  error: boolean;
  launch: DesktopLaunch | null;
  reset: boolean;
  archive: string | null;
}): string {
  if (archive) return "Backing up and resetting local storage.";
  if (reset) return "Resetting local storage.";
  if (busy) return "Starting the backend.";
  if (error) return "The last attempt did not connect.";
  if (!launch) return "Reading this desktop's settings.";
  return "Connect to a backend to plan and run cycles.";
}
