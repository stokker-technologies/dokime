/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Optional Host Firewall Permission
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
import { useQuery } from "@tanstack/react-query";
import { useState } from "react";

// Local Imports
import { Button } from "@/components/ui/button";
import { SettingsCard } from "@/features/settings/SettingsCard";
import { useBrand } from "@/lib/brand";
import { commands } from "@/lib/desktop/commands";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

type Props = { generation: string; listenerAddress?: string; enabled: boolean };

/* === Components === */

export function FirewallNotice({ generation, listenerAddress, enabled }: Props) {
  const { label } = useBrand();
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const query = useQuery({
    queryKey: ["settings", "firewall", generation, listenerAddress, enabled],
    queryFn: () => commands.desktopFirewall(generation),
    retry: false,
    refetchOnWindowFocus: true,
  });
  const required = query.data?.required ?? false;
  if (!required && !message && !query.data?.message && !query.error) return null;
  return (
    <SettingsCard title="Windows Firewall">
      <div className="grid gap-3 p-4">
        {required && (
          <p role="status" className="text-sm text-muted-foreground">
            Allow {label} through Windows Firewall for PCs on the same subnet. This needs an administrator.
          </p>
        )}
        {(message || query.data?.message || query.error) && (
          <p role="status" className="text-sm text-muted-foreground">
            {message ||
              query.data?.message ||
              "Windows Firewall access could not be checked. Ask an administrator to check inbound access on the listener's networks."}
          </p>
        )}
        {required && (
          <Button
            variant="outline"
            disabled={busy}
            onClick={() => {
              setBusy(true);
              setMessage(null);
              void commands
                .desktopFirewall(generation, true)
                .then((status) => {
                  setMessage(
                    status.message ??
                      (status.required
                        ? "Windows Firewall still blocks access. Ask an administrator to check the listener's network policy."
                        : "Windows Firewall allows remote access for PCs on the same subnet."),
                  );
                  void query.refetch();
                })
                .catch(() =>
                  setMessage(
                    "Windows Firewall was not changed. Administrator approval was cancelled or the change was refused. You can keep using the app.",
                  ),
                )
                .finally(() => setBusy(false));
            }}
          >
            Allow In Firewall…
          </Button>
        )}
      </div>
    </SettingsCard>
  );
}

/* === Hooks === */

/* === Public Functions === */

/* === Private Functions === */
