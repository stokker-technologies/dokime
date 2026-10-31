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

import { useNativeSessionOptional } from "@/app/native-session/context";
import { NotificationMessage } from "@/components/NotificationMessage";
import { SettingsCard } from "../SettingsCard";
import { SettingsPathField } from "../SettingsPathField";
import { SettingsRow } from "../SettingsRow";
import { useSettingsFrameQuery } from "../settings-queries";

/* === Types === */

/* === Components === */

export function BootstrapPaths() {
  const session = useNativeSessionOptional();
  const query = useSettingsFrameQuery(undefined, session !== null);
  const paths = query.data?.bootstrap_paths;
  return (
    <SettingsCard title="Bootstrap Paths" description="Paths are selected at startup.">
      {!session && <p className="text-sm text-muted-foreground">Connect to view backend paths.</p>}
      {session && query.isPending && (
        <p role="status" className="text-sm text-muted-foreground">
          Loading Paths…
        </p>
      )}
      {Boolean(query.error) && <NotificationMessage error={query.error}>{String(query.error)}</NotificationMessage>}
      {paths &&
        Object.entries(paths).map(([key, path]) => (
          <SettingsRow
            key={key}
            label={
              { application: "Application Directory", storage: "Data Directory", plugins: "Plugins Directory", logs: "Logs Directory" }[key]
            }
          >
            <SettingsPathField value={path} openable={session?.snapshot.ownership === "owned_local"} />
          </SettingsRow>
        ))}
    </SettingsCard>
  );
}

/* === Hooks === */

/* === Public Functions === */

/* === Private Functions === */
