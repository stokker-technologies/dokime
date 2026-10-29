/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import type { Row6 as GeneratedHistoryIssue } from "@generated/dokime/v1/HistoryMessages/BrowseExecutions/messages_pb";
import type {
  Row2 as GeneratedCheckpointContext,
  Row23 as GeneratedSeverity,
} from "@generated/dokime/v1/HistoryMessages/GetCheckpointContext/messages_pb";
import type { Row2 as GeneratedNativeCheckpointEvidence } from "@generated/dokime/v1/HistoryMessages/GetCheckpointEvidence/messages_pb";
import type { Row2 as GeneratedCheckpointTelemetryPage } from "@generated/dokime/v1/HistoryMessages/ListCheckpointTelemetry/messages_pb";
import { useQuery } from "@tanstack/react-query";
import { useState } from "react";
import { useNativeSession } from "@/app/native-session/context";
import { NotificationMessage } from "@/components/NotificationMessage";
import { Button } from "@/components/ui/button";
import {
  artifactStateLabel,
  checkpointFileKind,
  checkpointFileLabel,
  checkpointFileState,
  checkpointKey,
  downloadNativeCheckpointArtifact,
  type NativeCheckpointSession,
  nativeRecordText,
} from "@/lib/api/native-checkpoints";
import { saveNativeDocument } from "@/lib/api/native-delivery";
import { nativeValueObjectToJson, timestampIso } from "@/lib/api/native-values";
import type { Wire } from "@/lib/rpc/canonical";
import type { ExecutionCheckpoint } from "@/lib/types/view-contracts";
import {
  fetchCheckpointContext,
  fetchCheckpointEvidence,
  fetchCheckpointRecords,
  fetchCheckpointTelemetry,
  readCheckpointOutput,
} from "./explore-data-source";

const GUI_ERROR_PREVIEW_CHARS = 500;

type Severity = Wire<GeneratedSeverity>;
type CheckpointContext = Wire<GeneratedCheckpointContext>;
type CheckpointTelemetryPage = Wire<GeneratedCheckpointTelemetryPage>;
type HistoryIssue = Wire<GeneratedHistoryIssue>;
type NativeCheckpointEvidence = Wire<GeneratedNativeCheckpointEvidence>;

type Props = { checkpoint: ExecutionCheckpoint };
type SessionProps = Props & { session: NativeCheckpointSession };

export function CheckpointEvidence({ checkpoint }: Props) {
  const connected = useNativeSession();
  const session: NativeCheckpointSession = {
    ...connected.identity,
    own: connected.own,
  };
  const [panel, setPanel] = useState<string | null>(null);
  return (
    <div className="grid min-w-0 gap-3 border-t pt-3">
      <fieldset className="flex min-w-0 flex-wrap gap-2" aria-label="Evidence sections">
        {["Records", "Telemetry", "Files", "Context"].map((name) => (
          <Button
            key={name}
            size="sm"
            variant={panel === name ? "default" : "outline"}
            aria-pressed={panel === name}
            onClick={() => setPanel(name)}
          >
            {name}
          </Button>
        ))}
      </fieldset>
      {panel === "Records" && <Records checkpoint={checkpoint} session={session} />}
      {panel === "Telemetry" && <Telemetry checkpoint={checkpoint} session={session} />}
      {panel === "Files" && <Files checkpoint={checkpoint} session={session} />}
      {panel === "Context" && <Context checkpoint={checkpoint} session={session} />}
    </div>
  );
}

function Failure({ error, retry }: { error: Error; retry: () => void }) {
  return (
    <div className="grid gap-2 text-sm">
      <NotificationMessage source="Evidence" title="Evidence Could Not Load">
        {error.message}
      </NotificationMessage>
      <Button size="sm" variant="outline" className="justify-self-start" onClick={retry}>
        Retry Read
      </Button>
    </div>
  );
}

function Diagnostics({ issues, truncated }: { issues: HistoryIssue[]; truncated: boolean }) {
  if (!issues.length && !truncated) return null;
  return (
    <NotificationMessage tone="info">
      {issues.map((issue) => (
        <p key={`${issue.source ?? "history"}:${issue.dataset ?? "dataset"}:${issue.message}`}>{issue.message}</p>
      ))}
      {truncated && <p>Additional source issues were omitted.</p>}
    </NotificationMessage>
  );
}

