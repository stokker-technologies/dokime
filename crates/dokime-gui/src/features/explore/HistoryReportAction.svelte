<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { createMutation, createQuery } from "@tanstack/svelte-query";
  import type { ConnectedSession } from "../../app/native-session/session";
  import { cycleDetailQueryOptions, downloadReportOptions } from "../cycle/cycle-queries";
  import { cycleReportSealed } from "../cycle/cycle-workspace-model";
  import DownloadReportAction from "../cycle/DownloadReportAction.svelte";

  let {
    session,
    cycleId,
    label,
    sealed,
  }: {
    session: ConnectedSession;
    cycleId: string;
    label: string;
    sealed: boolean;
  } = $props();
  const client = () => session.queryClient;
  const detail = createQuery(() => ({ ...cycleDetailQueryOptions(session.identity, cycleId), enabled: !sealed }), client);
  const current = $derived(detail.data?.plan?.test_cycle);
  const available = $derived(sealed || Boolean(current && cycleReportSealed(current.status, detail.data?.executions ?? [])));
  const mutation = createMutation(() => {
    const { identity } = session;
    return downloadReportOptions(() => identity, cycleId);
  }, client);
</script>

<DownloadReportAction {mutation} executionId={null} {label} sealed={available} presentation="button" />
