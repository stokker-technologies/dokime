/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { Clipboard, Download, FileText, RefreshCw } from "lucide-react";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { NotificationMessage } from "@/components/NotificationMessage";

/* === Local Imports === */

import { useNativeSessionOptional } from "@/app/native-session/context";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { fetchNativeArtifactText, followNativeExecutionFile, readNativeExecutionFileWindow } from "@/lib/api/native-content";
import { formatArtifactSize } from "@/lib/artifact-tree";
import { artifactLabel } from "@/lib/dashboard";
import {
  advanceFollowCursor,
  connectingFollowCursor,
  type ExecutionFileFollowCursor,
  followTailOffset,
  pageExecutionFile,
} from "@/lib/execution-file-paging";
import {
  appendBoundedText,
  detectTextArtifactLanguage,
  formatTextArtifactContent,
  highlightTextLine,
  shouldFollowArtifact,
  TEXT_FILE_BUFFER_MAX_CHARS,
  TEXT_FILE_FOLLOW_TAIL_BYTES,
  TEXT_FILE_PREVIEW_BUDGET_BYTES,
  type TextArtifactLanguage,
  type TextHighlightKind,
  type TextHighlightToken,
} from "@/lib/execution-file-text";
import type { NativeStream } from "@/lib/rpc/transport";
import type { ArtifactRecord, ExecutionFileFollowEvent } from "@/lib/types";
import { occurrenceKeys } from "@/lib/utils";

/* === Constants === */

const GRPC_STATUS_NAMES = [
  "OK",
  "CANCELLED",
  "UNKNOWN",
  "INVALID_ARGUMENT",
  "DEADLINE_EXCEEDED",
  "NOT_FOUND",
  "ALREADY_EXISTS",
  "PERMISSION_DENIED",
  "RESOURCE_EXHAUSTED",
  "FAILED_PRECONDITION",
  "ABORTED",
  "OUT_OF_RANGE",
  "UNIMPLEMENTED",
  "INTERNAL",
  "UNAVAILABLE",
  "DATA_LOSS",
  "UNAUTHENTICATED",
] as const;

/* === Types === */

type TextViewerArtifact = Pick<
  ArtifactRecord,
  "artifact_id" | "downloadHref" | "href" | "label" | "path" | "scope" | "size_bytes" | "stream_state"
>;

type TextArtifactViewerProps = {
  artifact: TextViewerArtifact | null;
  open: boolean;
  onOpenChange: (open: boolean) => void;
};

/** What the status line says; every read ends in one of the settled states. */
type ViewerStatus =
  | { kind: "loading" }
  | { kind: "live"; bytes: number }
  | { kind: "paused"; bytes: number }
  | { kind: "snapshot"; bytes: number; observed: number }
  | { kind: "finished"; bytes: number }
  | { kind: "abandoned"; bytes: number }
  | { kind: "error"; message: string };

/* === Private Functions === */

/** The connection a read needs. Unreachable while nothing can be opened. */
function requireSession(session: ReturnType<typeof useNativeSessionOptional>) {
  if (!session) throw new Error("A connected native backend is required.");
  return session;
}

/** The error text, with the gRPC status name when the backend refused the call. */
function describeReadError(error: unknown): string {
  const message = error instanceof Error ? error.message : String(error);
  const detail = (error as { kind?: unknown; detail?: { code?: unknown } } | null)?.detail;
  if ((error as { kind?: unknown } | null)?.kind === "grpc" && typeof detail?.code === "number") {
    const name = GRPC_STATUS_NAMES[detail.code] ?? `${detail.code}`;
    return `${message} (gRPC ${name})`;
  }
  return message;
}

function describeStatus(status: ViewerStatus): string {
  switch (status.kind) {
    case "loading":
      return "Loading";
    case "live":
      return `Live · ${formatArtifactSize(status.bytes)}`;
    case "paused":
      return `Paused · ${formatArtifactSize(status.bytes)}`;
    case "snapshot":
      return `${formatArtifactSize(status.bytes)} of ${formatArtifactSize(status.observed)}`;
    case "finished":
      return `Finished · ${formatArtifactSize(status.bytes)}`;
    case "abandoned":
      return `Abandoned · ${formatArtifactSize(status.bytes)}`;
    case "error":
      return status.message;
  }
}

/** A resumable follow is one that knows its identity and has not settled. */
function resumable(cursor: ExecutionFileFollowCursor | null): cursor is ExecutionFileFollowCursor {
  return cursor?.identity != null && cursor.phase === "producing";
}

