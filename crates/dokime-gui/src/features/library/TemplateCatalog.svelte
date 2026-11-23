<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script module lang="ts">
  import type { SuiteTemplateEntry, TestSuiteEntry } from "../../lib/types";

  export type OpenTemplate = { suite: TestSuiteEntry; template: SuiteTemplateEntry };
</script>

<script lang="ts">
  import { Tooltip } from "bits-ui";
  import ErrorCard from "../../components/ErrorCard.svelte";
  import { selectionDraftKey, templateSelectionId } from "../../components/suite-picker/model";
  import SuiteTags from "../../components/suite-picker/SuiteTags.svelte";
  import Button from "../../components/ui/Button.svelte";
  import SortableTableHeader from "../../components/ui/SortableTableHeader.svelte";
  import TablePanel from "../../components/ui/TablePanel.svelte";
  import { nextTableOrder, orderTableRows, type TableOrder } from "../../components/ui/table-order";
  import { activateTableRow } from "../../components/ui/table-row";
  import type { NotificationFailure } from "../../lib/notification-details";
  import type { RuntimeInstalledPluginRecord } from "../settings/installed-plugins/installed-plugin-contracts";
  import CatalogIdentifier from "./CatalogIdentifier.svelte";
  import SuiteProvenance from "./SuiteProvenance.svelte";

  let {
    brandLabel,
    suites,
    plugins,
    failure,
    loading,
    onRetry,
    onOpen,
  }: {
    brandLabel: string;
    suites: TestSuiteEntry[];
    plugins: RuntimeInstalledPluginRecord[];
    failure?: NotificationFailure;
    loading: boolean;
    onRetry: () => void;
    onOpen: (value: OpenTemplate) => void;
  } = $props();

  const provider = $derived(new Map(plugins.map((plugin) => [plugin.plugin_id, plugin])));
  const waiting = $derived(plugins.filter((plugin) => ["failed", "connecting", "unavailable"].includes(plugin.status)));
  const catalogFailure = $derived(waiting.some((plugin) => plugin.plugin_id === failure?.plugin) ? undefined : failure);
  const hasFeedback = $derived(waiting.length > 0 || Boolean(catalogFailure));
  const sortableColumns = [
    { key: "name", label: "Template", width: "" },
    { key: "identifier", label: "ID", width: "w-80" },
    { key: "tags", label: "Tags", width: "w-40" },
  ] as const;
  type Column = (typeof sortableColumns)[number]["key"];
  let orders = $state.raw<Record<string, TableOrder<Column> | undefined>>({});

  function changeOrder(suiteId: string, column: Column) {
    orders = { ...orders, [suiteId]: nextTableOrder(orders[suiteId], column) };
  }
</script>

{#snippet feedback()}
  <div class="grid gap-3">
    {#each waiting as plugin (plugin.plugin_id)}
      <p role="status" class="text-sm text-muted-foreground">Waiting for Plugin {plugin.label}</p>
    {/each}
    {#if catalogFailure}
      <ErrorCard failure={catalogFailure} {onRetry} />
    {/if}
  </div>
{/snippet}

{#snippet headings(
  suiteId: string | null,
)}
  {#each sortableColumns as column (column.key)}
    {#if column.key === "tags"}
      <th scope="col" class="w-48 px-2 font-medium">Suite</th>
    {/if}
    {@const order = suiteId ? orders[suiteId] : undefined}
    {#if suiteId}
      <SortableTableHeader
        class={`px-2 first:pl-4 ${column.width}`}
        label={column.label}
        direction={order?.column === column.key ? order.direction : undefined}
        onclick={() => changeOrder(suiteId, column.key)}
      />
    {:else}
      <th scope="col" class={`px-2 first:pl-4 font-medium ${column.width}`}>{column.label}</th>
    {/if}
  {/each}
  <th scope="col" class="w-64 px-2 pr-4 font-medium">Source</th>
{/snippet}

<Tooltip.Provider>
  <section class="grid gap-3" aria-label="Library">
    {#each suites as suite, index (suite.suite_id)}
      {@const ordered = orderTableRows(
        suite.members.map((template) => ({
          template,
          name: template.name,
          identifier: template.identifier,
          tags: template.tags.join(", "),
        })),
        orders[suite.suite_id],
      )}
      <details class="rounded-lg border bg-card p-4" open={index === 0 && hasFeedback}>
        <summary class="cursor-pointer text-sm font-semibold">
          {suite.name} <span class="ml-2 font-normal text-muted-foreground">{suite.members.length} Templates</span>
        </summary>
        <div class="mt-3">
          <TablePanel
            title={suite.name}
            description={suite.description || "Templates available in this suite."}
            columnCount={5}
            rowCount={ordered.length}
            empty="No Templates Match"
            failure={index === 0 && hasFeedback ? feedback : undefined}
          >
            {#snippet columns()}
              {@render headings(suite.suite_id)}
            {/snippet}
            {#each ordered as { template } (templateSelectionId(template))}
              <tr
                class="h-(--list-row-height) cursor-pointer hover:bg-muted/50"
                data-row-id={selectionDraftKey(suite.suite_id, templateSelectionId(template))}
                {@attach (row) => activateTableRow(row, () => onOpen({ suite, template }))}
              >
                <td class="px-4">
                  <Button
                    variant="plain"
                    size="inline"
                    class="block w-full truncate text-left font-medium"
                    title={template.name}
                    onclick={() => onOpen({ suite, template })}
                    >{template.name}</Button
                  >
                </td>
                <td class="px-2">
                  {#if template.identifier}
                    <CatalogIdentifier value={template.identifier} />
                  {/if}
                </td>
                <td class="truncate px-2">{suite.name}</td>
                <td class="px-2"><SuiteTags tags={template.tags} /></td>
                <td class="px-2 pr-4"><SuiteProvenance plugin={provider.get(suite.source)} source={suite.source} {brandLabel} /></td>
              </tr>
            {/each}
          </TablePanel>
        </div>
      </details>
    {:else}
      <TablePanel
        title="Templates"
        description="Templates available in the connected host's suites."
        columnCount={5}
        rowCount={0}
        empty="No Templates Match"
        {loading}
        failure={hasFeedback ? feedback : undefined}
      >
        {#snippet columns()}
          {@render headings(null)}
        {/snippet}
      </TablePanel>
    {/each}
  </section>
</Tooltip.Provider>
