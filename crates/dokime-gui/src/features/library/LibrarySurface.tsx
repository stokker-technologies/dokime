/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { ErrorCard } from "@dokime/plots/components/ErrorCard";
import { failureOf } from "@dokime/plots/lib/notification-details";
import { useNavigate } from "@tanstack/react-router";
import { useEffect, useMemo, useRef, useState } from "react";

/* === Local Imports === */

import { DataTable } from "@dokime/plots/components/data-table/DataTable";
import { dataTableColumns } from "@dokime/plots/components/data-table/data-table-model";
import { filterTestSuites, selectionDraftKey, templateSelectionId } from "@/components/suite-picker/model";
import { SuiteTags } from "@/components/suite-picker/SuiteTags";
import { useTestSuitesQuery } from "@/components/suite-picker/server-state";
import { Button } from "@dokime/plots/components/ui/button";
import { Input } from "@dokime/plots/components/ui/input";
import { NewCycleDialog } from "@/features/cycles-home/NewCycleDialog";
import { CatalogIdentifier, pluginSearchText, SuiteProvenance } from "@/features/library/CatalogIdentity";
import { TemplateDialog } from "@/features/library/TemplateDialog";
import { useActiveCycle } from "@/features/library/use-active-cycle";
import { useInstalledPluginsQuery } from "@/features/settings/installed-plugins/installed-plugin-queries";
import { useBrand } from "@dokime/document/lib/brand";
import type { SuiteTemplateEntry, TestSuiteEntry } from "@dokime/plots/lib/types";
import { PageShell } from "@dokime/document/shell/PageShell";

/* === Constants === */

/* === Types === */

type LibrarySurfaceProps = {
  /** A cycle to make active on arrival, as Add Cases names it; a closed one is ignored. */
  appendTo?: string;
};

type OpenTemplate = {
  suite: TestSuiteEntry;
  template: SuiteTemplateEntry;
};

/* === Components === */

/**
 * Suite and template selection. Cases enter the active cycle through the
 * template dialog and nowhere else (concept 11), so this surface only has to
 * make the catalog scannable, hand one template to that dialog, and offer New
 * Cycle when there is no cycle to add to.
 */
export function LibrarySurface({ appendTo }: LibrarySurfaceProps) {
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
    if (!appendTo || applied.current === appendTo || !openCycles.some((cycle) => cycle.cycleId === appendTo)) return;
    applied.current = appendTo;
    choose(appendTo);
  }, [appendTo, choose, openCycles]);
  const suites = suitesQuery.data ?? null;
  const pluginsQuery = useInstalledPluginsQuery();
  const waitingPlugins = (pluginsQuery.data ?? []).filter((plugin) => ["failed", "connecting", "unavailable"].includes(plugin.status));
  const suiteFailure = suitesQuery.isError ? failureOf(suitesQuery.error, "Could Not Load Suites") : null;
  const waitingForSource = Boolean(suiteFailure?.plugin && waitingPlugins.some((plugin) => plugin.plugin_id === suiteFailure.plugin));
  const plugins = useMemo(() => new Map((pluginsQuery.data ?? []).map((plugin) => [plugin.plugin_id, plugin])), [pluginsQuery.data]);

  const [suiteSearch, setSuiteSearch] = useState("");
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

  const suiteRows = useMemo(
    () => filterTestSuites(suites ?? [], suiteSearch, (suite) => pluginSearchText(plugins.get(suite.source), brand.label)),
    [brand.label, plugins, suiteSearch, suites],
  );
  const column = dataTableColumns<OpenTemplate>();
  const columns = column.columns([
    column.accessor((row) => row.template.name, {
      id: "template",
      header: "Template",
      meta: { primary: true, filter: "text" },
      cell: ({ row }) => (
        <span className="block truncate font-medium" title={row.original.template.name}>
          {row.original.template.name}
        </span>
      ),
    }),
    // The identifier the CLI and Python client accept, whole: it wraps at its dots.
    column.accessor((row) => row.template.identifier ?? "", {
      id: "id",
      header: "ID",
      meta: { filter: "text", label: "ID", width: "w-80" },
      cell: ({ row }) =>
        row.original.template.identifier ? <CatalogIdentifier subject="template" value={row.original.template.identifier} /> : null,
    }),
    column.accessor((row) => row.suite.name, { id: "suite", header: "Suite", meta: { filter: "facet", width: "w-48" } }),
    column.accessor((row) => row.template.tags.join(", "), {
      id: "tags",
      header: "Tags",
      meta: { filter: "text", width: "w-40" },
      cell: ({ row }) => <SuiteTags tags={row.original.template.tags} />,
    }),
    column.accessor((row) => row.suite.source, {
      id: "source",
      header: "Source",
      meta: { filter: "facet", width: "w-64" },
      cell: ({ row }) => <SuiteProvenance plugin={plugins.get(row.original.suite.source)} source={row.original.suite.source} />,
    }),
  ]);

  const catalogFeedback =
    waitingPlugins.length || (suiteFailure && !waitingForSource) ? (
      <div className="grid gap-3">
        {waitingPlugins.map((plugin) => (
          <p key={plugin.plugin_id} role="status" className="text-sm text-muted-foreground">
            Waiting for Plugin {plugin.label}
          </p>
        ))}
        {suiteFailure && !waitingForSource ? <ErrorCard failure={suiteFailure} onRetry={() => void suitesQuery.refetch()} /> : null}
      </div>
    ) : undefined;

  return (
    <PageShell actions={actions} trail={["Library"]}>
      {active.failed ? <ErrorCard failure={failureOf(active.error, "Open Cycles Unavailable")} /> : null}

      <Input
        aria-label="Filter Templates"
        placeholder="Filter Templates"
        value={suiteSearch}
        onChange={(event) => setSuiteSearch(event.target.value)}
      />
      <section className="grid gap-3" aria-label="Library">
        {suiteRows.map(({ suite, members }, index) => (
          <details
            key={suite.suite_id}
            className="rounded-lg border bg-card p-4"
            open={Boolean(suiteSearch) || (index === 0 && Boolean(catalogFeedback)) || undefined}
          >
            <summary className="cursor-pointer text-sm font-semibold">
              {suite.name} <span className="ml-2 font-normal text-muted-foreground">{members.length} Templates</span>
            </summary>
            <div className="mt-3 grid gap-3">
              <DataTable
                aria-label={suite.name}
                columns={columns}
                data={members.map((template) => ({ suite, template }))}
                title={suite.name}
                description={suite.description || "Templates available in this suite."}
                emptyText="No Templates Match"
                failure={index === 0 ? catalogFeedback : undefined}
                getRowId={(row) => selectionDraftKey(row.suite.suite_id, templateSelectionId(row.template))}
                onRowClick={setOpenTemplate}
              />
            </div>
          </details>
        ))}
        {!suiteRows.length ? (
          <DataTable
            title="Templates"
            description="Templates available in the connected host's suites."
            columns={columns}
            data={[]}
            getRowId={(row) => selectionDraftKey(row.suite.suite_id, templateSelectionId(row.template))}
            emptyText="No Templates Match"
            loading={suitesQuery.isPending}
            failure={catalogFeedback}
          />
        ) : null}
      </section>

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
      <NewCycleDialog
        onCreated={(cycleId) => {
          choose(cycleId);
          void navigate({ search: { appendTo: cycleId }, to: "/library" });
        }}
        onOpenChange={setCreating}
        open={creating}
      />
    </PageShell>
  );
}

/* === Private Functions === */
