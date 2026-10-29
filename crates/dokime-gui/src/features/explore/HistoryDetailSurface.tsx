/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import { useQuery, useQueryClient } from "@tanstack/react-query";
import { Eye, FileText, Loader2 } from "lucide-react";
import { useMemo, useState } from "react";
import { useNativeSession } from "@/app/native-session/context";
import { DataTable, DataTableRowActions } from "@/components/data-table/DataTable";
import { dataTableColumns } from "@/components/data-table/data-table-model";
import { NotificationMessage } from "@/components/NotificationMessage";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { DropdownMenuItem } from "@/components/ui/dropdown-menu";
import { checkpointKey, deleteNativeExecution, executionStatusLabel } from "@/lib/api/native-checkpoints";
import { mapStepStatus } from "@/lib/api/native-planning";
import { openNativeReport } from "@/lib/api/native-reports";
import { timestampIso } from "@/lib/api/native-values";
import type { ExecutionBrowseEntry } from "@/lib/types/view-contracts";
import { usePageChrome } from "@/shell/page-chrome";
import { CheckpointEvidence } from "./CheckpointEvidence";
import { type ExploreEntry, type ExploreTarget, exploreEntryKey, unrunCaption } from "./explore-contracts";
import { fetchCheckpoint } from "./explore-data-source";
import { exploreQueryKeys, useExploreHistoryQuery } from "./explore-queries";

type DeleteRetryRequests = Record<string, string>;

interface HistoryDetailProps {
  target: ExploreTarget;
  onBack: () => void;
  /** Open one cycle's history. The surface never owns the router. */
  onOpenCycle?: (cycleId: string) => void;
}

export function HistoryDetailSurface(props: HistoryDetailProps) {
  const session = useNativeSession();
  return <SessionHistoryDetail key={session.identity.generation} {...props} />;
}

