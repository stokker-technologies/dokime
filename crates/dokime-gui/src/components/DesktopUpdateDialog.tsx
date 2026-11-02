/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

import { useEffect, useState } from "react";
// Standard Library Imports
// Third Party Imports
import { notificationMessage } from "@/lib/notification-details";
import { NotificationMessage } from "@/components/NotificationMessage";

// Local Imports
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { commands } from "@/lib/desktop/commands";
import type { UpdateCheck, UpdateMode, UpdateStatus } from "@/lib/desktop/updates";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

type DesktopUpdateDialogProps = { open: boolean; onOpenChange: (open: boolean) => void };

/* === Components === */

export function DesktopUpdateDialog({ open, onOpenChange }: DesktopUpdateDialogProps) {
  const [status, setStatus] = useState<UpdateStatus | null>(null);
  const [update, setUpdate] = useState<UpdateCheck | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<unknown>(null);

  useEffect(() => {
    if (!open) return;
    let active = true;
    void commands
      .desktopUpdatesStatus()
      .then((next) => {
        if (!active) return;
        setStatus(next);
        setUpdate(next.lastCheck);
        setError(next.error);
      })
      .catch((reason: unknown) => {
        if (active) setError(reason);
      });
    return () => {
      active = false;
    };
  }, [open]);

  async function perform(operation: () => Promise<void>) {
    setBusy(true);
    setError(null);
    try {
      await operation();
    } catch (reason) {
      setError(reason);
    } finally {
      setBusy(false);
    }
  }

  async function configure(mode: UpdateMode, source: string | null) {
    const next = await commands.desktopUpdatesConfigure({ mode, source });
    setStatus(next);
    setUpdate(null);
    setError(next.error);
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>Desktop Updates</DialogTitle>
          <DialogDescription>Choose a release source. Installation always requires your confirmation.</DialogDescription>
        </DialogHeader>
        {status ? (
          <div className="grid gap-4 text-sm">
            <label className="grid gap-1">
              Update Checks
              <select
                aria-label="Update Checks"
                className="rounded border bg-background p-2"
                value={status.mode}
                disabled={busy}
                onChange={(event) => void perform(() => configure(event.target.value as UpdateMode, status.source))}
              >
                <option value="disabled">Disabled</option>
                <option value="manual">Manual</option>
                <option value="automatic_check">Automatic Check</option>
              </select>
            </label>
            <label className="grid gap-1">
              Release Source
              <select
                aria-label="Release Source"
                className="rounded border bg-background p-2"
                value={status.source ?? ""}
                disabled={busy}
                onChange={(event) => void perform(() => configure(status.mode, event.target.value || null))}
              >
                <option value="" disabled={status.mode !== "disabled"}>
                  No Source
                </option>
                {status.sources.map((source) => (
                  <option key={source} value={source}>
                    {source}
                  </option>
                ))}
              </select>
            </label>
            {update ? (
              <div className="rounded border p-3">
                <div>
                  {update.status === "available"
                    ? `Update ${update.version} Available`
                    : update.status === "disabled"
                      ? "Updates Disabled"
                      : "Up To Date"}
                </div>
                <p className="text-muted-foreground">{update.body ?? `Current Version ${update.currentVersion}`}</p>
              </div>
            ) : null}
          </div>
        ) : null}
        {error ? <NotificationMessage error={error}>{notificationMessage(error)}</NotificationMessage> : null}
        <DialogFooter>
          <Button
            variant="outline"
            disabled={busy || !status?.source || status.mode === "disabled"}
            onClick={() =>
              void perform(async () => {
                setUpdate(await commands.desktopUpdatesCheck());
              })
            }
          >
            Check For Updates
          </Button>
          <Button
            disabled={busy || !update?.candidate}
            onClick={() =>
              void perform(async () => {
                if (!update?.candidate) return;
                await commands.desktopUpdatesInstall(update.candidate);
                setUpdate(null);
              })
            }
          >
            Install Update
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

/* === Hooks === */

/* === Public Functions === */

/* === Private Functions === */
