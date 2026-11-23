<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { createMutation, createQuery, type QueryClient } from "@tanstack/svelte-query";
  import ErrorCard from "../../../components/ErrorCard.svelte";
  import Button from "../../../components/ui/Button.svelte";
  import Input from "../../../components/ui/Input.svelte";
  import { commands } from "../../../lib/desktop/commands";
  import type { SavedSettingsChange } from "../../../lib/desktop/settings";
  import { formatUnitLabel } from "../../../lib/format";
  import { failureOf } from "../../../lib/notification-details";
  import SettingsCard from "../SettingsCard.svelte";
  import SettingsRow from "../SettingsRow.svelte";

  const QUERY_KEY = ["settings", "saved"];
  const MAX_WINDOW_MS = Number.MAX_SAFE_INTEGER;
  const MAX_LAG_WINDOWS = 4_294_967_295;
  type Draft = { window: string; lag: string };

  let { queryClient }: { queryClient: QueryClient } = $props();
  const id = $props.id();
  let draft = $state.raw<Draft | null>(null);
  const query = createQuery(
    () => ({ queryKey: QUERY_KEY, queryFn: () => commands.desktopSavedSettingsRead() }),
    () => queryClient,
  );
  const mutation = createMutation(
    () => ({
      mutationFn: (change: SavedSettingsChange) => commands.desktopSavedSettingsUpdate(change),
      onSuccess: (saved) => {
        queryClient.setQueryData(QUERY_KEY, saved);
        draft = null;
      },
    }),
    () => queryClient,
  );
  const saved = $derived(query.data);
  const values = $derived(draft ?? (saved ? { window: String(saved.interval_window_ms), lag: String(saved.interval_lag_windows) } : null));
  const fields = [
    {
      key: "window",
      label: formatUnitLabel("Interval Window", "ms"),
      max: MAX_WINDOW_MS,
      description: "Sampling time one window covers. A stream's own window setting overrides it.",
    },
    {
      key: "lag",
      label: formatUnitLabel("Interval Lag", "windows"),
      max: MAX_LAG_WINDOWS,
      description: "Window lengths a stream may fall behind before its execution is interrupted.",
    },
  ] as const;
  const changed = $derived(
    Boolean(
      values &&
        saved &&
        (values.window.trim() !== String(saved.interval_window_ms) || values.lag.trim() !== String(saved.interval_lag_windows)),
    ),
  );
  const valid = $derived(values !== null && fields.every((field) => !wholeNumberError(values?.[field.key] ?? "", field.max)));
  const canSave = $derived(changed && valid && !mutation.isPending);

  function wholeNumberError(text: string, max: number): string | undefined {
    const trimmed = text.trim();
    const value = Number(trimmed);
    return /^\d+$/.test(trimmed) && Number.isSafeInteger(value) && value >= 1 && value <= max
      ? undefined
      : `Enter a whole number from 1 to ${max}.`;
  }

  function save() {
    if (values && canSave) {
      mutation.mutate({ interval_window_ms: Number(values.window.trim()), interval_lag_windows: Number(values.lag.trim()) });
    }
  }
</script>

<SettingsCard
  title="Streams"
  description="This desktop's built-in host places interval streams with these values. Saved values apply when the built-in host next starts; an attached host keeps its own configuration."
>
  {#if query.isPending}
    <p role="status" class="px-4 py-3 text-sm text-muted-foreground">Loading Stream Settings…</p>
  {/if}
  {#if query.error}
    <ErrorCard failure={failureOf(query.error, "Stream Settings Unavailable")} />
  {/if}
  {#if values}
    {#each fields as field (field.key)}
      {@const inputId = `${id}-${field.key}`}
      {@const error = wholeNumberError(values[field.key], field.max)}
      <SettingsRow for={inputId} label={field.label} description={field.description} {error}>
        <Input
          id={inputId}
          aria-label={field.label}
          aria-invalid={Boolean(error)}
          class="text-right tabular-nums"
          inputmode="numeric"
          value={values[field.key]}
          oninput={(event) => {
            if (values) {
              draft = { ...values, [field.key]: event.currentTarget.value };
            }
          }}
        />
      </SettingsRow>
    {/each}
    <SettingsRow
      label="Save Stream Settings"
      notificationContext="action"
      error={mutation.error ? failureOf(mutation.error, "Save Stream Settings").message : undefined}
      failure={mutation.error ?? undefined}
    >
      {#snippet status()}
        {#if mutation.isPending}
          <span role="status">Saving</span>
        {/if}
      {/snippet}
      <Button disabled={!canSave} onclick={save} size="sm">Save</Button>
    </SettingsRow>
  {/if}
</SettingsCard>
