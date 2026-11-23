<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import Button from "@/components/ui/Button.svelte";
  import { downloadNativeCheckpointArtifact } from "@/lib/api/native-checkpoints";
  import { saveNativeDocument } from "@/lib/api/native-document-delivery";
  import type { CheckpointProps } from "./checkpoint-view";
  import ErrorCard from "@/components/ErrorCard.svelte";
  import { failureOf } from "@/lib/notification-details";

  let { checkpoint, connection, id, name }: CheckpointProps & { id: string; name: string } = $props();
  let busy = $state(false);
  let error = $state.raw<unknown>();

  async function download() {
    const { session } = connection;
    busy = true;
    error = undefined;
    try {
      const artifact = await downloadNativeCheckpointArtifact(session, checkpoint, id);
      await saveNativeDocument(session, {
        kind: "artifact",
        filename: (name || artifact.metadata.filename).split("/").at(-1) ?? name,
        bytes: artifact.bytes,
      });
    } catch (failure) {
      error = failure;
    } finally {
      busy = false;
    }
  }
</script>

<div>
  <Button class="h-auto max-w-full whitespace-normal break-all" variant="outline" size="sm" disabled={busy} onclick={() => void download()}>
    {busy ? "Downloading…" : name}
  </Button>
  {#if Boolean(error)}
    <ErrorCard notificationContext="action" failure={failureOf(error, "Operation Failed")} />
  {/if}
</div>
