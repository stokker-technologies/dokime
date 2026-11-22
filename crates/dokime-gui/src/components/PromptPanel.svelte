<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { AlertTriangle, LoaderCircle } from "lucide-svelte";
  import { formatNumber } from "../lib/format";
  import { isWithdrawingAction, promptDecision, promptFieldError, type PromptValue, type PromptValues } from "../lib/prompts";
  import type { PromptField, PromptRequest } from "../lib/types";
  import { occurrenceKeys } from "../lib/utils";
  import PathPill from "./PathPill.svelte";
  import PromptFieldInput from "./PromptFieldInput.svelte";
  import Button from "./ui/Button.svelte";

  let {
    prompt,
    submittingAction = null,
    values,
    onAction,
    onValueChange,
  }: {
    prompt: PromptRequest;
    submittingAction?: string | null;
    values: PromptValues;
    onAction: (action: string, values?: PromptValues) => void;
    onValueChange: (field: PromptField, value: PromptValue) => void;
  } = $props();
  const identifier = $props.id();
  const headingId = `${identifier}-title`;
  const decision = $derived(promptDecision(prompt));
  const fields = $derived(decision ? prompt.fields.filter((field) => field.name !== decision?.field.name) : prompt.fields);
  const fieldErrors = $derived(Object.fromEntries(fields.map((field) => [field.name, promptFieldError(field, values[field.name])])));
  const invalidFields = $derived(Object.values(fieldErrors).some(Boolean));
  const withdrawing = $derived(prompt.actions.filter((action) => isWithdrawingAction(action.id)));
  const status = $derived(prompt.kind === "status");
  let touched = $state.raw<ReadonlySet<string>>(new Set());
  let attempted = $state(false);

  function change(field: PromptField, value: PromptValue) {
    if (!touched.has(field.name)) {
      touched = new Set([...touched, field.name]);
    }
    onValueChange(field, value);
  }

  function submit(action: string, overrides?: PromptValues) {
    if (!isWithdrawingAction(action) && invalidFields) {
      attempted = true;
      return;
    }
    onAction(action, overrides);
  }

  function verificationConditionLabel(prompt: PromptRequest) {
    const condition = prompt.verification?.condition;
    if (!condition) {
      return "Verification Condition";
    }
    if (condition.kind === "host") {
      return condition.label;
    }
    if (condition.kind === "record") {
      const detailLabels = Object.entries(condition.details ?? {}).map(([key, value]) => `${key}=${String(value)}`);
      return detailLabels.length ? `${condition.record_type} with ${detailLabels.join(", ")}` : condition.record_type;
    }
    const unit = condition.unit ? ` ${condition.unit}` : "";
    const tolerance = condition.operator === "within" ? ` +/- ${promptNumber(condition.tolerance ?? 0.5)}${unit}` : "";
    return `${condition.stream}.${condition.field} ${operatorLabel(condition.operator)} ${promptNumber(condition.target)}${unit}${tolerance}`;
  }

  function operatorLabel(operator: string) {
    if (operator === "within") {
      return "within";
    }
    if (operator === "gte") {
      return ">=";
    }
    if (operator === "lte") {
      return "<=";
    }
    if (operator === "gt") {
      return ">";
    }
    if (operator === "lt") {
      return "<";
    }
    return "=";
  }

  /* A verification threshold reads as up to three decimals; a number the payload
   * cannot express reads as a dash rather than as empty space. */
  function promptNumber(value: number): string {
    return Number.isFinite(value) ? formatNumber(value, { kind: { kind: { case: "normal", value: { maxFractionDigits: 3 } } } }) : "-";
  }

  function isTechnicalPromptSection(section: string) {
    const normalized = section.trim().toLowerCase();
    return (
      normalized.startsWith("error:") ||
      normalized.startsWith("traceback") ||
      normalized.startsWith("stack trace") ||
      normalized.includes("\nat ")
    );
  }

  function pathParts(text: string) {
    const parts = text.split(/([A-Za-z]:\\[^\s.,;)]*)/gu);
    return occurrenceKeys(parts, (part) => part).map(({ key, value }) => ({
      key,
      text: value,
      path: /^[A-Za-z]:\\/u.test(value),
    }));
  }

  function messageSections(text: string) {
    return text
      .split(/\n\s*\n/u)
      .map((section) => section.trim())
      .filter(Boolean);
  }
</script>

{#snippet title()}
  <strong id={headingId} class="text-base [overflow-wrap:anywhere]">{prompt.title}</strong>
{/snippet}

{#snippet message(
  text: string,
)}
  <p class="whitespace-pre-line text-sm text-muted-foreground [overflow-wrap:anywhere]">{text}</p>
{/snippet}