/* === Components === */

export function TextArtifactViewer({ artifact, open, onOpenChange }: TextArtifactViewerProps) {
  /* The artifact explorer mounts this dialog beside the tree whether or not
     anything is open, and the frozen report mounts that tree. Asking for the
     connection here would make a file with one text artifact fail to render,
     so the connection is optional and only the reads insist on it — and those
     run when a reader opens a file, which the report offers no way to do. */
  const session = useNativeSessionOptional();
  const executionId = artifact?.scope?.test_execution_id ?? null;
  const artifactId = artifact?.artifact_id ?? null;
  const path = artifact?.path ?? null;
  const streamed = shouldFollowArtifact(artifact);
  // Follow defaults to the file's own state; a reader's toggle holds for that file only.
  const fileKey = `${executionId ?? ""}\u0000${artifactId ?? ""}\u0000${path ?? ""}`;
  const [followChoice, setFollowChoice] = useState<{ fileKey: string; follow: boolean } | null>(null);
  const follow = followChoice?.fileKey === fileKey ? followChoice.follow : streamed;
  const [text, setText] = useState("");
  const [truncated, setTruncated] = useState(false);
  const [status, setStatus] = useState<ViewerStatus>({ kind: "loading" });
  const [generation, setGeneration] = useState(0);
  const [copyStatus, setCopyStatus] = useState<"idle" | "copied" | "failed">("idle");
  const [wrap, setWrap] = useState(true);
  const textRef = useRef<HTMLDivElement | null>(null);
  // The accepted follow position survives a Follow toggle so a resume appends.
  const cursorRef = useRef<ExecutionFileFollowCursor | null>(null);

  // Phase: Reset Per Opened File Or Explicit Refresh
  // biome-ignore lint/correctness/useExhaustiveDependencies: a new file or refresh starts over
  useEffect(() => {
    cursorRef.current = null;
    setText("");
    setTruncated(false);
    setStatus({ kind: "loading" });
    setCopyStatus("idle");
  }, [artifactId, executionId, path, generation]);

  // Phase: One Owned Read Per File, Follow Mode And Refresh
  // biome-ignore lint/correctness/useExhaustiveDependencies: generation restarts the read
  useEffect(() => {
    if (!open || !artifactId || !path) {
      return;
    }
    const controller = new AbortController();
    const signal = controller.signal;
    let stream: NativeStream<ExecutionFileFollowEvent> | null = null;

    const readStatic = async () => {
      const connected = requireSession(session);
      const content = await fetchNativeArtifactText({ ...connected.identity, own: connected.own }, artifactId, signal);
      if (signal.aborted) return;
      const bytes = new TextEncoder().encode(content).length;
      setText(content.slice(0, TEXT_FILE_BUFFER_MAX_CHARS));
      setTruncated(content.length > TEXT_FILE_BUFFER_MAX_CHARS);
      setStatus({ kind: "finished", bytes });
    };

    const readPages = async (execution: string) => {
      const connected = requireSession(session);
      const page = await pageExecutionFile(
        (offset, maxBytes) => readNativeExecutionFileWindow(connected.identity, execution, path, offset, maxBytes),
        { budgetBytes: TEXT_FILE_PREVIEW_BUDGET_BYTES },
      );
      if (signal.aborted) return;
      setText(new TextDecoder().decode(page.data).slice(0, TEXT_FILE_BUFFER_MAX_CHARS));
      setTruncated(page.startOffset > 0 || page.stop !== "complete");
      if (page.stop === "stalled") {
        setStatus({
          kind: "error",
          message: `Read stalled at ${formatArtifactSize(page.nextOffset)} of ${formatArtifactSize(page.observedSizeBytes)}; refresh to retry.`,
        });
      } else if (page.state === "finalized" && page.stop === "complete") {
        setStatus({ kind: "finished", bytes: page.nextOffset });
      } else if (page.state === "interrupted" && page.stop === "complete") {
        setStatus({ kind: "abandoned", bytes: page.nextOffset });
      } else {
        setStatus({ kind: "snapshot", bytes: page.nextOffset, observed: page.observedSizeBytes });
      }
    };

    const readFollowing = async (execution: string) => {
      const connected = requireSession(session);
      const previous = cursorRef.current;
      let cursor = resumable(previous) ? connectingFollowCursor(previous.identity, previous.offset) : connectingFollowCursor();
      // A fresh follow may jump once to the tail; a resume continues its text.
      let tailJump = !resumable(previous);
      if (tailJump) {
        setText("");
        setTruncated(false);
      }
      const decoder = new TextDecoder();
      for (;;) {
        const subscription = await followNativeExecutionFile(connected.identity, execution, path, {
          identity: cursor.identity,
          offset: cursor.offset,
        });
        stream = subscription;
        if (signal.aborted) {
          await subscription.cancel();
          return;
        }
        let jumped = false;
        for await (const event of subscription) {
          if (signal.aborted) return;
          cursor = advanceFollowCursor(cursor, event);
          if (event.kind === "metadata") {
            const tail = followTailOffset(event.availableLength, TEXT_FILE_FOLLOW_TAIL_BYTES);
            if (tailJump && tail > event.offset) {
              cursor = connectingFollowCursor(event.identity, tail);
              tailJump = false;
              jumped = true;
              setTruncated(true);
              break;
            }
            tailJump = false;
            setStatus(cursor.phase === "producing" ? { kind: "live", bytes: cursor.offset } : { kind: "loading" });
          } else if (event.kind === "chunk") {
            const decoded = decoder.decode(event.data, { stream: true });
            setText((current) => appendBoundedText(current, decoded));
            if (cursor.phase === "producing") {
              setStatus({ kind: "live", bytes: cursor.offset });
            }
          } else {
            const rest = decoder.decode();
            if (rest) setText((current) => appendBoundedText(current, rest));
            cursorRef.current = cursor;
            setStatus({ kind: event.phase, bytes: cursor.offset });
            return;
          }
          cursorRef.current = cursor;
        }
        if (signal.aborted) return;
        if (!jumped) {
          throw new Error("Content.FollowExecutionFile ended without a terminal frame.");
        }
        await subscription.cancel();
      }
    };

    const run = async () => {
      if (!executionId) {
        await readStatic();
      } else if (follow) {
        await readFollowing(executionId);
      } else if (cursorRef.current) {
        // Pausing keeps what the follow delivered; resuming appends from there.
        const cursor = cursorRef.current;
        setStatus(
          cursor.phase === "producing"
            ? { kind: "paused", bytes: cursor.offset }
            : {
                kind: cursor.phase === "abandoned" ? "abandoned" : "finished",
                bytes: cursor.offset,
              },
        );
      } else {
        await readPages(executionId);
      }
    };

    void run().catch((caught: unknown) => {
      if (!signal.aborted) {
        setStatus({ kind: "error", message: describeReadError(caught) });
      }
    });
    return () => {
      controller.abort();
      void stream?.cancel().catch(() => undefined);
    };
  }, [artifactId, executionId, follow, generation, open, path, session]);

  // biome-ignore lint/correctness/useExhaustiveDependencies: new text keeps a follow pinned to the end
  useEffect(() => {
    if (follow && textRef.current) {
      textRef.current.scrollTop = textRef.current.scrollHeight;
    }
  }, [follow, text]);

  const title = artifact ? artifactLabel(artifact.label) : "File";
  const description = useMemo(() => {
    if (!artifact) {
      return "";
    }
    return [artifact.path, formatArtifactSize(artifact.size_bytes)].filter(Boolean).join(" · ");
  }, [artifact]);
  const language = useMemo(() => detectTextArtifactLanguage(artifact), [artifact]);
  const displayText = useMemo(() => formatTextArtifactContent(text, artifact), [artifact, text]);
  const lines = useMemo(() => displayText.split("\n"), [displayText]);

  const copyText = useCallback(async () => {
    try {
      await navigator.clipboard.writeText(displayText);
      setCopyStatus("copied");
    } catch {
      setCopyStatus("failed");
    }
  }, [displayText]);

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-h-[min(92vh,--spacing(232))] w-[min(96vw,--spacing(368))] max-w-none overflow-hidden p-0 sm:max-w-none">
        <DialogHeader className="border-b px-4 py-3 pr-12">
          <div className="flex min-w-0 items-start justify-between gap-3">
            <div className="min-w-0">
              <DialogTitle className="truncate">{title}</DialogTitle>
              <DialogDescription className="truncate">{description}</DialogDescription>
            </div>
            <div className="flex shrink-0 items-center gap-2">
              {streamed ? (
                <div className="flex items-center gap-2">
                  <Switch
                    checked={follow}
                    id="text-artifact-follow"
                    size="sm"
                    onCheckedChange={(checked) => setFollowChoice({ fileKey, follow: checked })}
                  />
                  <Label className="text-xs text-muted-foreground" htmlFor="text-artifact-follow">
                    Follow
                  </Label>
                </div>
              ) : null}
              <Button
                aria-label={wrap ? "Disable Wrapping" : "Enable Wrapping"}
                aria-pressed={wrap}
                size="icon-sm"
                title={wrap ? "Disable Wrapping" : "Enable Wrapping"}
                type="button"
                variant={wrap ? "default" : "outline"}
                onClick={() => setWrap((current) => !current)}
              >
                <FileText />
              </Button>
              <Button
                aria-label="Copy"
                disabled={!displayText}
                size="icon-sm"
                title="Copy"
                type="button"
                variant="outline"
                onClick={() => void copyText()}
              >
                <Clipboard />
              </Button>
              <Button
                aria-label="Refresh"
                disabled={status.kind === "loading"}
                size="icon-sm"
                title="Refresh"
                type="button"
                variant="outline"
                onClick={() => setGeneration((current) => current + 1)}
              >
                <RefreshCw />
              </Button>
              {artifact?.downloadHref ? (
                <Button asChild size="icon-sm" title="Download" variant="outline">
                  <a aria-label="Download" download href={artifact.downloadHref}>
                    <Download />
                  </a>
                </Button>
              ) : null}
            </div>
          </div>
        </DialogHeader>
        <div className="grid min-h-0 gap-0">
          {status.kind === "error" ? <NotificationMessage>{status.message}</NotificationMessage> : null}
          <section
            ref={textRef}
            aria-label="Text File Contents"
            className="max-h-[min(74vh,--spacing(176))] min-h-120 overflow-auto bg-background font-mono text-xs leading-5 text-foreground"
            data-testid="text-artifact-code-pane"
            data-wrap={wrap ? "true" : "false"}
          >
            <div className={wrap ? "min-w-full" : "min-w-max"}>
              {occurrenceKeys(lines, (line) => line).map(({ key, value: line }, index) => (
                <TextLine index={index} key={key} language={language} line={line} wrap={wrap} />
              ))}
            </div>
          </section>
          <footer
            className="flex min-h-9 items-center justify-between gap-3 border-t px-4 py-2 text-xs text-muted-foreground"
            data-testid="text-artifact-status"
          >
            <span className="min-w-0 truncate" data-state={status.kind}>
              {status.kind === "error" ? describeStatus(status) : `${describeStatus(status)} · ${language.toUpperCase()}`}
            </span>
            <span>{copyStatus === "copied" ? "Copied" : copyStatus === "failed" ? "Copy Failed" : truncated ? "Truncated" : ""}</span>
          </footer>
        </div>
      </DialogContent>
    </Dialog>
  );
}

