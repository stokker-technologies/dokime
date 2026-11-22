<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { CircleAlert, CircleCheck, Info, TriangleAlert } from "lucide-svelte";
  import { relativeTime } from "../../components/document/document-overview-model";
  import FailureConsequences from "../../components/FailureConsequences.svelte";
  import Button from "../../components/ui/Button.svelte";
  import { openNotificationDetails, type NotificationRecord } from "../../lib/notifications";

  const tones = {
    error: { icon: CircleAlert, label: "Error", class: "bg-destructive/10 text-destructive" },
    warning: { icon: TriangleAlert, label: "Warning", class: "bg-warning/10 text-warning" },
    info: { icon: Info, label: "Info", class: "bg-info/10 text-info" },
    success: { icon: CircleCheck, label: "Success", class: "bg-success/10 text-success" },
  };

  let {
    record,
    now,
    onNavigate,
  }: {
    record: NotificationRecord;
    now: number;
    onNavigate?: (href: string) => void;
  } = $props();
  const tone = $derived(tones[record.tone]);
  const Icon = $derived(tone.icon);
  const date = $derived(new Date(record.at));
  const source = $derived(record.source.trim());
  const href = $derived(
    record.missingSecret
      ? `/settings/secrets#add-secret=${encodeURIComponent(record.missingSecret)}`
      : record.pluginSettings
        ? `/settings/plugins/${encodeURIComponent(record.pluginSettings)}`
        : null,
  );

  function navigate(event: MouseEvent) {
    if (!href || !onNavigate || event.button !== 0 || event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) {
      return;
    }
    event.preventDefault();
    onNavigate(href);
  }
</script>

<li class="relative flex gap-3 px-4 py-4" data-tone={record.tone}>
  <span class={["flex size-8 shrink-0 items-center justify-center rounded-full", tone.class]}>
    <Icon class="size-4" aria-hidden="true" />
    <span class="sr-only">{tone.label}</span>
  </span>
  <div class="grid min-w-0 flex-1 gap-2">
    <div class="flex items-baseline justify-between gap-4">
      <Button
        variant="plain"
        size="inline"
        type="button"
        aria-haspopup="dialog"
        title={date.toLocaleString()}
        class="min-w-0 rounded text-left text-sm font-semibold text-foreground after:absolute after:inset-0 after:content-[''] [overflow-wrap:anywhere]"
        onclick={() => openNotificationDetails(record)}
      >
        {record.title}
      </Button>
      <time class="shrink-0 whitespace-nowrap text-xs text-muted-foreground" datetime={date.toISOString()} title={date.toLocaleString()}>
        {relativeTime(date.toISOString(), now)}
      </time>
    </div>
    {#if record.description?.trim()}
      <p class="whitespace-pre-wrap text-sm leading-relaxed text-muted-foreground [overflow-wrap:anywhere]">{record.description}</p>
    {/if}
    {#if source && source !== "/"}
      <span class="text-xs text-muted-foreground [overflow-wrap:anywhere]">{record.source}</span>
    {/if}
    <div class="relative z-10">
      <FailureConsequences items={record.consequences ?? []} />
    </div>
    {#if href}
      <a class="relative z-10 text-sm underline" {href} onclick={navigate}>
        {record.missingSecret ? "Set Secret" : "Open Plugin Settings"}
      </a>
    {/if}
  </div>
</li>