function Pager({
  next,
  cursors,
  setCursors,
}: {
  next: string | null;
  cursors: (string | null)[];
  setCursors: (value: (string | null)[]) => void;
}) {
  return (
    <div className="flex items-center gap-2">
      <Button size="sm" variant="outline" disabled={cursors.length === 1} onClick={() => setCursors(cursors.slice(0, -1))}>
        Previous page
      </Button>
      <span className="text-xs">Page {cursors.length}</span>
      <Button size="sm" variant="outline" disabled={!next} onClick={() => next && setCursors([...cursors, next])}>
        Next page
      </Button>
    </div>
  );
}

function Records({ checkpoint, session }: SessionProps) {
  const [cursors, setCursors] = useState<(string | null)[]>([null]);
  const after = cursors.at(-1) ?? null;
  const query = useQuery({
    queryKey: ["checkpoint-records", session.generation, checkpointKey(checkpoint), after],
    queryFn: ({ signal }) => fetchCheckpointRecords(session, checkpoint, after, signal),
    staleTime: Infinity,
    gcTime: 0,
    retry: false,
  });
  if (query.isPending) return <NotificationMessage tone="info">Loading records…</NotificationMessage>;
  if (query.isError) return <Failure error={query.error} retry={() => void query.refetch()} />;
  return (
    <section className="grid min-w-0 gap-3" aria-label="Captured records">
      <Diagnostics issues={query.data.issues} truncated={query.data.issuesTruncated} />
      {!query.data.records.length && <p>No execution records at this checkpoint.</p>}
      {query.data.records.map((row) => {
        const record = row.record;
        if (!record || !row.epoch) throw new Error("Checkpoint record source identity is missing");
        const text = nativeRecordText(record);
        const message = record.message ?? record.error?.message ?? "";
        return (
          <details className="min-w-0 rounded border p-2 text-sm" key={`${row.epoch}:${record.sequence.toString()}:${record.recordId}`}>
            <summary className="cursor-pointer break-words">
              <span className="text-muted-foreground">
                {text.timestamp} · {text.severity} · {row.epoch}
              </span>
              <span className="block font-medium">{record.recordType || "Record"}</span>
              {message.slice(0, GUI_ERROR_PREVIEW_CHARS)}
            </summary>
            <pre className="max-h-72 overflow-auto whitespace-pre-wrap break-all pt-2 text-xs">
              {message}
              {Object.keys(text.details).length ? `\n${formatStructured(text.details)}` : ""}
            </pre>
          </details>
        );
      })}
      <Pager next={query.data.nextAfter ?? null} cursors={cursors} setCursors={setCursors} />
    </section>
  );
}

function Telemetry({ checkpoint, session }: SessionProps) {
  const query = useCheckpointEvidence(session, checkpoint);
  const [selected, setSelected] = useState<string | null>(null);
  if (query.isPending) return <NotificationMessage tone="info">Loading streams…</NotificationMessage>;
  if (query.isError) return <Failure error={query.error} retry={() => void query.refetch()} />;
  const table = selected ?? query.data.telemetryTables[0]?.table ?? null;
  return (
    <div className="grid min-w-0 gap-3">
      {!table ? (
        <p>No telemetry streams at this checkpoint.</p>
      ) : (
        <>
          <label className="grid gap-1 text-sm">
            Stream
            <select
              className="max-w-full rounded border bg-background p-2"
              value={table}
              onChange={(event) => setSelected(event.target.value)}
            >
              {query.data.telemetryTables.map((item) => (
                <option key={item.table} value={item.table}>
                  {item.schema?.metadata?.label || item.schema?.stream || item.table} · v{item.schema?.schemaVersion ?? "?"}
                </option>
              ))}
            </select>
          </label>
          <TelemetryRows checkpoint={checkpoint} session={session} table={table} />
        </>
      )}
    </div>
  );
}

function useCheckpointEvidence(session: NativeCheckpointSession, checkpoint: ExecutionCheckpoint) {
  return useQuery<NativeCheckpointEvidence>({
    queryKey: ["checkpoint-evidence", session.generation, checkpointKey(checkpoint)],
    queryFn: ({ signal }) => fetchCheckpointEvidence(session, checkpoint, signal),
    staleTime: Infinity,
    gcTime: 0,
    retry: false,
  });
}

