/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Connection Selection And Local Startup Recovery. */

/* === External Imports === */

import { listen } from "@tauri-apps/api/event";
import { Check } from "lucide-react";
import { useEffect, useRef, useState, useSyncExternalStore } from "react";

/* === Local Imports === */

import { NotificationMessage } from "@/components/NotificationMessage";
import { PathPill } from "@/components/PathPill";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { BackendsTable } from "@/features/settings/BackendsTable";
import { DisconnectedSettings } from "@/features/settings/SettingsSurface";
import { defaultBrand } from "@/lib/brand";
import { commands } from "@/lib/desktop/commands";
import {
  DESKTOP_SESSION_STATUS_EVENT,
  type DesktopLaunch,
  type DesktopSessionSnapshot,
  type StorageBackupDestination,
} from "@/lib/desktop/session";
import { BrandMark } from "@/shell/BrandMark";
import { useNativeSessionController } from "./context";
import { connectionError, startupDiagnostics, storageRecoveryArchive, storageRecoveryAvailable } from "./errors";

/* === Constants === */

/** The startup phases `dokime server --dev-control` reports, in the order it reports
 * them. The wire carries only the phase it is in, so the order lives here. */
const STARTUP_PHASES = ["storage", "plugins", "requests"] as const;

/** The shell's reset phases; backend startup phases belong to Starting. */
const RESET_PHASES = ["stopping", "archiving", "deleting", "reinstalling", "starting"] as const;

/* === Components === */

