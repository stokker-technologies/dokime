<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { createQuery } from "@tanstack/svelte-query";
  import { untrack } from "svelte";
  import type { ConnectedSession } from "../../../app/native-session/session";
  import Button from "../../../components/ui/Button.svelte";
  import { putSecret, readSecrets, removeSecret, type SecretEntry, type SecretSnapshot } from "../../../lib/api/native-secrets";
  import { failureOf } from "../../../lib/notification-details";
  import SettingsCard from "../SettingsCard.svelte";
  import SettingsRow from "../SettingsRow.svelte";
  import SecretEditor, { type Editor } from "./SecretEditor.svelte";
  import SecretsTable from "./SecretsTable.svelte";
  import SecretTransfer from "./SecretTransfer.svelte";

  let { session, hash = "", clearHash }: { session: ConnectedSession; hash?: string; clearHash: () => void } = $props();
  const client = $derived(session.queryClient);
  const queryKey = $derived(["settings", "secrets", session.identity.generation]);
  const query = createQuery(
    () => ({ queryKey, queryFn: () => readSecrets(session.identity) }),
    () => client,
  );
  const snapshot = $derived(query.data);
  let editor = $state.raw<(Editor & { generation: bigint }) | null>(null);
  let deleting = $state.raw<{ identifier: string; generation: bigint } | null>(null);
  let busy = $state(false);
  let error = $state.raw<unknown>();

  $effect(() => {
    if (hash.startsWith("add-secret=") && snapshot) {
      editor = { kind: "new", identifier: hash.slice("add-secret=".length), generation: snapshot.generation };
      untrack(clearHash);
    }
  });

  async function change(work: () => Promise<SecretSnapshot>) {
    busy = true;
    error = undefined;
    const identity = session.identity;
    const key = queryKey;
    const active = client;
    try {
      await active.cancelQueries({ queryKey: key });
      const next = await work();
      if (identity.isCurrent()) {
        active.setQueryData(key, next);
        void active.invalidateQueries({ queryKey: ["settings", "installed-plugins"] });
        editor = null;
        deleting = null;
      }
    } catch (failure) {
      if (identity.isCurrent()) {
        error = failure;
      }
    } finally {
      if (identity.isCurrent()) {
        busy = false;
      }
    }
  }

  function remove() {
    const current = deleting;
    const identity = session.identity;
    if (current) {
      void change(() => removeSecret(identity, current.identifier, current.generation));
    }
  }

  async function save(metadata: SecretEntry["metadata"], value: string | undefined) {
    const current = editor;
    const identity = session.identity;
    if (current) {
      await change(() => putSecret(identity, metadata, value, current.generation));
    }
  }
</script>

<SettingsCard title="Secret Storage">
  <SettingsRow label="Owning Host">
    <span>{snapshot ? `${snapshot.hostLabel} · ${snapshot.hostId}` : "Loading Host…"}</span>
  </SettingsRow>
  <div class="grid gap-3 px-4 py-3">
    {#if snapshot?.cleanupPending}
      <p role="status" class="whitespace-pre-wrap text-sm text-muted-foreground">
        The change is active, but an obsolete keyring slot could not be removed.
      </p>
    {/if}
    <SecretsTable
      busy={busy || !snapshot}
      entries={snapshot?.entries ?? []}
      loading={query.isPending}
      failure={error ? failureOf(error, "Operation Failed") : query.error ? failureOf(query.error, "Secrets Unavailable") : undefined}
      onDelete={(entry) => {
        if (snapshot) deleting = { identifier: entry.metadata.identifier, generation: snapshot.generation };
      }}
      onEdit={(entry) => {
        if (snapshot) editor = { kind: "edit", entry, generation: snapshot.generation };
      }}
      onRetry={!error && query.error ? () => void query.refetch() : undefined}
    >
      {#snippet actions()}
        <Button
          disabled={!snapshot || busy}
          onclick={() => {
            if (snapshot) editor = { kind: "new", generation: snapshot.generation };
          }}
        >
          Add Secret
        </Button>
        <Button disabled={busy || query.isFetching} variant="outline" onclick={() => void query.refetch()}>Refresh</Button>
        {#if snapshot}
          <SecretTransfer
            session={session.identity}
            catalog={snapshot}
            onImported={(next) => {
              if (session.identity.isCurrent()) client.setQueryData(queryKey, next);
            }}
          />
        {/if}
      {/snippet}
    </SecretsTable>
    {#if deleting}
      <fieldset class="flex flex-wrap items-center gap-3 rounded-lg border p-3" aria-label="Confirm Secret Deletion">
        <p class="text-sm">Delete {deleting.identifier}?</p>
        <Button variant="destructive" disabled={busy} onclick={remove}>Confirm Delete</Button>
        <Button variant="outline" disabled={busy} onclick={() => (deleting = null)}>Cancel</Button>
      </fieldset>
    {/if}
    {#if editor && snapshot}
      {#key editor.kind === "edit" ? editor.entry.metadata.identifier : `new-${editor.identifier ?? ""}`}
        <SecretEditor {editor} {busy} cancel={() => (editor = null)} submit={save} />
      {/key}
    {/if}
  </div>
</SettingsCard>