function SessionHistoryDetail({ target, onOpenCycle }: HistoryDetailProps) {
  const query = useExploreHistoryQuery("", "All");
  const session = useNativeSession();
  const queryClient = useQueryClient();
  const entry = query.data?.find((cycle) => exploreEntryKey(cycle.target) === exploreEntryKey(target));
  /* A link can name a planned case that was never run — the rows never offer
     one, but a bookmark, a report or a copied URL can. It is not missing
     history; there is nothing behind it, and the cycle it belongs to is. */
  const unrun = target.kind === "execution" ? findUnrunExecution(query.data, target.id) : null;
  const unrunCycleId = unrun?.metadata.cycle_id ?? null;
  const [selected, setSelected] = useState<ExecutionBrowseEntry | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<ExecutionBrowseEntry | null>(null);
  const [deleteRetries, setDeleteRetries] = useState<DeleteRetryRequests>({});
  const [deletePending, setDeletePending] = useState(false);
  const [deleteError, setDeleteError] = useState<string | null>(null);
  const [deleteNotice, setDeleteNotice] = useState<string | null>(null);

  const actions = useMemo(() => (target.kind === "cycle" ? <CycleReportActions cycleId={target.id} /> : null), [target.kind, target.id]);
  // The page bar carries the title, so the surface never repeats it.
  usePageChrome({
    actions,
    title: selected?.metadata.title ?? entry?.title ?? (target.kind === "cycle" ? `Cycle ${target.id}` : `Execution ${target.id}`),
  });

  function retryKey(execution: ExecutionBrowseEntry): string {
    return JSON.stringify([session.identity.generation, execution.metadata.execution_id]);
  }

  async function confirmDeletion() {
    const execution = deleteTarget;
    if (!execution?.checkpoint || !isDeletableExecution(execution) || deletePending) return;

    const key = retryKey(execution);
    const requestIdentifier = deleteRetries[key] ?? crypto.randomUUID();
    setDeleteRetries((retries) => ({ ...retries, [key]: requestIdentifier }));
    setDeletePending(true);
    setDeleteError(null);
    setDeleteNotice(null);

    let receipt: Awaited<ReturnType<typeof deleteNativeExecution>>;
    try {
      receipt = await deleteNativeExecution(session.identity, execution.checkpoint, requestIdentifier);
    } catch (error: unknown) {
      if (!session.identity.isCurrent()) return;
      setDeleteError(error instanceof Error ? error.message : String(error));
      setDeletePending(false);
      return;
    }

    if (!session.identity.isCurrent()) return;
    setSelected(null);
    setDeleteTarget(null);
    setDeletePending(false);
    setDeleteRetries((retries) => {
      const remaining = { ...retries };
      delete remaining[key];
      return remaining;
    });
    const notice = receipt.local_complete
      ? `${execution.metadata.title} was deleted from execution history.`
      : `${execution.metadata.title} was removed from history. Local cleanup is still in progress.`;
    setDeleteNotice(notice);
    await queryClient.invalidateQueries({ queryKey: exploreQueryKeys.all }).catch(() => {
      if (session.identity.isCurrent()) {
        setDeleteNotice(`${notice} History could not refresh; reload to confirm the latest state.`);
      }
    });
  }

  return (
    <div aria-busy={query.isPending} className="grid content-start">
      <div className="grid w-full gap-[var(--section-gap)] p-[var(--shell-inset)]">
        {query.isPending ? <p>Loading executions…</p> : null}
        {query.isError ? <NotificationMessage>Unable to load history: {query.error?.message}</NotificationMessage> : null}
        {query.coverage && !query.coverage.discovery_complete ? (
          <NotificationMessage tone="info">History is incomplete. Some executions may be unavailable.</NotificationMessage>
        ) : null}
        {deleteNotice ? <NotificationMessage tone="info">{deleteNotice}</NotificationMessage> : null}
        {unrun ? (
          <section aria-label="Case Never Run" className="grid gap-2 rounded-lg border p-3">
            <h2 className="font-medium">This Case Was Never Run</h2>
            <p className="max-w-[var(--prose-measure)] text-sm text-muted-foreground">
              {unrun.metadata.title} was planned and never started, so this execution has no acquisition, no steps and no records behind it.
            </p>
            {unrunCycleId && onOpenCycle ? (
              <Button className="justify-self-start px-0" onClick={() => onOpenCycle(unrunCycleId)} variant="link">
                Open The Cycle It Belongs To
              </Button>
            ) : null}
          </section>
        ) : null}
        <DataTable
          aria-label="History Executions"
          data={entry?.executions ?? []}
          getRowId={(execution) => execution.metadata.execution_id}
          emptyText="No Executions In The Loaded History"
          columns={dataTableColumns<ExecutionBrowseEntry>().columns([
            dataTableColumns<ExecutionBrowseEntry>().accessor((execution) => execution.metadata.title, {
              id: "title",
              header: "Execution",
              meta: { filter: "text" },
              cell: ({ getValue, row }) => <span data-execution-id={row.original.metadata.execution_id}>{getValue()}</span>,
            }),
            dataTableColumns<ExecutionBrowseEntry>().accessor(
              (execution) => (execution.availability === "interrupted" ? "Interrupted" : execution.metadata.status),
              { id: "status", header: "Status", meta: { filter: "facet", width: "w-36" } },
            ),
            dataTableColumns<ExecutionBrowseEntry>().display({
              id: "actions",
              header: "Actions",
              cell: ({ row }) => (
                <DataTableRowActions
                  label={row.original.metadata.title}
                  primary={{ label: "Inspect", icon: Eye, disabled: !row.original.checkpoint, onSelect: () => setSelected(row.original) }}
                >
                  <DropdownMenuItem
                    variant="destructive"
                    disabled={!isDeletableExecution(row.original)}
                    onSelect={() => {
                      setDeleteError(null);
                      setDeleteNotice(null);
                      setDeleteTarget(row.original);
                    }}
                  >
                    Delete
                  </DropdownMenuItem>
                </DataTableRowActions>
              ),
            }),
          ])}
        />
        {/* The cases with nothing behind them are one line, not rows: the
            count stays visible without pretending each is inspectable. */}
        {!unrun && entry?.unrun_count ? <NotificationMessage tone="info">{unrunCaption(entry.unrun_count)}</NotificationMessage> : null}
        {entry?.target.kind === "cycle" && entry.execution_count === 0 && entry.unrun_count === 0 ? (
          <p>This sealed cycle contains no executions.</p>
        ) : null}
        {!query.isPending && !query.isError && !entry && !unrun ? <p>No executions found in the loaded history.</p> : null}
        {query.hasNextPage ? (
          <Button variant="outline" disabled={query.isFetchingNextPage} onClick={() => void query.fetchNextPage()}>
            {query.isFetchingNextPage ? "Loading…" : "Load More History"}
          </Button>
        ) : null}
        {selected?.checkpoint ? <CheckpointInspection key={checkpointKey(selected.checkpoint)} entry={selected} /> : null}
      </div>
      <Dialog
        open={deleteTarget !== null}
        onOpenChange={(open) => {
          if (open) return;
          if (deletePending) return;
          setDeleteTarget(null);
          setDeleteError(null);
        }}
      >
        <DialogContent showCloseButton={!deletePending}>
          <DialogHeader>
            <DialogTitle>Delete Execution</DialogTitle>
            <DialogDescription>
              Permanently delete {deleteTarget?.metadata.title ?? "this execution"} from history across configured sources? This cannot be
              undone.
            </DialogDescription>
          </DialogHeader>
          {deleteTarget?.checkpoint && deleteRetries[retryKey(deleteTarget)] ? (
            <p className="text-sm text-muted-foreground">Retry this deletion to confirm its result.</p>
          ) : null}
          {deleteError ? (
            <NotificationMessage>
              <p>Unable to delete this execution: {deleteError}</p>
              <p className="text-sm text-muted-foreground">Resolve the issue, then retry deletion.</p>
            </NotificationMessage>
          ) : null}
          <DialogFooter>
            <Button
              variant="outline"
              disabled={deletePending}
              onClick={() => {
                setDeleteTarget(null);
                setDeleteError(null);
              }}
            >
              Cancel
            </Button>
            <Button variant="destructive" disabled={deletePending || !deleteTarget} onClick={() => void confirmDeletion()}>
              {deletePending ? <Loader2 className="animate-spin" data-icon="inline-start" /> : null}
              {deletePending
                ? "Deleting…"
                : deleteError || (deleteTarget?.checkpoint && deleteRetries[retryKey(deleteTarget)])
                  ? "Retry Delete"
                  : "Delete Execution"}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}

/** The planned-but-never-started execution one direct link names, if any. */
function findUnrunExecution(entries: ExploreEntry[] | undefined, executionId: string): ExecutionBrowseEntry | null {
  for (const entry of entries ?? []) {
    const found = entry.unrun.find((execution) => execution.metadata.execution_id === executionId);
    if (found) return found;
  }
  return null;
}

function isDeletableExecution(execution: ExecutionBrowseEntry): boolean {
  return !!execution.checkpoint && execution.availability === "stored";
}

function CheckpointInspection({ entry }: { entry: ExecutionBrowseEntry }) {
  const session = useNativeSession();
  const checkpoint = entry.checkpoint;
  const query = useQuery({
    queryKey: ["execution-checkpoint", checkpoint ? checkpointKey(checkpoint) : null],
    queryFn: ({ signal }) => {
      if (!checkpoint) throw new Error("Execution has no checkpoint");
      return fetchCheckpoint(session.identity, checkpoint, signal);
    },
    enabled: !!checkpoint,
    staleTime: Number.POSITIVE_INFINITY,
  });
  if (query.isPending) return <NotificationMessage tone="info">Opening checkpoint…</NotificationMessage>;
  if (query.isError) return <NotificationMessage>Unable to open checkpoint: {query.error.message}</NotificationMessage>;
  const resolvedCheckpoint = query.data.checkpoint ?? checkpoint;
  if (!resolvedCheckpoint) {
    return <NotificationMessage>Unable to open checkpoint: the checkpoint identity is missing.</NotificationMessage>;
  }
  const execution = query.data.document?.execution;
  if (!execution) {
    return <NotificationMessage>Unable to open checkpoint: the execution document is missing.</NotificationMessage>;
  }
  return (
    <section className="grid gap-3 rounded-lg border bg-card p-4" aria-label="Captured Execution">
      <h2 className="font-semibold">{query.data.document?.cycle?.metadata?.label ?? entry.metadata.title}</h2>
      <p className="break-all text-xs text-muted-foreground">
        Checkpoint {(query.data.checkpoint?.position?.sequence ?? checkpoint?.position.sequence)?.toString()}
      </p>
      <dl className="grid gap-2 text-sm sm:grid-cols-2">
        <div>
          <dt className="text-muted-foreground">Recorded status</dt>
          <dd>{executionStatusLabel(execution.status)}</dd>
        </div>
        <div>
          <dt className="text-muted-foreground">Acquisition</dt>
          <dd>{query.data.acquisitionFinished ? "Finished" : "Incomplete"}</dd>
        </div>
        <div>
          <dt className="text-muted-foreground">Started</dt>
          <dd>{timestampIso(execution.startedAt) ?? "Not recorded"}</dd>
        </div>
        <div>
          <dt className="text-muted-foreground">Finished</dt>
          <dd>{timestampIso(execution.finishedAt) ?? "Not recorded"}</dd>
        </div>
      </dl>
      <h3 className="text-sm font-semibold">Recorded Steps</h3>
      {(execution.steps ?? []).length ? (
        (execution.steps ?? []).map((step) => (
          <div className="flex justify-between gap-3 border-t py-2 text-sm" key={step.stepId}>
            <span>{step.metadata?.label ?? step.stepId}</span>
            <Badge variant="outline">{mapStepStatus(step.status)}</Badge>
          </div>
        ))
      ) : (
        <p className="text-sm text-muted-foreground">No steps recorded at this checkpoint.</p>
      )}
      <CheckpointEvidence checkpoint={resolvedCheckpoint} />
    </section>
  );
}

/** Opens the cycle's report: the sealed one, or one generated when it is missing. */
function CycleReportActions({ cycleId }: { cycleId: string }) {
  const session = useNativeSession();
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function openReport() {
    if (busy) {
      return;
    }
    setBusy(true);
    try {
      setError(null);
      await openNativeReport(session.identity, { kind: "cycle_report", cycleId });
    } catch (error: unknown) {
      setError(error instanceof Error ? error.message : String(error));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="flex flex-wrap items-center gap-2">
      <Button disabled={busy} onClick={() => void openReport()} size="sm" variant="outline">
        {busy ? <Loader2 className="animate-spin" data-icon="inline-start" /> : <FileText data-icon="inline-start" />}
        {busy ? "Opening…" : "Open Report"}
      </Button>
      {error ? <NotificationMessage>{error}</NotificationMessage> : null}
    </div>
  );
}