function TelemetryRows({ checkpoint, session, table }: SessionProps & { table: string }) {
  const [cursors, setCursors] = useState<(string | null)[]>([null]);
  const after = cursors.at(-1) ?? null;
  const query = useQuery<CheckpointTelemetryPage>({
    queryKey: ["checkpoint-telemetry", session.generation, checkpointKey(checkpoint), table, after],
    queryFn: ({ signal }) => fetchCheckpointTelemetry(session, checkpoint, table, after, signal),
    staleTime: Infinity,
    gcTime: 0,
    retry: false,
  });
  if (query.isPending) return <NotificationMessage tone="info">Loading samples…</NotificationMessage>;
  if (query.isError) return <Failure error={query.error} retry={() => void query.refetch()} />;
  return (
    <section className="grid min-w-0 gap-3" aria-label="Captured telemetry">
      <Diagnostics issues={query.data.issues} truncated={query.data.issuesTruncated} />
      {!query.data.rows.length && <p>No samples at this checkpoint.</p>}
      {query.data.rows.map((row) => {
        const sample = row.sample;
        if (!row.epoch || !sample) throw new Error("Captured sample source identity is missing.");
        return (
          <div key={`${row.epoch}:${row.ordinal}:${sample.sampleId}`} className="min-w-0 border-b pb-2 text-sm">
            <p className="text-muted-foreground">{timestampIso(sample.timestamp) ?? "Unknown time"}</p>
            <ValueObjectView value={sample.values} />
            {sample.summary && (
              <details className="mt-2">
                <summary className="cursor-pointer text-xs text-muted-foreground">Summary</summary>
                <pre className="overflow-auto whitespace-pre-wrap break-all text-xs">
                  {formatStructured(nativeValueObjectToJson(sample.summary))}
                </pre>
              </details>
            )}
          </div>
        );
      })}
      <Pager next={query.data.nextAfter ?? null} cursors={cursors} setCursors={setCursors} />
    </section>
  );
}

function Download({ checkpoint, session, id, name }: SessionProps & { id: string; name: string }) {
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  async function download() {
    setBusy(true);
    setError(null);
    try {
      const artifact = await downloadNativeCheckpointArtifact(session, checkpoint, id);
      await saveNativeDocument(session, {
        kind: "artifact",
        // Artifact names may carry folders; the saved file takes the last segment.
        filename: (name || artifact.metadata.filename).split("/").at(-1) ?? name,
        bytes: artifact.bytes,
      });
    } catch (error) {
      setError(error instanceof Error ? error.message : String(error));
    } finally {
      setBusy(false);
    }
  }
  return (
    <div>
      <Button
        className="h-auto max-w-full whitespace-normal break-all"
        variant="outline"
        size="sm"
        disabled={busy}
        onClick={() => void download()}
      >
        {busy ? "Downloading…" : name}
      </Button>
      {error && <NotificationMessage>{error}</NotificationMessage>}
    </div>
  );
}

function Files({ checkpoint, session }: SessionProps) {
  const query = useCheckpointEvidence(session, checkpoint);
  const [selected, setSelected] = useState<string | null>(null);
  if (query.isPending) return <NotificationMessage tone="info">Loading files…</NotificationMessage>;
  if (query.isError) return <Failure error={query.error} retry={() => void query.refetch()} />;
  return (
    <div className="grid min-w-0 gap-3">
      <h3 className="font-medium">Output Files</h3>
      {!query.data.outputs.length && <p>No output files.</p>}
      {query.data.outputs.map((file) => (
        <Button
          className="h-auto justify-start whitespace-normal break-all"
          key={file.relativePath}
          variant="outline"
          onClick={() => setSelected(file.relativePath)}
        >
          {checkpointFileLabel(file)}
        </Button>
      ))}
      {selected && <FilePreview key={selected} checkpoint={checkpoint} session={session} path={selected} />}
      <h3 className="font-medium">Attachments</h3>
      {!query.data.artifacts.length && <p>No attachments.</p>}
      {query.data.artifacts.map((artifact) =>
        artifact.state?.HarmosVariant.case === "Complete" ? (
          <Download key={artifact.artifactId} checkpoint={checkpoint} session={session} id={artifact.artifactId} name={artifact.filename} />
        ) : (
          <p key={artifact.artifactId}>
            {artifact.filename} · {artifactStateLabel(artifact.state)}
          </p>
        ),
      )}
    </div>
  );
}

