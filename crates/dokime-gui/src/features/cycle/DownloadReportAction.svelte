<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { DropdownMenu } from "bits-ui";
  import { Download } from "lucide-svelte";
  import Button from "../../components/ui/Button.svelte";
  import type { downloadNativeReport } from "../../lib/api/native-reports";
  import { failureOf } from "../../lib/notification-details";
  import { toast } from "../../lib/notifications";
  import { reportDownloadState } from "./cycle-workspace-model";

  let {
    mutation,
    executionId,
    label,
    sealed,
    presentation = "menu",
  }: {
    mutation: {
      isPending: boolean;
      mutateAsync: (subject: { executionId: string | null; label: string }) => ReturnType<typeof downloadNativeReport>;
    };
    executionId: string | null;
    label: string;
    sealed: boolean;
    presentation?: "menu" | "button";
  } = $props();
  const delivery = $derived(reportDownloadState(mutation.isPending, sealed));

  async function save() {
    if (delivery.disabled) {
      return;
    }
    try {
      const saved = await mutation.mutateAsync({ executionId, label });
      if (saved) {
        toast.success("Report Saved");
        if (saved.openFailure) {
          toast.error(saved.openFailure);
        }
      }
    } catch (error: unknown) {
      toast.error(failureOf(error, "The Report Could Not Be Saved"));
    }
  }
</script>

{#snippet content()}
  <Download data-icon="inline-start" />
  {mutation.isPending ? "Saving Report…" : delivery.label}
{/snippet}

{#if presentation === "button"}
  <Button disabled={delivery.disabled} onclick={save} size="sm" title={delivery.title} variant="outline"> {@render content()} </Button>
{:else}
  <DropdownMenu.Item
    class="flex cursor-default items-center gap-2 rounded-lg px-2 py-1 text-sm outline-none data-highlighted:bg-accent data-disabled:pointer-events-none data-disabled:opacity-50 [&>svg]:size-4"
    disabled={delivery.disabled}
    onSelect={save}
    title={delivery.title}
  >
    {@render content()}
  </DropdownMenu.Item>
{/if}
