/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { useNavigate } from "@tanstack/react-router";
import { ChevronRight, Search } from "lucide-react";
import { useMemo, useState } from "react";

/* === Local Imports === */

import {
  filterTestSuites,
  selectionDraftKey,
  templateSelectionId,
} from "@/components/suite-picker/model";
import { SuiteTags } from "@/components/suite-picker/SuiteTags";
import { useCycleLabelQuery, useTestSuitesQuery } from "@/components/suite-picker/server-state";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { TemplateDialog } from "@/features/library/TemplateDialog";
import { TemplateStepTimeline } from "@/features/library/TemplateStepTimeline";
import type { SuiteTemplateEntry, TestSuiteEntry } from "@/lib/types";
import { usePageChrome } from "@/shell/page-chrome";

/* === Types === */

type LibrarySurfaceProps = {
  /** When set, materialized cases join that cycle instead of a new one. */
  appendTo?: string;
};

type OpenTemplate = {
  suite: TestSuiteEntry;
  template: SuiteTemplateEntry;
};

/* === Constants === */

/* Suite and template rows share one grid so the count pill, the tag column and
   the action sit on the same verticals down the whole list. */
const ROW_COLUMNS =
  "grid grid-cols-[minmax(0,1fr)_7rem_auto_auto] items-center gap-[var(--shell-gap)]";

/* === Components === */

/**
 * Suite and template selection. Cases enter a cycle through the template
 * dialog and nowhere else (concept 11), so this surface only has to make the
 * catalog scannable and hand one template to that dialog.
 */
export function LibrarySurface({ appendTo }: LibrarySurfaceProps) {
  const navigate = useNavigate();
  const suitesQuery = useTestSuitesQuery();
  const cycleLabelQuery = useCycleLabelQuery(appendTo);
  const suites = suitesQuery.data ?? null;

  const [suiteSearch, setSuiteSearch] = useState("");
  const [expanded, setExpanded] = useState<Set<string>>(() => new Set());
  const [openTemplate, setOpenTemplate] = useState<OpenTemplate | null>(null);

  const actions = useMemo(
    () =>
      appendTo ? (
        <Button
          className="h-[var(--control-height)]"
          onClick={() => void navigate({ params: { cycleId: appendTo }, to: "/cycles/$cycleId" })}
          type="button"
          variant="outline"
        >
          Open Cycle
        </Button>
      ) : null,
    [appendTo, navigate],
  );

  usePageChrome({ actions, title: "Library" });

  const suiteRows = useMemo(
    () => filterTestSuites(suites ?? [], suiteSearch),
    [suiteSearch, suites],
  );
  const hasSearch = suiteSearch.trim().length > 0;

  return (
    <div className="grid content-start gap-4 p-[var(--shell-inset)]">
      {appendTo ? (
        <p className="text-muted-foreground">
          Cases materialized here are added to{" "}
          <span className="text-foreground">{cycleLabelQuery.data ?? appendTo}</span>.
        </p>
      ) : null}

      <section aria-labelledby="library-suites" className="grid gap-2">
        <div className="flex flex-wrap items-center justify-between gap-[var(--bar-gap)] py-[var(--bar-pad-y)] border-b">
          <h2 className="font-heading font-semibold tracking-wide" id="library-suites">
            Suites
          </h2>
          <div className="relative w-full sm:w-80">
            <Search
              aria-hidden
              className="pointer-events-none absolute top-1/2 left-2.5 size-3.5 -translate-y-1/2 text-muted-foreground"
            />
            <Input
              aria-label="Filter Templates"
              className="h-[var(--bar-control-height)] pl-8"
              onChange={(event) => setSuiteSearch(event.target.value)}
              placeholder="Filter Templates"
              value={suiteSearch}
            />
          </div>
        </div>

        {suitesQuery.isError ? (
          <p className="text-destructive" role="alert">
            The suite catalog could not be read.
          </p>
        ) : suites === null ? (
          <p className="text-muted-foreground">Loading suites…</p>
        ) : suiteRows.length === 0 ? (
          <p className="text-muted-foreground">
            No suite matches this filter. Search by path, tag, suite, template, or id.
          </p>
        ) : (
          <ul className="grid gap-4">
            {suiteRows.map(({ members, suite }) => {
              const isExpanded = expanded.has(suite.suite_id) || hasSearch;
              return (
                <li className="grid gap-2" key={suite.suite_id}>
                  {/* One crisp row on shared columns: label, template count,
                      tags. The identifier stays in the accessible name. */}
                  <div className={ROW_COLUMNS}>
                    <button
                      className="flex min-h-[var(--control-height)] min-w-0 items-center gap-2 text-left outline-none focus-visible:ring-2 focus-visible:ring-ring"
                      onClick={() =>
                        setExpanded((current) => {
                          const next = new Set(current);
                          if (!next.delete(suite.suite_id)) {
                            next.add(suite.suite_id);
                          }
                          return next;
                        })
                      }
                      title={suite.suite_id}
                      type="button"
                    >
                      <ChevronRight
                        aria-hidden
                        className={`size-4 shrink-0 transition-transform ${isExpanded ? "rotate-90" : ""}`}
                      />
                      <span className="truncate font-medium">{suite.name}</span>
                    </button>
                    <Badge className="justify-self-start" variant="outline">
                      {templateCountLabel(members.length)}
                    </Badge>
                    <SuiteTags tags={suite.tags} />
                    <span />
                  </div>

                  {isExpanded ? (
                    <ul className="grid">
                      {members.map((template) => (
                        <li
                          className="grid gap-2 border-b border-border/60 py-3"
                          data-testid={`suite-template-${templateSelectionId(template)}`}
                          key={selectionDraftKey(suite.suite_id, templateSelectionId(template))}
                        >
                          {/* The whole row opens the dialog; Configure names
                              the same outcome for anyone scanning for a verb. */}
                          <div className={ROW_COLUMNS}>
                            <button
                              className="min-h-[var(--control-height)] min-w-0 truncate text-left font-medium outline-none focus-visible:ring-2 focus-visible:ring-ring"
                              onClick={() => setOpenTemplate({ suite, template })}
                              title={templateSelectionId(template)}
                              type="button"
                            >
                              {template.name}
                            </button>
                            <span />
                            <SuiteTags tags={template.tags} />
                            <Button
                              className="h-[var(--control-height)]"
                              onClick={() => setOpenTemplate({ suite, template })}
                              size="sm"
                              type="button"
                              variant="outline"
                            >
                              Configure
                            </Button>
                          </div>
                          {/* One paragraph, then the full-width timeline. */}
                          {template.description ? (
                            <p className="min-w-0 text-xs text-muted-foreground">
                              {template.description}
                            </p>
                          ) : null}
                          {template.steps.length ? (
                            <TemplateStepTimeline template={template} />
                          ) : null}
                        </li>
                      ))}
                    </ul>
                  ) : null}
                </li>
              );
            })}
          </ul>
        )}
      </section>

      {openTemplate ? (
        <TemplateDialog
          appendTo={appendTo}
          key={selectionDraftKey(
            openTemplate.suite.suite_id,
            templateSelectionId(openTemplate.template),
          )}
          onOpenChange={(next) => {
            if (!next) setOpenTemplate(null);
          }}
          open
          suite={openTemplate.suite}
          targetCycleLabel={cycleLabelQuery.data}
          template={openTemplate.template}
        />
      ) : null}
    </div>
  );
}

/* === Private Functions === */

function templateCountLabel(count: number): string {
  return count === 1 ? "1 Template" : `${count} Templates`;
}
