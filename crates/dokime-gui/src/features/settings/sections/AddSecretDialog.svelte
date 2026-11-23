<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { createQuery, type QueryClient } from "@tanstack/svelte-query";
  import { Dialog } from "bits-ui";
  import type { ConnectedSession } from "../../../app/native-session/session";
  import DialogContent from "../../../components/ui/DialogContent.svelte";
  import { putSecret, readSecrets, type SecretMetadata } from "../../../lib/api/native-secrets";
  import SecretEditor from "./SecretEditor.svelte";

  let {
    session,
    queryClient,
    identifier,
    onClose,
  }: {
    session: ConnectedSession | null;
    queryClient: QueryClient;
    identifier: string;
    onClose: () => void;
  } = $props();
  const queryKey = $derived(["settings", "secrets", session?.identity.generation]);
  const query = createQuery(
    () => ({
      queryKey,
      enabled: Boolean(session),
      queryFn: () => {
        if (!session) {
          throw new Error("Connect to manage Secret Storage.");
        }
        return readSecrets(session.identity);
      },
    }),
    () => queryClient,
  );
  let busy = $state(false);
  let error = $state.raw<unknown>();

  async function submit(metadata: SecretMetadata, value: string | undefined) {
    const identity = session?.identity;
    const snapshot = query.data;
    if (!identity || !snapshot) {
      return;
    }
    const key = queryKey;
    const client = queryClient;
    busy = true;
    error = undefined;
    try {
      await client.cancelQueries({ queryKey: key });
      const next = await putSecret(identity, metadata, value, snapshot.generation);
      if (identity.isCurrent()) {
        client.setQueryData(key, next);
        void client.invalidateQueries({ queryKey: ["settings", "installed-plugins"] });
        onClose();
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
</script>

{#if session && query.data}
  <SecretEditor editor={{ kind: "new", identifier }} {busy} cancel={onClose} {error} {submit} />
{:else}
  <Dialog.Root
    open
    onOpenChange={(open) => {
      if (!open) onClose();
    }}
  >
    <DialogContent>
      <header class="flex flex-col gap-2">
        <Dialog.Title class="font-medium">Add Secret</Dialog.Title>
        <Dialog.Description class="text-muted-foreground">
          {query.error?.message ?? (session ? "Reading Secret Storage…" : "Connect to manage Secret Storage.")}
        </Dialog.Description>
      </header>
    </DialogContent>
  </Dialog.Root>
{/if}