{#snippet inlinePaths(
  text: string,
)}
  {#each pathParts(text) as part (part.key)}
    {#if part.path}
      <PathPill value={part.text} />
    {:else}
      <span class="whitespace-pre-line">{part.text}</span>
    {/if}
  {/each}
{/snippet}

{#snippet formMessage(
  text: string,
)}
  {@const sections = messageSections(text)}
  {#if sections.length <= 1}
    <p class="text-sm text-muted-foreground [overflow-wrap:anywhere]">{@render inlinePaths(text)}</p>
  {:else}
    {@const lastSection = sections.at(-1) ?? ""}
    {@const hasAction = !isTechnicalPromptSection(lastSection)}
    {@const context = sections[0] ?? ""}
    {@const details = sections.slice(1, hasAction ? -1 : undefined)}
    <div class="grid gap-3 text-sm">
      {#if context}
        <p class="text-muted-foreground [overflow-wrap:anywhere]">{@render inlinePaths(context)}</p>
      {/if}
      {#if details.length}
        <div class="grid gap-2 rounded-lg border bg-muted/35 p-3">
          <p class="text-xs font-bold uppercase text-muted-foreground">Technical Detail</p>
          <pre
            class="whitespace-pre-wrap font-mono text-xs leading-relaxed text-muted-foreground [overflow-wrap:anywhere]"
          >{details.join("\n\n")}</pre>
        </div>
      {/if}
      {#if hasAction}
        <p class="rounded-lg border border-destructive/20 bg-destructive/5 p-3 font-medium text-foreground [overflow-wrap:anywhere]">
          {@render inlinePaths(lastSection)}
        </p>
      {/if}
    </div>
  {/if}
{/snippet}

{#if status || prompt.kind === "verification"}
  <section role={status ? "status" : undefined} aria-labelledby={headingId} class="grid min-w-0 gap-4">
    <div class="flex min-w-0 gap-3">
      <div
        class={[
          "flex size-9 shrink-0 items-center justify-center rounded-full",
          status ? "bg-primary/10 text-primary" : "bg-destructive/10 text-destructive",
        ]}
      >
        <LoaderCircle class="animate-spin" size={18} />
      </div>
      <div class="grid min-w-0 gap-1">
        <p class={["text-xs font-bold uppercase", status ? "text-primary" : "text-destructive"]}>
          {status ? "Working" : "Waiting for Verification"}
        </p>
        {@render title()}
        {#if prompt.message}
          {@render message(prompt.message)}
        {/if}
      </div>
    </div>
    {#if !status}
      <div class="rounded-lg border bg-muted/25 p-3">
        <p class="text-xs font-bold uppercase text-muted-foreground">Required Condition</p>
        <p class="mt-1 text-sm font-medium">{verificationConditionLabel(prompt)}</p>
      </div>
    {/if}
  </section>
{:else}
  <section aria-labelledby={headingId} class="grid min-w-0 gap-4">
    <div class="flex min-w-0 gap-3">
      <div class="flex size-9 shrink-0 items-center justify-center rounded-full bg-destructive/10 text-destructive">
        <AlertTriangle size={18} />
      </div>
      <div class="grid min-w-0 gap-1">
        <p class="text-xs font-bold uppercase text-destructive">Run Paused</p>
        {@render title()}
        {#if prompt.message}
          {@render formMessage(prompt.message)}
        {/if}
      </div>
    </div>

    {#if decision}
      <p class="text-sm font-medium [overflow-wrap:anywhere]">{decision.field.label}</p>
    {/if}

    {#if fields.length}
      <div class="grid gap-3">
        {#each fields as field (field.name)}
          {@const id = `${identifier}-${field.name}`}
          {@const error = fieldErrors[field.name]}
          <div class="grid gap-2">
            <label class="text-xs font-bold uppercase text-muted-foreground" for={id}>{field.label}</label>
            <PromptFieldInput {id} {field} value={values[field.name]} onChange={(value) => change(field, value)} />
            {#if error && (attempted || touched.has(field.name))}
              <p role="status" class="whitespace-pre-wrap text-sm text-muted-foreground">{error}</p>
            {/if}
          </div>
        {/each}
      </div>
    {/if}

    <div class="flex flex-wrap justify-between gap-2 border-t pt-3">
      {#if decision}
        <div class="flex flex-wrap gap-2">
          {#each withdrawing as action (action.id)}
            <Button disabled={submittingAction !== null} onclick={() => submit(action.id)} variant="outline">{action.label}</Button>
          {/each}
        </div>
        <div class="flex flex-wrap justify-end gap-2">
          {#each [false, true] as answer}
            <Button
              disabled={submittingAction !== null}
              onclick={() => submit(decision.action, { ...values, [decision.field.name]: answer })}
              variant={answer ? "primary" : "secondary"}
            >
              {submittingAction === decision.action ? "Submitting..." : answer ? "Yes" : "No"}
            </Button>
          {/each}
        </div>
      {:else}
        <div class="ml-auto flex flex-wrap justify-end gap-2">
          {#each prompt.actions as action (action.id)}
            <Button disabled={submittingAction !== null} onclick={() => submit(action.id)} variant={action.emphasis}>
              {submittingAction === action.id ? "Submitting..." : action.label}
            </Button>
          {/each}
        </div>
      {/if}
    </div>
  </section>
{/if}
