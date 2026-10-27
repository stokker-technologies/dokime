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

import { Check } from "lucide-react";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef, useState, useSyncExternalStore } from "react";

/* === Local Imports === */

import { PathPill } from "@/components/PathPill";
import { Button } from "@/components/ui/button";
import { DisconnectedSettings } from "@/features/settings/SettingsSurface";
import { BackendsTable } from "@/features/settings/BackendsTable";
import { defaultBrand } from "@/lib/brand";
import { commands } from "@/lib/desktop/commands";
import { DESKTOP_SESSION_STATUS_EVENT, type DesktopLaunch, type DesktopSessionSnapshot } from "@/lib/desktop/session";
import { BrandMark } from "@/shell/BrandMark";
import { useNativeSessionController } from "./context";
import { connectionError, startupDiagnostics } from "./errors";

/* === Constants === */

/** The startup phases `dokime server --dev-control` reports, in the order it reports
 * them. The wire carries only the phase it is in, so the order lives here. */
const STARTUP_PHASES = ["storage", "plugins", "requests"] as const;

/** The steps a local storage reset reports. The shell reports the first three
 * itself; every backend startup phase after them belongs to "Starting". */
const RESET_PHASES = ["stopping", "deleting", "starting"] as const;

/* === Components === */

export function ConnectionScreen({ launch, launchError }: { launch: DesktopLaunch | null; launchError?: unknown }) {
  const owner = useNativeSessionController();
  const state = useSyncExternalStore(owner.subscribe, owner.snapshot);
  const [settings, setSettings] = useState(false);
  const [phase, setPhase] = useState<string | null>(null);
  const statusRevision = useRef(-1);
  const localAttempt = useRef(false);
  const [failedEndpoint, setFailedEndpoint] = useState<string | null>(null);
  const busy = state.phase === "connecting";
  const reset = state.phase === "connecting" && state.reset;
  const error = state.phase === "failed" ? state.error : (launchError ?? launch?.error);
  const diagnostics = startupDiagnostics(error);
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
          <BrandMark className="h-10" />
          <h1 className="font-heading text-3xl leading-none font-semibold">{defaultBrand.label}</h1>

          <p className="text-sm text-muted-foreground" {...(busy ? { role: "status" } : {})}>
            {stateSentence({ busy, error: Boolean(error), launch, reset })}
          </p>
        </header>

        {busy && (
          <>
            {reset ? <ResetSteps current={phase} /> : <StartupSteps current={phase} />}
            <div className="grid w-full gap-3">
              <Button className="w-full" onClick={() => void owner.disconnect()} variant="outline">
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
                <span>or connect to a saved backend</span>
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
                    void owner.select({ kind: "profile", name: profile });
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
          <div role="alert" className="grid gap-3 rounded-lg border border-destructive/30 bg-destructive/5 p-4 text-left text-sm">
            <p className="whitespace-pre-wrap break-words">{diagnostics.cause || connectionError(error)}</p>
            {diagnostics.log && <PathPill value={diagnostics.log} />}
            <Button variant="outline" onClick={() => void owner.select({ kind: "owned_local" })}>
              Retry
            </Button>
          </div>
        )}
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
    case "deleting":
      return "Deleting Local Storage";
    case "starting":
      return "Starting Backend";
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
