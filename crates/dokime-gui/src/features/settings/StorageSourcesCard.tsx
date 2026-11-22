// Dokime — SPDX-License-Identifier: Apache-2.0
import { useNativeSessionOptional } from "@/app/native-session/context";
import { SvelteMount } from "@/components/SvelteMount";
import StorageSources from "./StorageSourcesCard.svelte";

export function StorageSourcesCard() {
  const session = useNativeSessionOptional();
  return session ? <SvelteMount key={session.identity.generation} component={StorageSources} props={{ session }} /> : null;
}
