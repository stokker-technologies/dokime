/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { useMemo, useState } from "react";

/* === Local Imports === */

import { useNativeSession } from "@/app/native-session/context";
import { SvelteMount } from "@/components/SvelteMount";
import { Button } from "@/components/ui/button";
import CyclesHome from "@/features/cycles-home/CyclesHome.svelte";
import { NewCycleDialog } from "@/features/cycles-home/NewCycleDialog";
import { PageShell } from "@/shell/PageShell";

/* === Route === */

export const Route = createFileRoute("/cycles")({
  component: CyclesRoute,
});

function CyclesRoute() {
  const navigate = useNavigate();
  const session = useNativeSession();
  const [creating, setCreating] = useState(false);
  const actions = useMemo(() => <Button onClick={() => setCreating(true)}>New Cycle</Button>, []);

  function openCycle(cycleId: string) {
    void navigate({ params: { cycleId }, to: "/cycles/$cycleId" });
  }

  return (
    <PageShell actions={actions} trail={["Cycles"]}>
      <SvelteMount component={CyclesHome} props={{ session, onOpen: openCycle }} />
      <NewCycleDialog onCreated={openCycle} onOpenChange={setCreating} open={creating} />
    </PageShell>
  );
}
