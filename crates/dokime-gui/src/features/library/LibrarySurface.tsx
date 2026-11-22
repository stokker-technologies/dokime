/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { ErrorCard } from "@/components/ErrorCard";
import { failureOf } from "@/lib/notification-details";
import { useNavigate } from "@tanstack/react-router";
import { useEffect, useMemo, useRef, useState } from "react";

/* === Local Imports === */

import { SvelteMount } from "@/components/SvelteMount";
import { useNativeSession } from "@/app/native-session/context";
import { selectionDraftKey, templateSelectionId } from "@/components/suite-picker/model";
import { useTestSuitesQuery } from "@/components/suite-picker/server-state";
import { Button } from "@/components/ui/button";
import NewCycleDialog from "@/features/cycles-home/NewCycleDialog.svelte";
import { TemplateDialog } from "@/features/library/TemplateDialog";
import { useActiveCycle } from "@/features/library/use-active-cycle";
import { useInstalledPluginsQuery } from "@/features/settings/installed-plugins/installed-plugin-queries";
import { useBrand } from "@/lib/brand-context";
import { PageShell } from "@/shell/PageShell";
import TemplateCatalog, { type OpenTemplate } from "./TemplateCatalog.svelte";

/* === Constants === */

/* === Types === */

type LibrarySurfaceProps = {
  /** A cycle to make active on arrival, as Add Cases names it; a closed one is ignored. */
  appendTo?: string;
};

/* === Components === */

/**
 * Suite and template selection. Cases enter the active cycle through the
 * template dialog and nowhere else (concept 11), so this surface only has to
 * make the catalog scannable, hand one template to that dialog, and offer New
 * Cycle when there is no cycle to add to.
 */
export function LibrarySurface({ appendTo }: LibrarySurfaceProps) {
  const session = useNativeSession();
  const brand = useBrand();
  const navigate = useNavigate();
  const suitesQuery = useTestSuitesQuery();
  const active = useActiveCycle();
  const { choose, cycles: openCycles } = active;
  const destinationId = active.cycle?.cycleId;
  /* Add Cases arrives naming its cycle: that cycle becomes the active one, as
     the picker would make it, once the open cycles confirm it is open. Each
     arrival applies once, so a later pick is never reverted by a refetch. */
  const applied = useRef<string | undefined>(undefined);
  useEffect(() => {
    if (!appendTo || applied.current === appendTo || !openCycles.some((cycle) => cycle.cycleId === appendTo)) {
      return;
    }
    applied.current = appendTo;
    choose(appendTo);
  }, [appendTo, choose, openCycles]);
  const pluginsQuery = useInstalledPluginsQuery();

  const [openTemplate, setOpenTemplate] = useState<OpenTemplate | null>(null);
  const [creating, setCreating] = useState(false);

  const actions = useMemo(
    () =>
      destinationId ? (
        <Button
          className="h-[var(--control-height)]"
          onClick={() => void navigate({ params: { cycleId: destinationId }, to: "/cycles/$cycleId" })}
          type="button"
          variant="outline"
        >
          Open Cycle
        </Button>
      ) : (
        <Button className="h-[var(--control-height)]" onClick={() => setCreating(true)} type="button">
          New Cycle
        </Button>
      ),
    [destinationId, navigate],
  );

  return (
    <PageShell actions={actions} trail={["Library"]}>
      {active.failed ? <ErrorCard failure={failureOf(active.error, "Open Cycles Unavailable")} /> : null}

      <SvelteMount
        component={TemplateCatalog}
        props={{
          brandLabel: brand.label,
          suites: suitesQuery.data ?? [],
          plugins: pluginsQuery.data ?? [],
          failure: suitesQuery.isError ? failureOf(suitesQuery.error, "Could Not Load Suites") : undefined,
          loading: suitesQuery.isPending,
          onRetry: () => void suitesQuery.refetch(),
          onOpen: setOpenTemplate,
        }}
      />

      {openTemplate ? (
        <TemplateDialog
          activeCycle={active.cycle}
          hasOpenCycles={openCycles.length > 0}
          key={selectionDraftKey(openTemplate.suite.suite_id, templateSelectionId(openTemplate.template))}
          onOpenChange={(next) => {
            if (!next) setOpenTemplate(null);
          }}
          onChangeCycle={active.openPicker}
          onNewCycle={() => setCreating(true)}
          open
          suite={openTemplate.suite}
          template={openTemplate.template}
        />
      ) : null}

      {/* A cycle created here becomes the active one: the Library stays put,
          and an open template dialog can add to it at once. */}
      <SvelteMount
        component={NewCycleDialog}
        props={{
          onCreated: (cycleId) => {
            choose(cycleId);
            void navigate({ search: { appendTo: cycleId }, to: "/library" });
          },
          onOpenChange: setCreating,
          open: creating,
          session,
        }}
      />
    </PageShell>
  );
}

/* === Private Functions === */
