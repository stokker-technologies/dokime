// Dokime — SPDX-License-Identifier: Apache-2.0
import { useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { useNativeSessionOptional } from "@/app/native-session/context";
import { SvelteMount, useSvelteSlots } from "@/components/SvelteMount";
import { ApplicationPreferences } from "@/features/settings/ApplicationPreferences";
import { useBrand } from "@/lib/brand-context";
import RemoteAccess from "./RemoteSection.svelte";

export function RemoteSection({ advanced = false }: { advanced?: boolean }) {
  const session = useNativeSessionOptional();
  const queryClient = useQueryClient();
  const { label } = useBrand();
  const [preferencesPending, setPreferencesPending] = useState(false);
  const { rendered, portals } = useSvelteSlots({
    preferences: <ApplicationPreferences section="remote" advanced={advanced} onPendingChange={setPreferencesPending} />,
  });
  return (
    <>
      <SvelteMount
        key={session?.identity.generation}
        component={RemoteAccess}
        props={{ session, advanced, label, queryClient, preferencesPending, ...rendered }}
      />
      {portals}
    </>
  );
}
