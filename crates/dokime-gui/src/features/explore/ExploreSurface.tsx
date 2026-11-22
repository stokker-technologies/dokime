// Dokime — SPDX-License-Identifier: Apache-2.0
import { useNativeSession } from "@/app/native-session/context";
import { SvelteMount } from "@/components/SvelteMount";
import { PageShell } from "@/shell/PageShell";
import HistoryView, { type ExploreSurfaceProps } from "./ExploreSurface.svelte";

export function ExploreSurface(props: Omit<ExploreSurfaceProps, "session">) {
  const session = useNativeSession();
  return (
    <PageShell trail={props.comparing ? ["History", "Compare"] : ["History"]}>
      <SvelteMount component={HistoryView} props={{ ...props, session }} />
    </PageShell>
  );
}
