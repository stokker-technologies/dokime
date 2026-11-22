/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import { ErrorCard } from "@/components/ErrorCard";
import { failureOf } from "@/lib/notification-details";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { Loader2 } from "lucide-react";
import { useMemo, useState } from "react";
import { useNativeSession } from "@/app/native-session/context";
import { SvelteMount } from "@/components/SvelteMount";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { checkpointKey, deleteNativeExecution, executionStatusLabel } from "@/lib/api/native-checkpoints";
import { checkpointStepStatus } from "@/lib/api/native-checkpoints";
import { DownloadReportAction } from "@/features/cycle/DownloadReportAction";
import { useCycleDetailQuery } from "@/features/cycle/cycle-queries";
import { cycleReportSealed } from "@/features/cycle/cycle-workspace-model";
import { timestampIso } from "@/lib/api/native-values";
import type { ExecutionBrowseEntry } from "@/lib/types/browse";
import { usePageChrome } from "@/shell/page-chrome";
import { CheckpointEvidence } from "./CheckpointEvidence";
import { type ExploreEntry, type ExploreTarget, exploreEntryKey, isDeletableExecution, unrunCaption } from "./explore-contracts";
import { fetchCheckpoint } from "./explore-data-source";
import { exploreQueryKeys, useExploreHistoryQuery } from "./explore-queries";
import HistoryExecutionsTable from "./HistoryExecutionsTable.svelte";

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
  const query = useExploreHistoryQuery("All");
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
  const [deleteError, setDeleteError] = useState<unknown>(null);
  const [deleteNotice, setDeleteNotice] = useState<string | null>(null);

  const actions = useMemo(
    () =>
      target.kind === "cycle" ? (
        <CycleReportActions cycleId={target.id} label={entry?.title ?? target.id} sealed={entry?.phase === "sealed"} />
      ) : null,
    [target.kind, target.id, entry?.title, entry?.phase],
  );
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
      setDeleteError(error);
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

  const emptyCycle = entry?.target.kind === "cycle" && entry.execution_count === 0 && entry.unrun_count === 0;

  return (
    <div aria-busy={query.isPending} className="grid content-start">
      <div className="grid w-full gap-[var(--section-gap)] p-[var(--shell-inset)]">
        {query.coverage && !query.coverage.discovery_complete ? (
          <p role="status" className="whitespace-pre-wrap text-sm text-muted-foreground">
            History is incomplete. Some executions may be unavailable.
          </p>
        ) : null}
        {deleteNotice ? (
          <p role="status" className="whitespace-pre-wrap text-sm text-muted-foreground">
            {deleteNotice}
          </p>
        ) : null}
        {unrun ? (
          <section aria-label="Case Never Run" className="grid gap-2 rounded-lg border p-3">
            <h2 className="font-medium">This Case Was Never Run</h2>
            <p className="max-w-[var(--prose-measure)] text-sm text-muted-foreground">
              {unrun.metadata.title} was planned and never started, so this execution has no acquisition, no steps and no records behind it.
            </p>
            {unrunCycleId && onOpenCycle ? (
              <Button className="justify-self-start px-0" onClick={() => onOpenCycle(unrunCycleId)} variant="link">
                Open the Cycle It Belongs To
              </Button>
            ) : null}
          </section>
        ) : null}
        <SvelteMount
          component={HistoryExecutionsTable}
          props={{
            rows: entry?.executions ?? [],
            emptyCycle,
            loading: query.isPending,
            failure: query.isError ? failureOf(query.error, "Data Unavailable") : undefined,
            onInspect: setSelected,
            onDelete: (execution) => {
              setDeleteError(null);
              setDeleteNotice(null);
              setDeleteTarget(execution);
            },
          }}
        />
        {/* The cases with nothing behind them are one line, not rows: the
            count stays visible without pretending each is inspectable. */}
        {!unrun && entry?.unrun_count ? (
          <p role="status" className="whitespace-pre-wrap text-sm text-muted-foreground">
            {unrunCaption(entry.unrun_count)}
          </p>
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
          {deleteError ? <ErrorCard notificationContext="action" failure={failureOf(deleteError, "Operation Failed")} /> : null}
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
  if (query.isPending)
    return (
      <p role="status" className="text-sm text-muted-foreground">
        Opening checkpoint…
      </p>
    );
  if (query.isError) return <ErrorCard failure={failureOf(query.error, "Data Unavailable")} />;
  const resolvedCheckpoint = query.data.checkpoint ?? checkpoint;
  if (!resolvedCheckpoint) {
    return (
      <ErrorCard failure={failureOf("The checkpoint identity is missing.", "Checkpoint Unavailable", "missing-checkpoint-identity")} />
    );
  }
  const execution = query.data.execution;
  if (!execution) {
    return <ErrorCard failure={failureOf("The execution document is missing.", "Checkpoint Unavailable", "missing-checkpoint-document")} />;
  }
  return (
    <section className="grid gap-3 rounded-lg border bg-card p-4" aria-label="Captured Execution">
      <h2 className="font-semibold">{query.data.cycle?.metadata?.label ?? entry.metadata.title}</h2>
      <p className="break-all text-xs text-muted-foreground">
        Checkpoint {(query.data.checkpoint?.position?.sequence ?? checkpoint?.position.sequence)?.toString()}
      </p>
      <dl className="grid gap-2 text-sm sm:grid-cols-2">
        <div>
          <dt className="text-muted-foreground">Recorded Status</dt>
          <dd>{executionStatusLabel(execution.summary?.state, execution.outcome, execution.startedAt !== undefined)}</dd>
        </div>
        <div>
          <dt className="text-muted-foreground">Acquisition</dt>
          <dd>{query.data.acquisitionFinished ? "Sealed" : "Incomplete"}</dd>
        </div>
        <div>
          <dt className="text-muted-foreground">Started</dt>
          <dd>{timestampIso(execution.startedAt) ?? "Not Recorded"}</dd>
        </div>
        <div>
          <dt className="text-muted-foreground">Sealed</dt>
          <dd>{timestampIso(execution.finishedAt) ?? "Not Recorded"}</dd>
        </div>
      </dl>
      <h3 className="text-sm font-semibold">Recorded Steps</h3>
      {query.data.steps.length ? (
        query.data.steps.map((step) => (
          <div className="flex justify-between gap-3 border-t py-2 text-sm" key={step.stepId}>
            <span>{step.metadata?.label ?? step.stepId}</span>
            <Badge variant="outline">{checkpointStepStatus(step.status)}</Badge>
          </div>
        ))
      ) : (
        <p className="text-sm text-muted-foreground">No steps recorded at this checkpoint.</p>
      )}
      <CheckpointEvidence checkpoint={resolvedCheckpoint} />
    </section>
  );
}

/** History uses the same sealed report download as the live workspace. */
function CycleReportActions({ cycleId, label, sealed }: { cycleId: string; label: string; sealed: boolean }) {
  const detail = useCycleDetailQuery(cycleId, !sealed);
  const plan = detail.data?.plan;
  const currentSealed = plan !== null && plan !== undefined && cycleReportSealed(plan.test_cycle.status, detail.data?.executions ?? []);
  const available = sealed || currentSealed;
  return <DownloadReportAction cycleId={cycleId} executionId={null} label={label} sealed={available} presentation="button" />;
}