export function ConnectionScreen({ launch, launchError }: { launch: DesktopLaunch | null; launchError?: unknown }) {
  const owner = useNativeSessionController();
  const state = useSyncExternalStore(owner.subscribe, owner.snapshot);
  const [settings, setSettings] = useState(false);
  const [phase, setPhase] = useState<string | null>(null);
  const [archivedBytes, setArchivedBytes] = useState<number | null>(null);
  const [backup, setBackup] = useState<StorageBackupDestination | null>(null);
  const [backupError, setBackupError] = useState<unknown>(null);
  const [preparingBackup, setPreparingBackup] = useState(false);
  const statusRevision = useRef(-1);
  const localAttempt = useRef(false);
  const [failedEndpoint, setFailedEndpoint] = useState<string | null>(null);
  const busy = state.phase === "connecting";
  const reset = state.phase === "connecting" && state.reset !== false;
  const backingUp = state.phase === "connecting" && state.reset === "backup";
  const error = state.phase === "failed" ? state.error : (launchError ?? launch?.error);
  const diagnostics = startupDiagnostics(error);
  const recoverable = storageRecoveryAvailable(error, localAttempt.current, launch?.localBackendAvailable ?? false);
  const archive = storageRecoveryArchive(error);

  async function prepareBackup() {
    setPreparingBackup(true);
    setBackupError(null);
    try {
      setBackup(await commands.desktopStorageBackupDestination());
    } catch (failure) {
      setBackupError(failure);
    } finally {
      setPreparingBackup(false);
    }
  }
  useEffect(() => {
    if (!error) return;
    let active = true;
    void commands
      .desktopSessionStatus()
      .then((status) => {
        if (active) setFailedEndpoint(status.connection.endpoint);
      })
      .catch(() => {});
    return () => {
      active = false;
    };
  }, [error]);

  useEffect(() => {
    if (!busy) {
      setPhase(null);
      return;
    }
    let active = true;
    let unlisten: (() => void) | undefined;
    const applyStatus = (status: DesktopSessionSnapshot) => {
      if (!active || status.statusRevision < statusRevision.current) return;
      statusRevision.current = status.statusRevision;
      setPhase(status.phase);
      setArchivedBytes(status.archivedBytes ?? null);
    };
    const readCurrentStatus = () =>
      commands
        .desktopSessionStatus()
        .then(applyStatus)
        .catch(() => {});
    void listen<DesktopSessionSnapshot>(DESKTOP_SESSION_STATUS_EVENT, ({ payload }) => applyStatus(payload))
      .then((dispose) => {
        if (!active) {
          dispose();
          return;
        }
        unlisten = dispose;
        return readCurrentStatus();
      })
      .catch(() => {});
    return () => {
      active = false;
      unlisten?.();
    };
  }, [busy]);

  if (settings) return <DisconnectedSettings close={() => setSettings(false)} />;

  return (
    <main className="flex min-h-screen items-center justify-center px-4 py-12 sm:px-6">
      <div className="grid w-full max-w-140 min-w-0 gap-6 text-center">
        <header className="grid justify-items-center gap-4">
          <BrandMark className="connection-brand brand-column" heading />

          <p className="text-sm text-muted-foreground" {...(busy ? { role: "status" } : {})}>
            {stateSentence({ busy, error: Boolean(error), launch, reset })}
          </p>
        </header>

        {busy && (
          <>
            {reset ? <ResetSteps current={phase} archivedBytes={archivedBytes} backup={backingUp} /> : <StartupSteps current={phase} />}
            <div className="grid w-full gap-3">
              <Button className="w-full" disabled={reset} onClick={() => void owner.disconnect()} variant="outline">
                Cancel
              </Button>
            </div>
          </>
        )}
        <div className={busy ? "hidden" : "grid gap-6"}>
          {!launch && !error ? (
            <p role="status">Loading desktop settings…</p>
          ) : (
            <>
              {launch ? (
                <div className="grid w-full gap-2">
                  <Button
                    className="w-full"
                    size="lg"
                    disabled={!launch.localBackendAvailable}
                    onClick={() => {
                      localAttempt.current = true;
                      void owner.select({ kind: "owned_local" });
                    }}
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

              <div className="flex items-center gap-3 text-xs text-muted-foreground">
                <span className="h-px flex-1 bg-border" />
                <span>Or Connect To A Saved Backend</span>
                <span className="h-px flex-1 bg-border" />
              </div>
              <BackendsTable
                failure={error && !diagnostics ? <p>{connectionError(error)}</p> : undefined}
                endpoint={failedEndpoint ?? launch?.target?.endpoint}
                onConnect={() => {
                  localAttempt.current = false;
                }}
                retry={(profile) => {
                  if (localAttempt.current || diagnostics) {
                    void owner.select({ kind: "owned_local" });
                    return;
                  }
                  if (profile) {
                    void owner.select({ kind: "profile", identifier: profile });
                    return;
                  }
                  if (launch?.target)
                    void owner.select({
                      kind: "attach",
                      request: {
                        target: launch.target,
                        expectedBackendId: launch.expectedBackendId,
                        secrets: { kind: launch.target.kind === "remote" ? "saved" : "none" },
                      },
                    });
                  else if (launch?.localBackendAvailable) void owner.select({ kind: "owned_local" });
                }}
              />
            </>
          )}
        </div>
        {!busy && diagnostics && (
          <div className="grid gap-3 text-left text-sm">
            <NotificationMessage context="background" source="Startup" title="Backend Could Not Start" error={error}>
              {diagnostics.cause || connectionError(error)}
            </NotificationMessage>
            {diagnostics.log && <PathPill value={diagnostics.log} />}
            <Button variant="outline" onClick={() => void owner.select({ kind: "owned_local" })}>
              Retry
            </Button>
          </div>
        )}
        {!busy && archive && <PathPill value={archive} />}
        {!busy && recoverable && (
          <div className="grid gap-3">
            <Button variant="destructive" disabled={preparingBackup} onClick={() => void prepareBackup()} data-action="backup-reset">
              Back Up And Reset
            </Button>
            {backupError ? <p role="alert">{connectionError(backupError)}</p> : null}
          </div>
        )}
        <Dialog
          open={backup !== null && !busy && recoverable}
          onOpenChange={(open) => {
            if (!open) setBackup(null);
          }}
        >
          <DialogContent className="sm:max-w-lg">
            <DialogHeader>
              <DialogTitle>Back Up And Reset</DialogTitle>
              <DialogDescription>
                Back up the data folder to this archive, then remove the folder's contents and restart the local backend. Installed bundles
                will be restored with readable enabled states and valid settings. If saved plugin preferences cannot be recovered, bundles
                are enabled with default settings. Credentials remain in secret storage. If the backup fails, nothing is removed.
              </DialogDescription>
            </DialogHeader>
            {backup && (
              <div className="grid gap-3 text-left text-sm">
                <div className="grid gap-1">
                  <span>Data Folder</span>
                  <PathPill value={backup.directory} />
                </div>
                <div className="grid gap-1">
                  <span>Backup Archive</span>
                  <PathPill value={backup.archive} />
                </div>
              </div>
            )}
            <DialogFooter>
              <Button variant="outline" onClick={() => setBackup(null)}>
                Cancel
              </Button>
              <Button
                variant="destructive"
                data-action="confirm-backup-reset"
                onClick={() => {
                  if (!backup) return;
                  const confirmed = backup;
                  setBackup(null);
                  localAttempt.current = true;
                  void owner.backupAndResetLocalStorage(confirmed.directory, confirmed.archive);
                }}
              >
                Back Up And Reset
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
        <footer>
          <Button variant="link" disabled={busy} onClick={() => setSettings(true)}>
            Settings
          </Button>
        </footer>
      </div>
    </main>
  );
}

/* === Private Components === */

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

/** An explicit reset reports backup and installation restoration before startup. */
function ResetSteps({ current, archivedBytes, backup }: { current: string | null; archivedBytes: number | null; backup: boolean }) {
  const phases: readonly string[] = RESET_PHASES.filter((phase) => backup || phase !== "archiving");
  // Backend phases follow the shell's reset work.
  const index = current === null ? 0 : phases.includes(current) ? phases.indexOf(current) : phases.length - 1;
  return (
    <ol aria-label="Reset Steps" className="grid w-full gap-2 text-left">
      {phases.map((name, position) => (
        <Step
          active={position === index}
          done={position < index}
          key={name}
          label={phaseLabel(name)}
          detail={name === "archiving" && archivedBytes !== null ? `${archivedBytes.toLocaleString()} B` : null}
        />
      ))}
    </ol>
  );
}

/** One row of a vertical step list. */
function Step({ active, detail = null, done, label }: { active: boolean; detail?: string | null; done: boolean; label: string }) {
  return (
    <li
      className="grid grid-cols-[--spacing(5)_minmax(0,1fr)] items-center gap-3 text-sm"
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
    case "archiving":
      return "Backing Up Local Storage";
    case "deleting":
      return "Deleting Local Storage";
    case "starting":
      return "Starting Backend";
    case "reinstalling":
      return "Reinstalling Plugins";
    default:
      return "Starting Backend";
  }
}

/** One sentence naming what the screen is doing. */
function stateSentence({
  busy,
  error,
  launch,
  reset,
}: {
  busy: boolean;
  error: boolean;
  launch: DesktopLaunch | null;
  reset: boolean;
}): string {
  if (reset) return "Resetting local storage.";
  if (busy) return "Starting the backend.";
  if (error) return "Connect to a backend to plan and run cycles.";
  if (!launch) return "Reading this desktop's settings.";
  return "Connect to a backend to plan and run cycles.";
}