function TextLine({ index, language, line, wrap }: { index: number; language: TextArtifactLanguage; line: string; wrap: boolean }) {
  return (
    <div
      className={
        wrap ? "grid grid-cols-(--layout-record-label) hover:bg-muted/25" : "grid grid-cols-(--layout-record-value) hover:bg-muted/25"
      }
    >
      <span
        aria-hidden="true"
        className="sticky left-0 z-10 select-none border-r bg-muted/45 px-3 text-right tabular-nums text-muted-foreground"
        data-testid="text-artifact-line-number"
      >
        {index + 1}
      </span>
      <code className={wrap ? "block min-h-5 whitespace-pre-wrap break-words px-3" : "block min-h-5 whitespace-pre px-3"}>
        {occurrenceKeys(highlightTextLine(line, language), tokenSignature).map(({ key, value: token }) => (
          <span className={tokenClassName(token.kind)} key={key}>
            {token.text}
          </span>
        ))}
      </code>
    </div>
  );
}

function tokenSignature(token: TextHighlightToken): string {
  return `${token.kind}:${token.text}`;
}

function tokenClassName(kind: TextHighlightKind): string {
  switch (kind) {
    case "boolean":
    case "null":
      return "text-violet-700 dark:text-violet-300";
    case "key":
      return "text-sky-700 dark:text-sky-300";
    case "level":
      return "font-semibold text-amber-700 dark:text-amber-300";
    case "number":
      return "text-emerald-700 dark:text-emerald-300";
    case "punctuation":
      return "text-muted-foreground";
    case "string":
      return "text-rose-700 dark:text-rose-300";
    case "timestamp":
      return "text-cyan-700 dark:text-cyan-300";
    default:
      return "";
  }
}
