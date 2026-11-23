<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { createMutation } from "@tanstack/svelte-query";
  import { untrack } from "svelte";
  import type { ConnectedSession } from "../../app/native-session/session";
  import ErrorCard from "../../components/ErrorCard.svelte";
  import PromptPanel from "../../components/PromptPanel.svelte";
  import { respondNativePrompt } from "../../lib/api/native-prompts";
  import { failureOf } from "../../lib/notification-details";
  import { promptDefaults, promptResponseValues, type PromptValue, type PromptValues } from "../../lib/prompts";
  import type { PromptField, PromptRequest } from "../../lib/types";
  import { workflowQueryKeys } from "./workflow-query-keys";

  let { prompt, session }: { prompt: PromptRequest; session: ConnectedSession } = $props();
  let values = $state.raw(untrack(() => promptDefaults(prompt)));
  const response = createMutation(
    () => ({
      mutationFn: ({ action, correlationId, values }: { action: string; correlationId: string; values: Record<string, unknown> }) =>
        respondNativePrompt(session.identity, correlationId, action, values),
      onSuccess: async (_data, variables) => {
        const client = session.queryClient;
        client.setQueryData(workflowQueryKeys.prompts(), (current: unknown) =>
          Array.isArray(current) ? current.filter((item) => item?.correlation_id !== variables.correlationId) : current,
        );
        await client.invalidateQueries({ queryKey: workflowQueryKeys.prompts() });
      },
    }),
    () => session.queryClient,
  );

  function submit(action: string, overrides?: PromptValues) {
    response.mutate({ action, correlationId: prompt.correlation_id, values: promptResponseValues(prompt, overrides ?? values) });
  }

  function changeValue(field: PromptField, value: PromptValue) {
    values = { ...values, [field.name]: value };
  }
</script>

<div class="grid min-w-0 gap-3">
  <PromptPanel
    {prompt}
    {values}
    onAction={submit}
    onValueChange={changeValue}
    submittingAction={response.isPending ? response.variables?.action : null}
  />
  {#if response.isError}
    <ErrorCard notificationContext="action" failure={failureOf(response.error, "Response Not Accepted")} />
  {/if}
</div>
