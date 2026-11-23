<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { createQuery } from "@tanstack/svelte-query";
  import type { ConnectedSession } from "../../app/native-session/session";
  import ErrorCard from "../../components/ErrorCard.svelte";
  import { selectionDraftKey, templateSelectionId } from "../../components/suite-picker/model";
  import { testSuitesQueryOptions } from "../../components/suite-picker/server-state";
  import Button from "../../components/ui/Button.svelte";
  import { failureOf } from "../../lib/notification-details";
  import { admittedPluginRecords } from "../../lib/api/native-plugins";
  import { publishPageChrome } from "../../shell/page-chrome.svelte";
  import { pageContentClass } from "../../shell/page-layout";
  import NewCycleDialog from "../cycles-home/NewCycleDialog.svelte";
  import { pluginsListQueryOptions } from "../settings/plugins/plugin-queries";
  import TemplateCatalog, { type OpenTemplate } from "./TemplateCatalog.svelte";
  import TemplateDialog from "./TemplateDialog.svelte";
  import type { ActiveCycleState } from "./active-cycle-model";

  let {
    session,
    active,
    label,
    appendTo,
    openCycle,
    addCases,
  }: {
    session: ConnectedSession;
    active: ActiveCycleState;
    label: string;
    appendTo?: string;
    openCycle: (cycleId: string) => void;
    addCases: (cycleId: string) => void;
  } = $props();
  const suites = createQuery(
    () => testSuitesQueryOptions(session.identity),
    () => session.queryClient,
  );
  const plugins = createQuery(
    () => ({ ...pluginsListQueryOptions(session), select: admittedPluginRecords }),
    () => session.queryClient,
  );
  let openTemplate = $state.raw<OpenTemplate | null>(null);
  let creating = $state(false);
  let applied: string | undefined;

  // Apply The Cycle Named By Add Cases Once Its Open State Is Confirmed.
  $effect(() => {
    const requested = appendTo;
    if (requested && requested !== applied && active.cycles.some((cycle) => cycle.cycleId === requested)) {
      applied = requested;
      active.choose(requested);
    }
  });
  publishPageChrome(() => ({ title: "Library", actions }));
</script>

{#snippet actions()}
  {#if active.cycle}
    <Button
      class="h-[var(--control-height)]"
      variant="outline"
      onclick={() => {
        if (active.cycle) {
          openCycle(active.cycle.cycleId);
        }
      }}
      >Open Cycle</Button
    >
  {:else}
    <Button class="h-[var(--control-height)]" onclick={() => (creating = true)}>New Cycle</Button>
  {/if}
{/snippet}

<div class={pageContentClass}>
  {#if active.failed}
    <ErrorCard failure={failureOf(active.error, "Open Cycles Unavailable")} />
  {/if}
  <TemplateCatalog
    brandLabel={label}
    suites={suites.data ?? []}
    plugins={plugins.data ?? []}
    failure={suites.error ? failureOf(suites.error, "Could Not Load Suites") : undefined}
    loading={suites.isPending}
    onRetry={() => void suites.refetch()}
    onOpen={(template) => (openTemplate = template)}
  />
  {#if openTemplate}
    {#key selectionDraftKey(openTemplate.suite.suite_id, templateSelectionId(openTemplate.template))}
      <TemplateDialog
        activeCycle={active.cycle}
        hasOpenCycles={active.cycles.length > 0}
        open
        onOpenChange={(open) => {
          if (!open) {
            openTemplate = null;
          }
        }}
        onChangeCycle={active.openPicker}
        onNewCycle={() => (creating = true)}
        {openCycle}
        {session}
        suite={openTemplate.suite}
        template={openTemplate.template}
      />
    {/key}
  {/if}
  <NewCycleDialog
    {session}
    open={creating}
    onOpenChange={(open) => (creating = open)}
    onCreated={(cycleId) => {
      active.choose(cycleId);
      addCases(cycleId);
    }}
  />
</div>
