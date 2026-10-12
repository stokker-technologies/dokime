/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { useQueryClient } from "@tanstack/react-query";
import { Clipboard, Download, FileText, RefreshCw } from "lucide-react";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";

/* === Local Imports === */

import { useNativeSessionOptional } from "@/app/native-session/NativeSessionProvider";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { fetchExecutionFileTextQuery } from "@/features/cycle/server-state";
import { fetchNativeArtifactText } from "@/lib/api/native-content";
import { formatArtifactSize } from "@/lib/artifact-tree";
import { artifactLabel } from "@/lib/dashboard";
import {
  applyExecutionFileRead,
  detectTextArtifactLanguage,
  emptyTextFileBuffer,
  formatTextArtifactContent,
  highlightTextLine,
  shouldFollowArtifact,
  TEXT_FILE_BUFFER_MAX_CHARS,
  TEXT_FILE_FOLLOW_POLL_MS,
  TEXT_FILE_FOLLOW_TAIL_BYTES,
  TEXT_FILE_PREVIEW_MAX_BYTES,
  type TextArtifactLanguage,
  type TextFileBuffer,
  type TextHighlightKind,
  type TextHighlightToken,
} from "@/lib/execution-file-text";
import type { ArtifactRecord } from "@/lib/types";
import { occurrenceKeys } from "@/lib/utils";

/* === Types === */

type TextViewerArtifact = Pick<
  ArtifactRecord,
  | "artifact_id"
  | "downloadHref"
  | "href"
  | "label"
  | "path"
  | "scope"
  | "size_bytes"
  | "stream_state"
>;

type TextArtifactViewerProps = {
  artifact: TextViewerArtifact | null;
  open: boolean;
  onOpenChange: (open: boolean) => void;
};

/* === Private Functions === */

/** The connection a read needs. Unreachable while nothing can be opened. */
function requireSession(session: ReturnType<typeof useNativeSessionOptional>) {
  if (!session) throw new Error("A connected native backend is required.");
  return session;
}

/* === Components === */

