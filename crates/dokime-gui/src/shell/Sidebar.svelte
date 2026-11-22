<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script module lang="ts">
  export type ShellDestination = "/cycles" | "/library" | "/history" | "/notifications" | "/settings";
</script>

<script lang="ts">
  import { Dialog, Tooltip } from "bits-ui";
  import { Bell, History, Library, ListChecks, Settings, X } from "lucide-svelte";
  import { createQuery } from "@tanstack/svelte-query";
  import { onMount, type Snippet } from "svelte";
  import type { ConnectedSession } from "../app/native-session/session";
  import Badge from "../components/ui/Badge.svelte";
  import Button from "../components/ui/Button.svelte";
  import { buttonVariants } from "../components/ui/button-variants";
  import { STATUS_PILL_CLASS } from "../features/cycle/cycle-workspace-model";
  import { highestPromptLevel, pendingPromptCountLabel } from "../features/cycle/prompt-banner-model";
  import { cycleRows } from "../features/cycles-home/cycles-model";
  import { cyclesQueryOptions, pendingPromptsQueryOptions } from "../features/cycles-home/cycles-query-options";
  import { DEFAULT_SETTINGS_SECTION, SETTINGS_SECTIONS } from "../features/settings/settings-sections";
  import type { Brand } from "../lib/brand";
  import { getNotificationSummary, subscribeNotificationSummary } from "../lib/notifications";
  import { recordLevelVariant } from "../lib/record-display";
  import { cn } from "../lib/utils";
  import SidebarFrame from "./SidebarFrame.svelte";
  import { sidebarCycles } from "./sidebar-cycles";
  import type { SidebarState } from "./use-sidebar-state";

  type Props = {
    brand: Brand;
    session: Pick<ConnectedSession, "identity" | "queryClient">;
    sidebar: SidebarState;
    pathname: string;
    cycleId: string | null;
    cycleCases?: Snippet;
    navigate: (destination: ShellDestination) => void;
    openCycle: (cycleId: string) => void;
    openSettings: (section: string) => void;
  };

  let { brand, session, sidebar, pathname, cycleId, cycleCases, navigate, openCycle, openSettings }: Props = $props();
  const destinations: readonly { to: ShellDestination; label: string; icon: typeof Bell }[] = [
    { icon: ListChecks, label: "Cycles", to: "/cycles" },
    { icon: Library, label: "Library", to: "/library" },
    { icon: History, label: "History", to: "/history" },
    { icon: Bell, label: "Notifications", to: "/notifications" },
    { icon: Settings, label: "Settings", to: "/settings" },
  ];
  const noAttention: ReadonlyMap<string, number> = new Map();
  const offsets = new Map<string, number>();
  const client = () => session.queryClient;
  const cycles = createQuery(() => cyclesQueryOptions(session.identity), client);
  const prompts = createQuery(() => pendingPromptsQueryOptions(session.identity), client);
  const pendingPrompts = $derived(prompts.data ?? []);
  const promptLevel = $derived(highestPromptLevel(pendingPrompts));
  const listedCycles = $derived(sidebarCycles(cycleRows(cycles.data ?? [], noAttention), cycleId));
  const settingsSection = $derived(
    isCurrent("/settings") ? (/^\/settings\/([^/]+)/.exec(pathname)?.[1] ?? DEFAULT_SETTINGS_SECTION) : null,
  );
  const closeClass = buttonVariants({ variant: "ghost", size: "icon-sm", class: "absolute top-3 right-3" });
  let drawer = $state(false);
  let unread = $state(getNotificationSummary().unread);

  onMount(() =>
    subscribeNotificationSummary(() => {
      unread = getNotificationSummary().unread;
    }),
  );

  function isCurrent(destination: ShellDestination) {
    return pathname === destination || pathname.startsWith(`${destination}/`);
  }

  function choose(destination: ShellDestination) {
    drawer = false;
    navigate(destination);
  }

  function rowTone(active: boolean) {
    return active ? "bg-primary/10 text-primary" : "text-muted-foreground hover:bg-muted hover:text-foreground";
  }

  function restoreScroll(node: HTMLUListElement, key: string) {
    node.scrollTop = offsets.get(key) ?? 0;
  }
