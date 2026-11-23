<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import type { ApplicationSettings, PreferencesSnapshot, SettingsScope } from "@generated/typescript/application-settings";
  import { createMutation, createQuery, type QueryClient } from "@tanstack/svelte-query";
  import { onDestroy, untrack } from "svelte";
  import type { ConnectedSession } from "../../app/native-session/session";
  import ErrorCard from "../../components/ErrorCard.svelte";
  import Input from "../../components/ui/Input.svelte";
  import SelectField from "../../components/ui/SelectField.svelte";
  import Switch from "../../components/ui/Switch.svelte";
  import { formatUnitLabel } from "../../lib/format";
  import { failureOf } from "../../lib/notification-details";
  import { toast } from "../../lib/notifications";
  import { preferencesMutation, preferencesQuery } from "./application-preferences";
  import { PREFERENCE_FIELDS, type PreferenceField, type PreferencesSection } from "./preference-fields";
  import { createDebouncedWriter } from "./settings-autosave";
  import SettingsCard from "./SettingsCard.svelte";
  import SettingsRow from "./SettingsRow.svelte";

  type FieldValue = string | number | boolean | string[] | null;
  let {
    queryClient,
    session,
    scope,
    advanced,
    section,
    onPendingChange,
  }: {
    queryClient: QueryClient;
    session: ConnectedSession | null;
    scope: SettingsScope;
    advanced: boolean;
    section?: PreferencesSection;
    onPendingChange?: (pending: boolean) => void;
  } = $props();

  const query = createQuery(
    () => preferencesQuery(scope, session),
    () => queryClient,
  );
  const save = createMutation(
    () => preferencesMutation(scope, session, queryClient),
    () => queryClient,
  );
  let draft = $state.raw<ApplicationSettings>();
  let edited = $state.raw(new Set<string>());
  let rawText = $state.raw<Record<string, string>>({});
  let invalid = $state.raw<Record<string, string>>({});
  let saveState = $state<"saved" | "saving" | "error">("saved");
  let failure = $state.raw<unknown>();
  let baseline: PreferencesSnapshot | undefined;
  let observed: PreferencesSnapshot | undefined;
  let writing = false;
  let version = 0;

  $effect(() => {
    const snapshot = query.data;
    if (snapshot && observed !== snapshot) {
      observed = snapshot;
      if (untrack(() => edited.size === 0) && !writing) {
        baseline = snapshot;
        draft = structuredClone(snapshot.settings);
      }
    }
  });
  $effect(() => onPendingChange?.(edited.size > 0 || Boolean(failure)));

  const writer = createDebouncedWriter<{ settings: ApplicationSettings; version: number }>(
    async (input) => {
      const previous = baseline;
      if (!previous) {
        return;
      }
      writing = true;
      try {
        const { appearance, client, report, telemetry, listener } = input.settings;
        const settings =
          scope === "local" ? { ...previous.settings, appearance, client } : { ...previous.settings, report, telemetry, listener };
        baseline = await save.mutateAsync({ previous, settings });
        toast.success("Settings Saved", { source: "Settings" });
        if (version === input.version) {
          edited = new Set();
        }
      } catch (error) {
        // A Newer Edit Supersedes A Rejected Draft Without Advancing Its Revision.
        if (validationFailure(error) && version !== input.version) {
          return;
        }
        failure = error;
        throw error;
      } finally {
        writing = false;
      }
    },
    (next) => (saveState = next.kind === "idle" ? "saved" : next.kind),
  );

  // Leaving A Section Flushes Valid Edits Instead Of Dropping Them.
  onDestroy(() => writer.flush());

  const fields = $derived(
    PREFERENCE_FIELDS.filter((field) => {
      const local = field.group === "appearance" || field.group === "client";
      return (
        local === (scope === "local") &&
        (section === "remote"
          ? field.group === "listener"
          : section === "appearance"
            ? field.group === "appearance"
            : field.group !== "listener")
      );
    }),
  );

  function change(field: PreferenceField, name: string, next: FieldValue) {
    if (!draft) {
      return;
    }
    if (validationFailure(failure ?? save.error)) {
      writer.reset();
      save.reset();
      failure = undefined;
    }
    version++;
    edited = new Set(edited).add(name);
    saveState = "saving";
    const errors = { ...invalid };
    const max = field.max ?? Number.MAX_SAFE_INTEGER;
    if (field.unit && (!Number.isSafeInteger(next) || Number(next) <= 0 || Number(next) > max)) {
      errors[name] = `Enter a whole number from 1 to ${max}.`;
    } else if (field.group === "listener" && field.key === "address" && !validAddress(next)) {
      errors[name] = "Enter a host and port from 1 to 65535.";
    } else {
      delete errors[name];
    }
    invalid = errors;
    draft = { ...draft, [field.group]: { ...draft[field.group], [field.key]: next } };
    if (Object.keys(errors).length) {
      writer.cancel();
    } else if (!failure) {
      writer.change({ settings: draft, version });
    }
  }

  async function reload() {
    // A Conflict Must Be Reviewed Against The Latest Revision Before Another Edit.
    const fresh = await query.refetch();
    if (fresh.error) {
      failure = fresh.error;
    } else if (fresh.data) {
      writer.reset();
      baseline = fresh.data;
      draft = structuredClone(fresh.data.settings);
      edited = new Set();
      rawText = {};
      invalid = {};
      failure = undefined;
      save.reset();
      saveState = "saved";
    }
  }

  function validAddress(value: FieldValue): boolean {
    if (typeof value !== "string" || !/^.+:\d+$/.test(value.trim())) {
      return false;
    }
    const port = Number(value.slice(value.lastIndexOf(":") + 1));
    return port >= 1 && port <= 65535;
  }

  function validationFailure(error: unknown): boolean {
    return typeof error === "object" && error !== null && "kind" in error && error.kind === "validation";
  }
