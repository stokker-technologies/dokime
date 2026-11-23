<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import type { QueryClient } from "@tanstack/svelte-query";
  import { Dialog } from "bits-ui";
  import type { ConnectedSession } from "../../../app/native-session/session";
  import DialogContent from "../../../components/ui/DialogContent.svelte";
  import type { SecretMetadata } from "../../../lib/api/native-secrets";
  import { createSecretStorage } from "./secret-storage.svelte";
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
  const storage = createSecretStorage(
    () => session,
    () => queryClient,
  );
  const query = storage.query;

  async function submit(metadata: SecretMetadata, value: string | undefined) {
    const snapshot = query.data;
    if (snapshot) {
      await storage.save(metadata, value, snapshot.generation, onClose);
    }
  }
</script>

{#if session && query.data}
  <SecretEditor editor={{ kind: "new", identifier }} busy={storage.busy} cancel={onClose} error={storage.error} {submit} />
{:else}
  <Dialog.Root
    open
    onOpenChange={(open) => {
      if (!open) {
        onClose();
      }
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