</script>

{#snippet unreadBadge()}
  {#if unread}
    <Badge aria-label={`${unread} Unread Notifications`} variant="secondary">{unread}</Badge>
  {/if}
{/snippet}

{#snippet row(
  label: string,
  level: "destination" | "overview",
  active: boolean,
  chooseRow: () => void,
  icon?: typeof Bell,
  trailing?: Snippet,
)}
  {@const Icon = icon}
  <li
    class={cn(
      "grid min-h-[var(--sidebar-row-height)] grid-cols-(--layout-sidebar-destination) items-center gap-1 rounded-lg py-1 pr-2 pl-4",
      rowTone(active),
    )}
    data-active={active || undefined}
    data-level={level}
  >
    <span aria-hidden="true" class="flex shrink-0 items-center justify-start">
      {#if Icon}
        <Icon class="size-5" />
      {/if}
    </span>
    <Button
      variant="plain"
      size="inline"
      aria-current={active ? "page" : undefined}
      class="grid min-h-[var(--control-height)] min-w-0 items-center text-left"
      onclick={chooseRow}
      type="button"
    >
      <span class="line-clamp-2 min-w-0 text-left text-sm leading-tight font-medium">{label}</span>
    </Button>
    <span class="flex shrink-0 items-center gap-1">{@render trailing?.()}</span>
  </li>
{/snippet}

{#snippet destinationRow(
  destination: (typeof destinations)[number],
  collapsed: boolean,
  railLabels: boolean,
)}
  {@const active = isCurrent(destination.to)}
  {@const Icon = destination.icon}
  {#snippet trailing()}
    {#if destination.to === "/notifications"}
      {@render unreadBadge()}
    {:else if destination.to === "/cycles" && promptLevel}
      <Badge
        aria-label={pendingPromptCountLabel(pendingPrompts.length)}
        class={STATUS_PILL_CLASS}
        data-testid="navigation-prompt-count"
        variant={recordLevelVariant(promptLevel)}
      >
        {pendingPrompts.length}
      </Badge>
    {/if}
  {/snippet}
  {#if collapsed}
    <li
      class={cn("grid min-h-[var(--sidebar-row-height)] place-items-center rounded-lg", rowTone(active))}
      data-active={active || undefined}
    >
      <Tooltip.Root disabled={railLabels}>
        <Tooltip.Trigger
          data-slot="button"
          data-variant="plain"
          data-size="inline"
          aria-current={active ? "page" : undefined}
          aria-label={destination.label}
          class={buttonVariants({
            variant: "plain",
            size: "inline",
            class: "grid min-h-[var(--control-height)] w-full place-items-center content-center gap-1 rounded-lg",
          })}
          onclick={() => choose(destination.to)}
        >
          <Icon aria-hidden="true" class="size-5" />
          {#if railLabels}
            <span class="text-xs">{destination.label}</span>
          {/if}
        </Tooltip.Trigger>
        {#if !railLabels}
          <Tooltip.Portal>
            <Tooltip.Content side="right" class="z-50 max-w-xs rounded-lg bg-foreground px-3 py-2 text-xs text-background"
              >{destination.label}</Tooltip.Content
            >
          </Tooltip.Portal>
        {/if}
      </Tooltip.Root>
    </li>
  {:else}
    {@render row(destination.label, "destination", active, () => choose(destination.to), Icon, trailing)}
  {/if}
{/snippet}

{#snippet tree(
  expanded: boolean,
)}
  {@const collapsed = sidebar.collapsed && !expanded}
  {@const railLabels = sidebar.mode === "tablet" && !expanded}
  {@const cycleScrollKey = `${expanded}:cycles:${cycleId ?? ""}`}
  <ul class="flex min-h-0 flex-col gap-1 px-[var(--shell-gap)] pt-2 pb-2">
    {@render destinationRow(destinations[0], collapsed, railLabels)}
    {#if !collapsed && listedCycles.length > 0}
      <li class="flex min-h-0 flex-col">
        <ul
          class="grid min-h-0 content-start gap-1 overflow-y-auto"
          {@attach (node) => restoreScroll(node, cycleScrollKey)}
          onscroll={(event) => offsets.set(cycleScrollKey, event.currentTarget.scrollTop)}
        >
          {#each listedCycles as cycle (cycle.cycleId)}
            {#if cycle.current}
              {@render cycleCases?.()}
            {:else}
              {@render row(cycle.label, "overview", false, () => {
                drawer = false;
                openCycle(cycle.cycleId);
              })}
            {/if}
          {/each}
        </ul>
      </li>
    {/if}
    {@render destinationRow(destinations[1], collapsed, railLabels)}
    {@render destinationRow(destinations[2], collapsed, railLabels)}
    {#if expanded && settingsSection}
      <li class="flex min-h-0 flex-1 flex-col">
        <ul
          class="grid min-h-0 content-start gap-1 overflow-y-auto"
          {@attach (node) => restoreScroll(node, "/settings")}
          onscroll={(event) => offsets.set("/settings", event.currentTarget.scrollTop)}
        >
          {#each SETTINGS_SECTIONS as section (section.id)}
            {@render row(section.label, "overview", section.id === settingsSection, () => {
              drawer = false;
              openSettings(section.id);
            })}
          {/each}
        </ul>
      </li>
    {:else}
      <li class="flex-1"></li>
    {/if}
    {@render destinationRow(destinations[3], collapsed, railLabels)}
    {@render destinationRow(destinations[4], collapsed, railLabels)}
  </ul>
{/snippet}

{#if sidebar.mode === "phone"}
  <nav aria-label="Navigation" class="fixed inset-x-0 bottom-0 z-30 grid grid-cols-5 border-t bg-background">
    {#each destinations as destination (destination.to)}
      {@const Icon = destination.icon}
      <Button
        variant="plain"
        size="inline"
        type="button"
        aria-current={isCurrent(destination.to) ? "page" : undefined}
        class="grid min-h-16 justify-items-center content-center gap-1 text-xs aria-[current=page]:text-primary"
        onclick={() => choose(destination.to)}
      >
        <Icon class="size-5" />
        {destination.label}
        {#if destination.to === "/notifications"}
          {@render unreadBadge()}
        {/if}
      </Button>
    {/each}
  </nav>
{:else}
  <Tooltip.Provider>
    <SidebarFrame {...sidebar} {brand}>
      {@render tree(false)}
      {#if sidebar.mode === "tablet"}
        <Button
          variant="ghost"
          class="min-h-11"
          onclick={() => {
            drawer = true;
          }}
          >More</Button
        >
      {/if}
    </SidebarFrame>
    {#if sidebar.mode === "tablet"}
      <Dialog.Root bind:open={drawer}>
        <Dialog.Portal>
          <Dialog.Content
            data-slot="sheet-content"
            data-side="left"
            class="fixed inset-y-0 left-0 z-40 flex w-80 max-w-100 flex-col gap-0 bg-popover text-sm text-popover-foreground ring-1 ring-border duration-150 outline-none data-open:animate-in data-open:slide-in-from-left data-closed:animate-out data-closed:slide-out-to-left"
            onclick={(event) => {
              if (event.target instanceof Element && event.target.closest("button")) {
                drawer = false;
              }
            }}
          >
            <Dialog.Title data-slot="sheet-title" class="font-sans text-base leading-none font-medium">Navigation</Dialog.Title>
            {@render tree(true)}
            <Dialog.Close data-slot="sheet-close" class={closeClass} aria-label="Close"><X /></Dialog.Close>
          </Dialog.Content>
        </Dialog.Portal>
      </Dialog.Root>
    {/if}
  </Tooltip.Provider>
{/if}