export function TextArtifactViewer({ artifact, open, onOpenChange }: TextArtifactViewerProps) {
  /* The artifact explorer mounts this dialog beside the tree whether or not
     anything is open, and the frozen report mounts that tree. Asking for the
     connection here would make a file with one text artifact fail to render,
     so the connection is optional and only the reads insist on it — and those
     run when a reader opens a file, which the report offers no way to do. */
  const session = useNativeSessionOptional();
  const queryClient = useQueryClient();
  const executionId = artifact?.scope?.test_execution_id ?? null;
  const streamed = shouldFollowArtifact(artifact);
  const [follow, setFollow] = useState(false);
  const [buffer, setBuffer] = useState<TextFileBuffer>(emptyTextFileBuffer);
  const bufferRef = useRef<TextFileBuffer>(emptyTextFileBuffer());
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [copyStatus, setCopyStatus] = useState<"idle" | "copied" | "failed">("idle");
  const [wrap, setWrap] = useState(true);
  const textRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    bufferRef.current = buffer;
  }, [buffer]);

  useEffect(() => {
    setFollow(streamed);
    setBuffer(emptyTextFileBuffer());
    setError(null);
    setCopyStatus("idle");
  }, [streamed]);

  const readExecutionFile = useCallback(
    async (mode: "initial" | "append", signal?: AbortSignal) => {
      if (!artifact || !executionId) {
        return;
      }
      const read = await fetchExecutionFileTextQuery(queryClient, requireSession(session), {
        executionId,
        maxBytes: TEXT_FILE_PREVIEW_MAX_BYTES,
        offset:
          mode === "append"
            ? bufferRef.current.nextOffset
            : mode === "initial" && follow
              ? undefined
              : 0,
        path: artifact.path,
        signal,
        tailBytes: mode === "initial" && follow ? TEXT_FILE_FOLLOW_TAIL_BYTES : undefined,
      });
      setBuffer((current) => applyExecutionFileRead(current, read));
    },
    [artifact, executionId, follow, queryClient, session],
  );

  const readStaticArtifact = useCallback(
    async (signal?: AbortSignal) => {
      if (!artifact || executionId) {
        return;
      }
      const connected = requireSession(session);
      const text = await fetchNativeArtifactText(
        { ...connected.identity, own: connected.own },
        artifact.artifact_id,
        signal,
      );
      const byteLength = new TextEncoder().encode(text).length;
      const trimmedText =
        text.length > TEXT_FILE_BUFFER_MAX_CHARS ? text.slice(0, TEXT_FILE_BUFFER_MAX_CHARS) : text;
      setBuffer({
        text: trimmedText,
        startOffset: 0,
        nextOffset: byteLength,
        sizeBytes: byteLength,
        truncated: trimmedText.length < text.length,
        rotated: false,
      });
    },
    [artifact, executionId, session],
  );

  const refresh = useCallback(
    async (mode: "initial" | "append", signal?: AbortSignal) => {
      if (!artifact) {
        return;
      }
      setLoading(true);
      setError(null);
      try {
        if (executionId) {
          await readExecutionFile(mode, signal);
        } else {
          await readStaticArtifact(signal);
        }
      } catch (caught) {
        if (!signal?.aborted) {
          setError(caught instanceof Error ? caught.message : String(caught));
        }
      } finally {
        if (!signal?.aborted) {
          setLoading(false);
        }
      }
    },
    [artifact, executionId, readExecutionFile, readStaticArtifact],
  );

  useEffect(() => {
    if (!open || !artifact) {
      return;
    }
    const controller = new AbortController();
    setBuffer(emptyTextFileBuffer());
    void refresh("initial", controller.signal);
    return () => controller.abort();
  }, [artifact, open, refresh]);

  useEffect(() => {
    if (!open || !artifact || !executionId || !follow) {
      return;
    }
    const interval = window.setInterval(() => {
      void refresh("append");
    }, TEXT_FILE_FOLLOW_POLL_MS);
    return () => window.clearInterval(interval);
  }, [artifact, executionId, follow, open, refresh]);

  useEffect(() => {
    if (follow && textRef.current) {
      textRef.current.scrollTop = textRef.current.scrollHeight;
    }
  }, [follow]);

  const title = artifact ? artifactLabel(artifact.label) : "File";
  const description = useMemo(() => {
    if (!artifact) {
      return "";
    }
    const size = formatArtifactSize(buffer.sizeBytes || artifact.size_bytes);
    return [artifact.path, size].filter(Boolean).join(" · ");
  }, [artifact, buffer.sizeBytes]);
  const language = useMemo(() => detectTextArtifactLanguage(artifact), [artifact]);
  const displayText = useMemo(
    () => formatTextArtifactContent(buffer.text, artifact),
    [artifact, buffer.text],
  );
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
      <DialogContent className="max-h-[min(92vh,58rem)] w-[min(96vw,92rem)] max-w-none overflow-hidden p-0 sm:max-w-none">
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
                    onCheckedChange={setFollow}
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
                disabled={loading}
                size="icon-sm"
                title="Refresh"
                type="button"
                variant="outline"
                onClick={() => void refresh("initial")}
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
          {error ? (
            <div className="border-b bg-destructive/10 px-4 py-2 text-sm text-destructive">
              {error}
            </div>
          ) : null}
          <section
            ref={textRef}
            aria-label="Text File Contents"
            className="max-h-[min(74vh,44rem)] min-h-[30rem] overflow-auto bg-background font-mono text-[12px] leading-5 text-foreground"
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
            <span className="min-w-0 truncate">
              {loading
                ? "Loading"
                : `${formatArtifactSize(buffer.nextOffset)} read · ${language.toUpperCase()}`}
            </span>
            <span>
              {copyStatus === "copied"
                ? "Copied"
                : copyStatus === "failed"
                  ? "Copy Failed"
                  : buffer.truncated
                    ? "Truncated"
                    : buffer.rotated
                      ? "Rotated"
                      : ""}
            </span>
          </footer>
        </div>
      </DialogContent>
    </Dialog>
  );
}

function TextLine({
  index,
  language,
  line,
  wrap,
}: {
  index: number;
  language: TextArtifactLanguage;
  line: string;
  wrap: boolean;
}) {
  return (
    <div
      className={
        wrap
          ? "grid grid-cols-[4.5rem_minmax(0,1fr)] hover:bg-muted/25"
          : "grid grid-cols-[4.5rem_max-content] hover:bg-muted/25"
      }
    >
      <span
        aria-hidden="true"
        className="sticky left-0 z-10 select-none border-r bg-muted/45 px-3 text-right tabular-nums text-muted-foreground"
        data-testid="text-artifact-line-number"
      >
        {index + 1}
      </span>
      <code
        className={
          wrap
            ? "block min-h-5 whitespace-pre-wrap break-words px-3"
            : "block min-h-5 whitespace-pre px-3"
        }
      >
        {occurrenceKeys(highlightTextLine(line, language), tokenSignature).map(
          ({ key, value: token }) => (
            <span className={tokenClassName(token.kind)} key={key}>
              {token.text}
            </span>
          ),
        )}
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
