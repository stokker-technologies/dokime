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
import { isTauriDesktop } from "@/lib/desktop-open";
import { type GitlabAuthStatus, gitlabAuthStatus } from "@/lib/desktop-updates";

/* === Components === */

export function AccountUpdatesSection() {
  const desktop = isTauriDesktop();
  const [auth, setAuth] = useState<GitlabAuthStatus | null>(null);
  const [dialogOpen, setDialogOpen] = useState(false);

  useEffect(() => {
    if (!desktop) {
      return;
    }
    let active = true;
    // Re-reads when the manage/updates dialog closes, so a sign-in there is reflected.
    gitlabAuthStatus()
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
  }, [desktop]);

  const accountLabel = !desktop
    ? "Available in the desktop app."
    : auth?.signedIn
      ? `Signed in as ${auth.user?.name ?? auth.user?.username ?? "GitLab user"}.`
      : "Not signed in.";

  return (
    <>
      <SettingsCard title="Account">
        <SettingsRow description={accountLabel} label="GitLab">
          <Button
            disabled={!desktop}
            onClick={() => setDialogOpen(true)}
            size="sm"
            variant="outline"
          >
            <LogIn />
            {auth?.signedIn ? "Manage" : "Sign In"}
          </Button>
        </SettingsRow>
      </SettingsCard>

      <SettingsCard title="Updates">
        <SettingsRow label="Current Version">
          <span className="font-mono text-sm">v{__APP_VERSION__}</span>
        </SettingsRow>
        <SettingsRow description="Check GitLab releases for a newer desktop build." label="Updates">
          <Button
            disabled={!desktop}
            onClick={() => setDialogOpen(true)}
            size="sm"
            variant="outline"
          >
            <RefreshCw />
            Check For Updates
          </Button>
        </SettingsRow>
      </SettingsCard>

      {!desktop ? (
        <p className="text-xs text-muted-foreground">
          Sign-in and desktop updates are available in the desktop app.
        </p>
      ) : null}

      <DesktopUpdateDialog onOpenChange={setDialogOpen} open={dialogOpen} />
    </>
  );
}
