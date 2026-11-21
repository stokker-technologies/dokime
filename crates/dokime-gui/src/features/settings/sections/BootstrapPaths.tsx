/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

/* === Local Imports === */

import { ErrorCard } from "@dokime/plots/components/ErrorCard";
import { failureOf } from "@dokime/plots/lib/notification-details";
import { useNativeSessionOptional } from "@/app/native-session/context";
import { SettingsCard } from "../SettingsCard";
import { SettingsPathField } from "../SettingsPathField";
import { SettingsRow } from "../SettingsRow";
import { useSettingsFrameQuery } from "../settings-queries";

/* === Types === */

/* === Components === */

export function BootstrapPaths() {
  const session = useNativeSessionOptional();
  const query = useSettingsFrameQuery(undefined, session !== null);
  const paths = query.data?.bootstrapPaths;
  return (
    <SettingsCard title="Bootstrap Paths" description="Paths are selected at startup.">
      {!session && <p className="text-sm text-muted-foreground">Connect to view backend paths.</p>}
      {session && query.isPending && (
        <p role="status" className="text-sm text-muted-foreground">
          Loading Paths…
        </p>
      )}
      {Boolean(query.error) && <ErrorCard failure={failureOf(query.error, "Data Unavailable")} />}
      {paths &&
        Object.entries({
          application: "Application Directory",
          storage: "Data Directory",
          plugins: "Plugins Directory",
          logs: "Logs Directory",
        }).map(([key, label]) => (
          <SettingsRow key={key} label={label}>
            <SettingsPathField
              value={paths[key as "application" | "storage" | "plugins" | "logs"]}
              openable={session?.snapshot.ownership === "owned_local"}
            />
          </SettingsRow>
        ))}
    </SettingsCard>
  );
}

/* === Hooks === */

/* === Public Functions === */

/* === Private Functions === */
