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
import { useDiagnosticsQuery } from "@/features/cycle/server-state";
import { SettingsCard } from "../SettingsCard";
import { SettingsPathField } from "../SettingsPathField";
import { SettingsRow } from "../SettingsRow";

/* === Types === */

/* === Components === */

export function BootstrapPaths() {
  const session = useNativeSessionOptional();
  const diagnostics = useDiagnosticsQuery();
  const runtime = diagnostics.data?.runtime;
  return (
    <SettingsCard title="Bootstrap Paths" description="Paths are selected at startup.">
      {!session && <p role="status">Connect to view backend paths.</p>}
      {session && diagnostics.isPending && <p role="status">Loading Paths…</p>}
      {Boolean(diagnostics.error) && <p role="alert">{String(diagnostics.error)}</p>}
      {runtime && (
        <>
          <SettingsRow label="Data Directory">
            <SettingsPathField value={runtime.storage_root} openable={session?.snapshot.ownership === "owned_local"} />
          </SettingsRow>
          {runtime.path_conventions
            .filter((path) => path.absolute_path)
            .map((path) => (
              <SettingsRow key={path.key} label={path.key}>
                <SettingsPathField value={path.absolute_path} openable={session?.snapshot.ownership === "owned_local"} />
              </SettingsRow>
            ))}
        </>
      )}
    </SettingsCard>
  );
}

/* === Hooks === */

/* === Public Functions === */

/* === Private Functions === */
