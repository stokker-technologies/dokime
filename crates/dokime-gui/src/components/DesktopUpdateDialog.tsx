/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { CheckCircle2, Download, LogIn, LogOut, RefreshCw, Settings } from "lucide-react";
import { useEffect, useMemo, useState } from "react";

/* === Local Imports === */

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { useBrand } from "@/lib/brand";
import {
  type GitlabAuthStatus,
  type GitlabUpdateCheck,
  gitlabAuthStatus,
  gitlabCheckForUpdates,
  gitlabInstallUpdate,
  gitlabSignIn,
  gitlabSignOut,
} from "@/lib/desktop-updates";

/* === Types === */

type DesktopUpdateDialogProps = {
  open: boolean;
  onOpenChange: (open: boolean) => void;
};

type BusyAction = "checking" | "installing" | "signing-in" | "signing-out" | null;

/* === Components === */

export function DesktopUpdateDialog({ open, onOpenChange }: DesktopUpdateDialogProps) {
  const brand = useBrand();
  const [auth, setAuth] = useState<GitlabAuthStatus | null>(null);
  const [busy, setBusy] = useState<BusyAction>(null);
  const [error, setError] = useState<string | null>(null);
  const [update, setUpdate] = useState<GitlabUpdateCheck | null>(null);

  useEffect(() => {
    if (!open) {
      return;
    }
    let active = true;
    setError(null);
    void gitlabAuthStatus()
      .then((status) => {
        if (active) {
          setAuth(status);
        }
      })
      .catch((reason: unknown) => {
        if (active) {
          setError(renderError(reason));
        }
      });
    return () => {
      active = false;
    };
  }, [open]);

  const statusLabel = useMemo(() => {
    if (!auth?.configured) {
      return "Not Configured";
    }
    if (auth.signedIn) {
      return "Signed In";
    }
    return "Signed Out";
  }, [auth]);

  const canCheck = auth?.configured && auth.signedIn && busy === null;
  const canInstall = update?.status === "available" && busy === null;

  async function handleSignIn() {
    setBusy("signing-in");
    setError(null);
    try {
      const status = await gitlabSignIn();
      setAuth(status);
      setUpdate(null);
    } catch (reason) {
      setError(renderError(reason));
    } finally {
      setBusy(null);
    }
  }

  async function handleSignOut() {
    setBusy("signing-out");
    setError(null);
    try {
      const status = await gitlabSignOut();
      setAuth(status);
      setUpdate(null);
    } catch (reason) {
      setError(renderError(reason));
    } finally {
      setBusy(null);
    }
  }

  async function handleCheckForUpdates() {
    setBusy("checking");
    setError(null);
    try {
      setUpdate(await gitlabCheckForUpdates());
    } catch (reason) {
      setError(renderError(reason));
    } finally {
      setBusy(null);
    }
  }

  async function handleInstallUpdate() {
    setBusy("installing");
    setError(null);
    try {
      await gitlabInstallUpdate();
    } catch (reason) {
      setError(renderError(reason));
      setBusy(null);
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>GitLab Updates</DialogTitle>
          <DialogDescription>Sign in to check private {brand.label} releases.</DialogDescription>
        </DialogHeader>

        <div className="grid gap-3 text-sm">
          <div className="flex items-center justify-between gap-3 rounded-md border bg-muted/25 px-3 py-2">
            <div className="min-w-0">
              <div className="font-medium">GitLab</div>
              <div className="truncate text-xs text-muted-foreground">
                {auth?.user ? `${auth.user.name} · ${auth.user.username}` : (auth?.message ?? "No active sign-in")}
              </div>
            </div>
            <Badge variant={auth?.signedIn ? "secondary" : "outline"}>{statusLabel}</Badge>
          </div>

          {update ? (
            <div className="grid gap-1 rounded-md border px-3 py-2">
              <div className="flex items-center gap-2 font-medium">
                {update.status === "available" ? <Download className="size-4" /> : <CheckCircle2 className="size-4" />}
                {updateTitle(update)}
              </div>
              <div className="text-xs text-muted-foreground">
                {update.message ?? update.body ?? `Current version ${update.currentVersion}`}
              </div>
            </div>
          ) : null}

          {error ? (
            <div className="rounded-md border border-destructive/35 bg-destructive/5 px-3 py-2 text-xs text-destructive">{error}</div>
          ) : null}
        </div>

        <DialogFooter className="gap-2 sm:justify-between">
          <div className="flex flex-wrap gap-2">
            {auth?.signedIn ? (
              <Button disabled={busy !== null} onClick={() => void handleSignOut()} type="button" variant="outline">
                <LogOut data-icon="inline-start" />
                Sign Out
              </Button>
            ) : (
              <Button disabled={!auth?.configured || busy !== null} onClick={() => void handleSignIn()} type="button" variant="outline">
                <LogIn data-icon="inline-start" />
                Sign In With GitLab
              </Button>
            )}
          </div>
          <div className="flex flex-wrap gap-2">
            <Button disabled={!canCheck} onClick={() => void handleCheckForUpdates()} type="button" variant="outline">
              <RefreshCw data-icon="inline-start" />
              Check For Updates
            </Button>
            <Button disabled={!canInstall} onClick={() => void handleInstallUpdate()} type="button">
              <Settings data-icon="inline-start" />
              Install Update
            </Button>
          </div>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

function updateTitle(update: GitlabUpdateCheck): string {
  if (update.status === "available") {
    return `Update ${update.version ?? ""} Available`.trim();
  }
  if (update.status === "signed_out") {
    return "Sign In Required";
  }
  if (update.status === "not_configured") {
    return "OAuth Setup Required";
  }
  return "Up To Date";
}

function renderError(reason: unknown): string {
  if (reason instanceof Error) {
    return reason.message;
  }
  return String(reason);
}
