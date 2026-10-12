/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import { useQuery, useQueryClient } from "@tanstack/react-query";
import { ArrowLeft, Loader2, Package, Trash2 } from "lucide-react";
import { useState } from "react";
import { useNativeSession } from "@/app/native-session/NativeSessionProvider";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import {
  checkpointKey,
  deleteNativeExecution,
  executionStatusLabel,
} from "@/lib/api/native-checkpoints";
import { downloadNativeCycleBundle } from "@/lib/api/native-content";
import { timestampIso } from "@/lib/api/native-values";
import type { ExecutionBrowseEntry } from "@/lib/types/view-contracts";
import { usePageChrome } from "@/shell/page-chrome";
import { CheckpointEvidence } from "./CheckpointEvidence";
import {
  type ExploreEntry,
  type ExploreTarget,
  exploreEntryKey,
  unrunCaption,
} from "./explore-contracts";
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

function SessionHistoryDetail({ target, onBack, onOpenCycle }: HistoryDetailProps) {
  const query = useExploreHistoryQuery();
  const session = useNativeSession();
  const queryClient = useQueryClient();
  const entry = query.data?.find(
    (cycle) => exploreEntryKey(cycle.target) === exploreEntryKey(target),
  );
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

  // The page bar carries the title, so the surface never repeats it.
  usePageChrome({
    title:
      selected?.metadata.title ??
      entry?.title ??
      (target.kind === "cycle" ? `Cycle ${target.id}` : `Execution ${target.id}`),
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
      receipt = await deleteNativeExecution(
        session.identity,
        execution.checkpoint,
        requestIdentifier,
      );
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
    await queryClient.invalidateQueries({ queryKey: exploreQueryKeys.history() }).catch(() => {
      if (session.identity.isCurrent()) {
        setDeleteNotice(`${notice} History could not refresh; reload to confirm the latest state.`);
      }
    });
  }

  return (
    <div className="grid content-start">
      <div className="mx-auto grid w-full max-w-5xl gap-4 p-[var(--shell-inset)]">
        {/* One toolbar row: Back on the left column, the bundle action on the
            right, both on the same fixed control height. */}
        <div className="grid grid-cols-[auto_minmax(0,1fr)_auto] items-center gap-[var(--shell-gap)]">
          <Button className="h-[var(--control-height)]" variant="ghost" onClick={onBack}>
            <ArrowLeft />
            History Library
          </Button>
          <span />
          {target.kind === "cycle" ? <CycleBundleActions cycleId={target.id} /> : null}
        </div>
        {query.isPending ? <p>Loading executions…</p> : null}
        {query.isError ? (
          <p role="alert" className="text-destructive">
            Unable to load history: {query.error.message}
          </p>
        ) : null}
        {query.coverage && !query.coverage.discovery_complete ? (
          <p role="status">History is incomplete. Some executions may be unavailable.</p>
        ) : null}
        {deleteNotice ? <p role="status">{deleteNotice}</p> : null}
        {unrun ? (
          <section aria-label="Case never run" className="grid gap-2 rounded-md border p-3">
            <h2 className="font-medium">This case was never run</h2>
            <p className="text-sm text-muted-foreground">
              {unrun.metadata.title} was planned and never started, so this execution has no
              acquisition, no steps and no records behind it.
            </p>
            {unrunCycleId && onOpenCycle ? (
              <Button
                className="justify-self-start px-0"
                onClick={() => onOpenCycle(unrunCycleId)}
                variant="link"
              >
                Open the cycle it belongs to
              </Button>
            ) : null}
          </section>
        ) : null}
        {entry?.executions?.map((execution) => (
          <section
            key={execution.metadata.execution_id}
            className="flex flex-wrap items-center justify-between gap-3 rounded-md border p-3"
          >
            <div className="min-w-0">
              <h2 className="font-medium">{execution.metadata.title}</h2>
              <p className="break-all text-xs text-muted-foreground">
                {execution.metadata.execution_id}
              </p>
            </div>
            <div className="flex items-center gap-2">
              <Badge variant="outline">
                {execution.availability === "interrupted"
                  ? "Interrupted"
                  : execution.metadata.status}
              </Badge>
              <Button
                variant="outline"
                size="sm"
                disabled={!execution.checkpoint}
                onClick={() => setSelected(execution)}
              >
                Inspect checkpoint
              </Button>
              {isDeletableExecution(execution) ? (
                <Button
                  aria-label={`Delete ${execution.metadata.title}`}
                  variant="destructive"
                  size="sm"
                  onClick={() => {
                    setDeleteError(null);
                    setDeleteNotice(null);
                    setDeleteTarget(execution);
                  }}
                >
                  <Trash2 data-icon="inline-start" />
                  Delete
                </Button>
              ) : null}
            </div>
          </section>
        ))}
        {/* The cases with nothing behind them are one line, not rows: the
            count stays visible without pretending each is inspectable. */}
        {!unrun && entry?.unrun.length ? (
          <p className="text-sm text-muted-foreground" role="status">
            {unrunCaption(entry.unrun.length)}
          </p>
        ) : null}
        {!query.isPending && !query.isError && !entry && !unrun ? (
          <p>No executions found in the loaded history.</p>
        ) : null}
        {query.hasNextPage ? (
          <Button
            variant="outline"
            disabled={query.isFetchingNextPage}
            onClick={() => void query.fetchNextPage()}
          >
            {query.isFetchingNextPage ? "Loading…" : "Load more executions"}
          </Button>
        ) : null}
        {selected?.checkpoint ? (
          <CheckpointInspection key={checkpointKey(selected.checkpoint)} entry={selected} />
        ) : null}
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
              Permanently delete {deleteTarget?.metadata.title ?? "this execution"} from history
              across configured sources? This cannot be undone.
            </DialogDescription>
          </DialogHeader>
          {deleteTarget?.checkpoint && deleteRetries[retryKey(deleteTarget)] ? (
            <p className="text-sm text-muted-foreground">
              Retry this deletion to confirm its result.
            </p>
          ) : null}
          {deleteError ? (
            <div className="grid gap-1" role="alert">
              <p>Unable to delete this execution: {deleteError}</p>
              <p className="text-sm text-muted-foreground">
                Resolve the issue, then retry deletion.
              </p>
            </div>
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
            <Button
              variant="destructive"
              disabled={deletePending || !deleteTarget}
              onClick={() => void confirmDeletion()}
            >
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
function findUnrunExecution(
  entries: ExploreEntry[] | undefined,
  executionId: string,
): ExecutionBrowseEntry | null {
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
  if (query.isPending) return <p role="status">Opening checkpoint…</p>;
  if (query.isError)
    return (
      <p role="alert" className="text-destructive">
        Unable to open checkpoint: {query.error.message}
      </p>
    );
  const resolvedCheckpoint = query.data.checkpoint ?? checkpoint;
  if (!resolvedCheckpoint) {
    return (
      <p role="alert" className="text-destructive">
        Unable to open checkpoint: the checkpoint identity is missing.
      </p>
    );
  }
  const execution = query.data.document?.execution;
  if (!execution) {
    return (
      <p role="alert" className="text-destructive">
        Unable to open checkpoint: the execution document is missing.
      </p>
    );
  }
  return (
    <section className="grid gap-3 rounded-md border bg-card p-4" aria-label="Captured execution">
      <h2 className="font-semibold">
        {query.data.document?.cycle?.metadata?.label ?? entry.metadata.title}
      </h2>
      <p className="break-all text-xs text-muted-foreground">
        Checkpoint{" "}
        {(query.data.checkpoint?.position?.sequence ?? checkpoint?.position.sequence)?.toString()}
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
            <Badge variant="outline">{step.status}</Badge>
          </div>
        ))
      ) : (
        <p className="text-sm text-muted-foreground">No steps recorded at this checkpoint.</p>
      )}
      <CheckpointEvidence checkpoint={resolvedCheckpoint} />
    </section>
  );
}

function CycleBundleActions({ cycleId }: { cycleId: string }) {
  const session = useNativeSession();
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function downloadBundle() {
    if (busy) {
      return;
    }
    setBusy(true);
    try {
      setError(null);
      await downloadNativeCycleBundle({ ...session.identity, own: session.own }, cycleId);
    } catch (error: unknown) {
      setError(error instanceof Error ? error.message : String(error));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="flex flex-wrap items-center gap-2">
      <Button disabled={busy} onClick={() => void downloadBundle()} size="sm" variant="outline">
        {busy ? (
          <Loader2 className="animate-spin" data-icon="inline-start" />
        ) : (
          <Package data-icon="inline-start" />
        )}
        {busy ? "Bundling…" : "Cycle ZIP"}
      </Button>
      {error ? <span className="text-xs text-destructive">{error}</span> : null}
    </div>
  );
}
