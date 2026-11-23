<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { createMutation, createQuery } from "@tanstack/svelte-query";
  import { DropdownMenu } from "bits-ui";
  import { ArrowUp } from "lucide-svelte";
  import type { ConnectedSession } from "../../app/native-session/session";
  import ErrorCard from "../../components/ErrorCard.svelte";
  import Switch from "../../components/ui/Switch.svelte";
  import TablePanel from "../../components/ui/TablePanel.svelte";
  import TableRowActions from "../../components/ui/TableRowActions.svelte";
  import { reorderStorageSources, setStorageSourceEnabled } from "../../lib/api/native-storage-settings";
  import { storageSourcesQueryOptions } from "./settings-queries";
  import { failureOf } from "../../lib/notification-details";

  type Change = { kind: "enabled"; identifier: string; enabled: boolean } | { kind: "order"; identifiers: string[] };

  let { session }: { session: ConnectedSession } = $props();
  const options = $derived(storageSourcesQueryOptions(session.identity));
  const client = () => session.queryClient;
  const query = createQuery(() => options, client);
  const mutation = createMutation(() => {
    const { identity, queryClient } = session;
    const key = options.queryKey;
    return {
      mutationFn: async (change: Change) => {
        await queryClient.cancelQueries({ queryKey: key });
        return change.kind === "order"
          ? reorderStorageSources(identity, change.identifiers)
          : setStorageSourceEnabled(identity, change.identifier, change.enabled);
      },
      onSuccess: (sources) => {
        if (identity.isCurrent()) {
          queryClient.setQueryData(key, sources);
        }
      },
    };
  }, client);
  const canChange = $derived(session.snapshot.connection.backend?.methods.includes("/dokime.host.v1.storage.Sources/Set") ?? false);
  const sources = $derived(query.data ?? []);
  const change = $derived(mutation.variables);
  const changedIdentifier = $derived(change?.kind === "enabled" ? change.identifier : null);

  function move(index: number, direction: number) {
    const identifiers = sources.map((source) => source.identifier);
    const target = index + direction;
    if (target < 0 || target >= identifiers.length || mutation.isPending || !canChange) {
      return;
    }
    [identifiers[index], identifiers[target]] = [identifiers[target], identifiers[index]];
    mutation.mutate({ kind: "order", identifiers });
  }

  function sourceLabel(identifier: string): string {
    return identifier === "local" ? "Local" : identifier === "fabric" ? "Fabric" : identifier;
  }
</script>

{#snippet failureRow()}
  {#if mutation.error}
    <ErrorCard notificationContext="action" failure={failureOf(mutation.error, "Operation Failed")} />
  {:else if query.error}
    <ErrorCard failure={failureOf(query.error, "Storage Sources Unavailable")} onRetry={() => void query.refetch()} />
  {/if}
{/snippet}

<TablePanel
  title="Storage Sources"
  label="Storage Source Priority"
  description={canChange
    ? "Higher rows win conflicting edits; changing priority does not move data. Remote synchronization is not available yet."
    : "This host does not advertise storage source changes, so ordering and enabling are unavailable. Remote synchronization is not available yet."}
  empty="No Storage Sources Are Configured"
  columnCount={4}
  rowCount={sources.length}
  loading={query.isPending}
  failure={mutation.error || query.error ? failureRow : undefined}
>
  {#snippet columns()}
    <th scope="col" class="px-4 font-medium">Source</th>
    <th scope="col" class="w-48 px-2 font-medium">State</th>
    <th scope="col" class="w-24 px-2 text-right font-medium">Priority</th>
    <th scope="col" class="w-24 px-4 text-right font-medium">Actions</th>
  {/snippet}
  {#each sources as source, index (source.identifier)}
    {@const label = sourceLabel(source.identifier)}
    <tr class="h-(--list-row-height)" data-row-id={source.identifier}>
      <td class="truncate px-4 font-medium">{label}</td>
      <td class="px-2">
        <div class="flex items-center gap-2">
          <Switch
            aria-label={`${source.enabled ? "Disable" : "Enable"} ${label}`}
            checked={source.enabled}
            disabled={source.identifier === "native" || !canChange || mutation.isPending}
            onCheckedChange={(enabled) => mutation.mutate({ kind: "enabled", identifier: source.identifier, enabled })}
          />
          <span class="text-muted-foreground">{source.enabled ? "Enabled" : "Disabled"}</span>
          {#if changedIdentifier === source.identifier}
            <span class="text-xs text-muted-foreground" role={mutation.isPending ? "status" : undefined}>
              {mutation.isPending ? "Saving" : mutation.isSuccess ? "Saved" : ""}
            </span>
          {/if}
        </div>
      </td>
      <td class="px-2 text-right">{index + 1}</td>
      <td class="px-4">
        <TableRowActions
          {label}
          primary="Move Up"
          disabled={!canChange || mutation.isPending || index === 0}
          onSelect={() => move(index, -1)}
        >
          {#snippet icon()}
            <ArrowUp />
          {/snippet}
          <DropdownMenu.Item
            class="cursor-default rounded-lg px-2 py-1 text-sm outline-none data-highlighted:bg-accent data-disabled:opacity-50"
            disabled={!canChange || mutation.isPending || index === sources.length - 1}
            onSelect={() => move(index, 1)}
            >Move Down</DropdownMenu.Item
          >
        </TableRowActions>
      </td>
    </tr>
  {/each}
</TablePanel>
