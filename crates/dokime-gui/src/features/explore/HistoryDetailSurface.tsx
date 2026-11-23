// Dokime — SPDX-License-Identifier: Apache-2.0
import { useNativeSession } from "@/app/native-session/context";
import { SvelteMount } from "@/components/SvelteMount";
import HistoryDetail, { type HistoryDetailProps } from "./HistoryDetailSurface.svelte";

export function HistoryDetailSurface(props: Omit<HistoryDetailProps, "session">) {
  const session = useNativeSession();
  return <SvelteMount key={session.identity.generation} component={HistoryDetail} props={{ ...props, session }} />;
}
