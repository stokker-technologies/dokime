/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useId, useState, useSyncExternalStore } from "react";

/* === Local Imports === */

import { useNativeSessionController, useNativeSessionOptional } from "@/app/native-session/context";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { SettingsCard } from "@/features/settings/SettingsCard";
import { SettingsRow } from "@/features/settings/SettingsRow";
import { commands } from "@/lib/desktop/commands";
import type { BackendProfiles } from "@/lib/desktop/connection";

/* === Constants === */

const PROFILES_KEY = ["desktop", "profiles"];

/* === Components === */

export function BackendSelectionCard() {
  const owner = useNativeSessionController();
  const session = useNativeSessionOptional();
  const state = useSyncExternalStore(owner.subscribe, owner.snapshot);
  const selectionError = state.phase === "connected" ? state.selectionError : null;
  const client = useQueryClient();
  const query = useQuery({ queryKey: PROFILES_KEY, queryFn: commands.desktopProfilesList, retry: false });
  const [choice, setChoice] = useState("local");
  const [editing, setEditing] = useState(false);
  const [originalName, setOriginalName] = useState<string | null>(null);
  const [name, setName] = useState("");
  const [endpoint, setEndpoint] = useState("");
  const [identity, setIdentity] = useState("");
  const [certificate, setCertificate] = useState("");
  const [secret, setSecret] = useState<"none" | "keyring">("keyring");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const id = useId();
  const selection = session?.snapshot.selection;
  const selectedName = selection?.kind === "profile" ? selection.name : null;
  const profile = query.data?.profiles.find((entry) => choice === `profile:${entry.name}`);
  const removalBlocked = profile?.name === query.data?.default || profile?.name === selectedName;

  useEffect(() => {
    setChoice(selection?.kind === "profile" ? `profile:${selection.name}` : "local");
  }, [selection]);

  function edit(value?: BackendProfiles["profiles"][number]) {
    setEditing(true);
    setOriginalName(value?.name ?? null);
    setName(value?.name ?? "");
    setEndpoint(value?.endpoint ?? "");
    setIdentity(value?.expected_backend_id ?? "");
    setCertificate(value?.ca_certificate_pem ?? "");
    setSecret(value?.secret ?? "keyring");
    setError(null);
  }

  async function change(operation: () => Promise<BackendProfiles>) {
    setBusy(true);
    setError(null);
    try {
      client.setQueryData(PROFILES_KEY, await operation());
      setEditing(false);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusy(false);
    }
  }

  return (
    <SettingsCard
      span="full"
      title="Desktop Backend"
      description="Local is owned by this desktop. Saved remote profiles are shared with the CLI; its default stays separate."
    >
      <SettingsRow label="Backend">
        <select
          aria-label="Desktop Backend"
          className="rounded-md border bg-background p-2"
          disabled={busy || query.isPending}
          value={choice}
          onChange={(event) => setChoice(event.target.value)}
        >
          <option value="local">Local</option>
          {query.data?.profiles.map((entry) => (
            <option key={entry.name} value={`profile:${entry.name}`}>
              {entry.name}
            </option>
          ))}
        </select>
        <Button
          disabled={busy || (choice !== "local" && !profile) || (choice === "local" && session?.snapshot.ownership === "owned_local")}
          onClick={() => {
            if (choice === "local") void owner.select({ kind: "owned_local" });
            else if (profile) void owner.select({ kind: "profile", name: profile.name });
          }}
        >
          {choice === "local" ? "Start Local Backend" : "Connect"}
        </Button>
        <Button variant="outline" disabled={busy || !session} onClick={() => void owner.disconnect()}>
          {session?.snapshot.ownership === "owned_local" ? "Stop Local Backend" : "Disconnect"}
        </Button>
      </SettingsRow>
      <SettingsRow
        label="Saved Remotes"
        error={error ?? (selectionError ? String(selectionError) : query.error ? String(query.error) : null)}
      >
        <Button variant="outline" disabled={busy} onClick={() => edit()}>
          Add Remote
        </Button>
        <Button variant="outline" disabled={busy || !profile} onClick={() => edit(profile)}>
          Edit Remote
        </Button>
        <Button
          variant="outline"
          disabled={busy || !profile || removalBlocked}
          onClick={() => profile && void change(() => commands.desktopProfilesRemove(profile.name))}
        >
          Remove Remote
        </Button>
        {removalBlocked && profile ? (
          <p className="text-xs text-muted-foreground">The selected desktop backend or CLI default cannot be removed.</p>
        ) : null}
        {query.isError ? (
          <Button variant="outline" onClick={() => void query.refetch()}>
            Reload Remotes
          </Button>
        ) : null}
      </SettingsRow>
      {editing ? (
        <form
          className="grid gap-3 p-4"
          onSubmit={(event) => {
            event.preventDefault();
            void change(() =>
              commands.desktopProfilesSave({
                name: name.trim(),
                endpoint: endpoint.trim(),
                expected_backend_id: identity.trim() || null,
                ca_certificate_pem: certificate.trim() || null,
                secret,
              }),
            );
          }}
        >
          <label htmlFor={`${id}-name`}>Remote Name</label>
          <Input
            id={`${id}-name`}
            required
            disabled={busy || originalName !== null}
            value={name}
            onChange={(event) => setName(event.target.value)}
          />
          <label htmlFor={`${id}-endpoint`}>Endpoint</label>
          <Input
            id={`${id}-endpoint`}
            required
            type="url"
            disabled={busy}
            value={endpoint}
            onChange={(event) => setEndpoint(event.target.value)}
          />
          <label htmlFor={`${id}-identity`}>Expected Backend ID (Optional)</label>
          <Input id={`${id}-identity`} disabled={busy} value={identity} onChange={(event) => setIdentity(event.target.value)} />
          <label htmlFor={`${id}-certificate`}>CA Certificate PEM (Optional)</label>
          <textarea
            id={`${id}-certificate`}
            className="rounded-md border bg-background p-2"
            disabled={busy}
            value={certificate}
            onChange={(event) => setCertificate(event.target.value)}
          />
          <label htmlFor={`${id}-secret`}>Credentials</label>
          <select
            id={`${id}-secret`}
            disabled={busy}
            value={secret}
            onChange={(event) => setSecret(event.target.value as "none" | "keyring")}
          >
            <option value="keyring">System Keyring</option>
            <option value="none">None</option>
          </select>
          <div className="flex gap-2">
            <Button type="submit" disabled={busy}>
              Save Remote
            </Button>
            <Button type="button" variant="outline" disabled={busy} onClick={() => setEditing(false)}>
              Cancel
            </Button>
          </div>
          {error ? <p role="alert">{error}</p> : null}
        </form>
      ) : null}
    </SettingsCard>
  );
}
