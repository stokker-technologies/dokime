/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { LogIn, RefreshCw } from "lucide-react";
import { useEffect, useState } from "react";

/* === Local Imports === */

import { DesktopUpdateDialog } from "@/components/DesktopUpdateDialog";
import { Button } from "@/components/ui/button";
import { SettingsCard } from "@/features/settings/SettingsCard";
import { SettingsRow } from "@/features/settings/SettingsRow";
import { commands } from "@/lib/desktop/commands";
import type { UpdateStatus } from "@/lib/desktop/updates";
import { isTauriDesktop } from "@/lib/desktop-open";

/* === Components === */

export function AccountUpdatesSection() {
  const desktop = isTauriDesktop();
  const [auth, setAuth] = useState<UpdateStatus | null>(null);
  const [dialogOpen, setDialogOpen] = useState(false);

  useEffect(() => {
    if (!desktop || dialogOpen) {
      return;
    }
    let active = true;
    // Re-reads when the manage/updates dialog closes, so a sign-in there is reflected.
    commands
      .desktopUpdatesStatus()
      .then((status) => {
        if (active) {
          setAuth(status);
        }
      })
      .catch(() => {
        if (active) {
          setAuth(null);
        }
      });
    return () => {
      active = false;
    };
  }, [desktop, dialogOpen]);

  const accountLabel = !desktop
    ? "Available in the desktop app."
    : auth?.user
      ? `Signed in as ${auth.user?.name ?? auth.user?.username ?? "GitLab user"}.`
      : "Public sources do not require sign-in.";
  const modeLabel =
    auth?.mode === "automatic_check"
      ? "Automatic Check"
      : auth?.mode === "manual"
        ? "Manual"
        : auth?.mode === "disabled"
          ? "Disabled"
          : "Unavailable";

  return (
    <>
      <SettingsCard title="Account">
        <SettingsRow description={accountLabel} label={auth?.host ?? "Release Account"}>
          <Button disabled={!desktop} onClick={() => setDialogOpen(true)} size="sm" variant="outline">
            <LogIn />
            Manage
          </Button>
        </SettingsRow>
      </SettingsCard>

      <SettingsCard title="Updates">
        <SettingsRow label="Current Version">
          <span className="font-mono text-sm">v{__APP_VERSION__}</span>
        </SettingsRow>
        <SettingsRow description={auth?.error ?? `Source: ${auth?.source ?? "None"}. Checks: ${modeLabel}.`} label="Updates">
          <Button disabled={!desktop} onClick={() => setDialogOpen(true)} size="sm" variant="outline">
            <RefreshCw />
            Check For Updates
          </Button>
        </SettingsRow>
      </SettingsCard>

      {!desktop ? <p className="text-xs text-muted-foreground">Sign-in and desktop updates are available in the desktop app.</p> : null}

      <DesktopUpdateDialog onOpenChange={setDialogOpen} open={dialogOpen} />
    </>
  );
}