</script>

{#snippet row(
  field: PreferenceField,
  settings: ApplicationSettings,
)}
  {@const name = `${field.group}.${field.key}`}
  {@const id = `${scope}-${name}`}
  {@const label = formatUnitLabel(field.label, field.unit)}
  {@const value = (settings[field.group] as unknown as Record<string, FieldValue>)[field.key]}
  {@const locked = field.group === "listener" && (field.key === "tls_enabled" || field.key === "authentication_enabled")}
  {@const error =
    invalid[name] ??
    (edited.has(name) && (failure || save.error) ? failureOf(failure ?? save.error, "Settings Operation Failed").message : undefined)}
  <SettingsRow {label} description={locked ? "This Host Requires TLS and Pairing Authentication." : field.help} for={id} {error}>
    {#snippet status()}
      <span data-state={error ? "error" : edited.has(name) ? saveState : "saved"} class="inline-block min-h-5"></span>
    {/snippet}
    {#if field.kind === "boolean"}
      <Switch disabled={locked} {id} aria-label={label} checked={Boolean(value)} onCheckedChange={(next) => change(field, name, next)} />
    {:else if field.options}
      <SelectField
        {id}
        {label}
        {name}
        class="w-full"
        value={String(value)}
        options={field.options.map((option) => ({
          value: option,
          label: option === "lttb" ? "LTTB" : option.replaceAll("_", " ").replace(/\b[a-z]/g, (letter) => letter.toUpperCase()),
        }))}
        onChange={(next) => change(field, name, next)}
      />
    {:else}
      <div class="flex w-full items-center gap-2">
        <Input
          {id}
          aria-label={label}
          aria-invalid={Boolean(error)}
          class={field.unit ? "text-right tabular-nums" : undefined}
          type={field.unit ? "number" : "text"}
          min={field.unit ? 1 : undefined}
          max={field.max}
          step={field.unit ? 1 : undefined}
          value={Array.isArray(value)
            ? (rawText[name] ?? value.join(", "))
            : typeof value === "string" || (typeof value === "number" && Number.isFinite(value))
              ? value
              : ""}
          oninput={(event) => {
            const input = event.currentTarget;
            if (field.kind === "list") {
              rawText = { ...rawText, [name]: input.value };
              change(field, name, input.value.split(/[\s,]+/).filter(Boolean));
            } else {
              change(field, name, field.unit ? input.valueAsNumber : input.value || null);
            }
          }}
        />
        {#if field.unit}
          <span class="text-xs text-muted-foreground">{field.unit}</span>
        {/if}
      </div>
    {/if}
  </SettingsRow>
{/snippet}

<SettingsCard title={scope === "local" ? "Local Preferences" : "Backend Preferences"}>
  {#if query.isPending}
    <p role="status" class="text-sm text-muted-foreground">Loading settings.</p>
  {:else if query.error}
    <div class="grid justify-items-stretch gap-3 p-4 text-sm">
      <ErrorCard failure={failureOf(query.error, "Preferences Unavailable")} onRetry={() => void query.refetch()} />
    </div>
  {:else if draft}
    <div>
      {#each fields.filter((field) => !field.advanced) as field (field.key)}
        {@render row(field, draft)}
      {/each}
      {#if advanced && fields.some((field) => field.advanced)}
        <h3 class="bg-muted/20 px-4 py-3 text-xs font-semibold">Advanced</h3>
        {#each fields.filter((field) => field.advanced) as field (field.key)}
          {@render row(field, draft)}
        {/each}
      {/if}
      {#if failure || save.error}
        <div class="grid gap-3 px-4 py-3">
          <ErrorCard notificationContext="action" failure={failureOf(failure ?? save.error, "Settings Save Failed")} onRetry={reload} />
        </div>
      {/if}
    </div>
  {/if}
</SettingsCard>
