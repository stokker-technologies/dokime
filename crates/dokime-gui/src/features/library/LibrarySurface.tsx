/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import { useNavigate } from "@tanstack/react-router";
import { useEffect, useMemo, useRef, useState } from "react";
import { DataTable } from "@/components/data-table/DataTable";
import { dataTableColumns } from "@/components/data-table/data-table-model";
import { NotificationMessage } from "@/components/NotificationMessage";
import { filterTestSuites, selectionDraftKey, templateSelectionId } from "@/components/suite-picker/model";
import { SuiteTags } from "@/components/suite-picker/SuiteTags";
import { useTestSuitesQuery } from "@/components/suite-picker/server-state";
import { Button } from "@/components/ui/button";
import { NewCycleDialog } from "@/features/cycles-home/NewCycleDialog";
import { CatalogIdentifier, pluginSearchText, SuiteProvenance } from "@/features/library/CatalogIdentity";
import { TemplateDialog } from "@/features/library/TemplateDialog";
import { useActiveCycle } from "@/features/library/use-active-cycle";
import { useInstalledPluginsQuery } from "@/features/settings/installed-plugins/installed-plugin-queries";
import { useBrand } from "@/lib/brand";
import type { SuiteTemplateEntry, TestSuiteEntry } from "@/lib/types";
import { PageShell } from "@/shell/PageShell";

type LibrarySurfaceProps = {
  /** A cycle to make active on arrival, as Add Cases names it; a closed one is ignored. */
  appendTo?: string;
};

type OpenTemplate = {
  suite: TestSuiteEntry;
  template: SuiteTemplateEntry;
};

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
  const rows = useMemo(() => suiteRows.flatMap(({ suite, members }) => members.map((template) => ({ suite, template }))), [suiteRows]);
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

  return (
    <PageShell actions={actions} trail={["Library"]}>
      {active.failed ? <NotificationMessage>Open cycles could not be read. Reconnect and try again.</NotificationMessage> : null}

      {suitesQuery.isError ? <NotificationMessage>The suite catalog could not be read.</NotificationMessage> : null}
      <DataTable
        aria-label="Library"
        columns={columns}
        data={rows}
        getRowId={(row) => selectionDraftKey(row.suite.suite_id, templateSelectionId(row.template))}
        emptyText={suites === null ? "Loading Suites…" : "No Templates Match"}
        search={{ label: "Filter Templates", value: suiteSearch, onChange: setSuiteSearch }}
        onRowClick={setOpenTemplate}
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