function FilePreview({ checkpoint, session, path }: SessionProps & { path: string }) {
  const [offset, setOffset] = useState<bigint>(0n);
  const query = useQuery({
    queryKey: ["checkpoint-output", session.generation, checkpointKey(checkpoint), path, offset.toString()],
    queryFn: ({ signal }) => readCheckpointOutput(session, checkpoint, path, offset, signal),
    staleTime: Infinity,
    gcTime: 0,
    retry: false,
  });
  if (query.isPending) return <NotificationMessage tone="info">Loading file bytes…</NotificationMessage>;
  if (query.isError) return <Failure error={query.error} retry={() => void query.refetch()} />;
  const file = query.data;
  const end = file.offset + BigInt(file.data.byteLength);
  const kind = checkpointFileKind(file.kind);
  return (
    <section className="grid min-w-0 gap-2" aria-label="File preview">
      <p className="break-all text-sm">
        {path} · {kind} · {checkpointFileState(file.state)} · bytes {file.offset.toString()}–{end.toString()} of{" "}
        {file.observedSizeBytes.toString()}
      </p>
      <pre className="max-h-80 overflow-auto whitespace-pre-wrap break-all rounded bg-muted p-2 text-xs">
        {kind === "text"
          ? new TextDecoder().decode(file.data)
          : Array.from(file.data, (byte) => byte.toString(16).padStart(2, "0")).join(" ")}
      </pre>
      <div className="flex gap-2">
        <Button
          size="sm"
          variant="outline"
          disabled={offset === 0n}
          onClick={() => setOffset(file.offset > 65536n ? file.offset - 65536n : 0n)}
        >
          Previous bytes
        </Button>
        <Button
          size="sm"
          variant="outline"
          disabled={end >= file.observedSizeBytes || !file.data.byteLength}
          onClick={() => setOffset(end)}
        >
          Next bytes
        </Button>
      </div>
    </section>
  );
}

function Context({ checkpoint, session }: SessionProps) {
  const query = useQuery<CheckpointContext>({
    queryKey: ["checkpoint-context", session.generation, checkpointKey(checkpoint)],
    queryFn: ({ signal }) => fetchCheckpointContext(session, checkpoint, signal),
    staleTime: Infinity,
    gcTime: 0,
    retry: false,
  });
  if (query.isPending) return <NotificationMessage tone="info">Loading captured context…</NotificationMessage>;
  if (query.isError) return <Failure error={query.error} retry={() => void query.refetch()} />;
  const context = query.data;
  return (
    <div className="grid min-w-0 gap-3">
      <h3 className="font-medium">{context.cycle?.cycle?.metadata?.label ?? "Execution context"}</h3>
      <h4 className="text-sm font-medium">Notes At Capture</h4>
      {!context.notes.length && <p>No notes captured.</p>}
      {context.notes.map((note) => (
        <div key={note.noteId} className="break-words text-sm">
          {note.author && <p className="text-muted-foreground">{note.author}</p>}
          <p className="whitespace-pre-wrap">{note.message}</p>
        </div>
      ))}
      <h4 className="text-sm font-medium">Health At Capture</h4>
      {!context.health.length && <p>No health items captured.</p>}
      {context.health.map((item) => (
        <p key={item.healthItemId} className="break-words text-sm">
          {severityLabel(item.severity)} · {item.message}
        </p>
      ))}
      <h4 className="text-sm font-medium">Prompts At Capture</h4>
      <p className="text-xs text-muted-foreground">Historical prompts are read-only.</p>
      {!context.prompts.length && <p>No prompts captured.</p>}
      {context.prompts.map((prompt) => (
        <div key={prompt.promptId} className="rounded border p-2">
          <p>{prompt.definition?.title || prompt.promptId}</p>
          <pre className="overflow-auto whitespace-pre-wrap break-all text-xs">
            {formatStructured(nativeValueObjectToJson(prompt.values))}
          </pre>
        </div>
      ))}
      <h4 className="text-sm font-medium">Captured Documents</h4>
      {context.executionDocuments.map((document, index) => {
        const presentation = document.presentation;
        return (
          <p key={presentation?.documentId || presentation?.metadata?.identifier || `document-${index}`}>
            {presentation?.metadata?.label || presentation?.documentId || "Captured document"}
          </p>
        );
      })}
    </div>
  );
}

function ValueObjectView({ value }: { value?: Parameters<typeof nativeValueObjectToJson>[0] }) {
  const object = nativeValueObjectToJson(value);
  if (!Object.keys(object).length) return <p className="text-muted-foreground">No values.</p>;
  return (
    <dl className="grid gap-1">
      {Object.entries(object).map(([name, item]) => (
        <div className="flex min-w-0 flex-wrap gap-2" key={name}>
          <dt className="font-medium">{name}</dt>
          <dd className="break-all">{typeof item === "string" ? item : formatStructured(item)}</dd>
        </div>
      ))}
    </dl>
  );
}

function severityLabel(value: Severity | undefined): string {
  return value?.HarmosVariant.case?.toLowerCase() ?? "unspecified";
}

function formatStructured(value: unknown): string {
  try {
    return JSON.stringify(value, (_key, item: unknown) => (typeof item === "bigint" ? item.toString() : item), 2) ?? "";
  } catch {
    return String(value);
  }
}
