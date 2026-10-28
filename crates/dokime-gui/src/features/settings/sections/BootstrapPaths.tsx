/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import { useNativeSessionOptional } from "@/app/native-session/context";
import { SettingsCard } from "../SettingsCard";
import { SettingsPathField } from "../SettingsPathField";
import { SettingsRow } from "../SettingsRow";
import { useSettingsFrameQuery } from "../settings-queries";

export function BootstrapPaths() {
  const session = useNativeSessionOptional();
  const query = useSettingsFrameQuery(undefined, session !== null);
  const paths = query.data?.bootstrap_paths;
  return (
    <SettingsCard title="Bootstrap Paths" description="Paths are selected at startup.">
      {!session && <p role="status">Connect to view backend paths.</p>}
      {session && query.isPending && <p role="status">Loading Paths…</p>}
      {Boolean(query.error) && <p role="alert">{String(query.error)}</p>}
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
