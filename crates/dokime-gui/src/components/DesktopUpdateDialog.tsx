/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { failureOf } from "@/lib/notification-details";
import { useEffect, useId, useState } from "react";
// Standard Library Imports
// Third Party Imports
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
  const modeId = useId();
  const sourceId = useId();
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
            <label htmlFor={modeId} className="grid gap-1">
              Update Checks
              <Select
                value={status.mode}
                disabled={busy}
                onValueChange={(value) => void perform(() => configure(value as UpdateMode, status.source))}
              >
                <SelectTrigger id={modeId} className="w-full" aria-label="Update Checks">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="disabled">Disabled</SelectItem>
                  <SelectItem value="manual">Manual</SelectItem>
                  <SelectItem value="automatic_check">Automatic Check</SelectItem>
                </SelectContent>
              </Select>
            </label>
            <label htmlFor={sourceId} className="grid gap-1">
              Release Source
              <Select
                value={status.source === null ? "none" : String(status.sources.indexOf(status.source))}
                disabled={busy}
                onValueChange={(value) =>
                  void perform(() => configure(status.mode, value === "none" ? null : status.sources[Number(value)]))
                }
              >
                <SelectTrigger id={sourceId} className="w-full" aria-label="Release Source">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="none" disabled={status.mode !== "disabled"}>
                    No Source
                  </SelectItem>
                  {status.sources.map((source, index) => (
                    <SelectItem key={source} value={String(index)}>
                      {source}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </label>
            {update ? (
              <div className="rounded border p-3">
                <div>
                  {update.status === "available"
                    ? `Update ${update.version} Available`
                    : update.status === "disabled"
                      ? "Updates Disabled"
                      : "Up to Date"}
                </div>
                <p className="text-muted-foreground">{update.body ?? `Current Version ${update.currentVersion}`}</p>
              </div>
            ) : null}
          </div>
        ) : null}
        {error ? <NotificationMessage error={error}>{failureOf(error, "Operation Failed").message}</NotificationMessage> : null}
        <DialogFooter>
          <Button
            data-testid="desktop-updates-check"
            variant="outline"
            disabled={busy || !status?.source || status.mode === "disabled"}
            onClick={() =>
              void perform(async () => {
                setUpdate(await commands.desktopUpdatesCheck());
              })
            }
          >
            Check for Updates
          </Button>
          <Button
            data-testid="desktop-updates-install"
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
