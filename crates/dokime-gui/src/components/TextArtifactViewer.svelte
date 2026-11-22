<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { Dialog } from "bits-ui";
  import { Clipboard, Download, FileText, RefreshCw } from "lucide-svelte";
  import { untrack } from "svelte";
  import type { ConnectedSession } from "../app/native-session/session";
  import { failureOf } from "../lib/notification-details";
  import ErrorCard from "./ErrorCard.svelte";
  import Button from "./ui/Button.svelte";
  import DialogContent from "./ui/DialogContent.svelte";
  import Switch from "./ui/Switch.svelte";
  import { buttonVariants } from "./ui/button-variants";
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
    type TextHighlightKind,
    type TextHighlightToken,
  } from "@/lib/execution-file-text";
  import type { NativeStream } from "@/lib/rpc/transport";
  import type { ArtifactRecord, ExecutionFileFollowEvent } from "@/lib/types";
  import { occurrenceKeys } from "@/lib/utils";


  type TextViewerArtifact = Pick<
    ArtifactRecord,
    "artifact_id" | "downloadHref" | "href" | "label" | "path" | "scope" | "size_bytes" | "stream_state"
  >;

  type Props = {
    session: ConnectedSession | null;
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
    | { kind: "error"; message: string; failure?: unknown };


  function requireSession(session: ConnectedSession | null) {
    if (!session) {
      throw new Error("Connect to a backend before opening the file.");
    }
    return session;
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

  let { artifact, open, onOpenChange, session }: Props = $props();
  const executionId = $derived(artifact?.scope?.executionId ?? null);
  const artifactId = $derived(artifact?.artifact_id ?? null);
  const path = $derived(artifact?.path ?? null);
  const streamed = $derived(shouldFollowArtifact(artifact));
  const fileKey = $derived(`${executionId ?? ""}\u0000${artifactId ?? ""}\u0000${path ?? ""}`);
  let followChoice = $state<{ fileKey: string; follow: boolean } | null>(null);
  const follow = $derived(followChoice?.fileKey === fileKey ? followChoice.follow : streamed);
  let text = $state("");
  let truncated = $state(false);
  let status = $state<ViewerStatus>({ kind: "loading" });
  let generation = $state(0);
  let copyStatus = $state<"idle" | "copied" | "failed">("idle");
  let wrap = $state(true);
  let codePane: HTMLElement | undefined = $state();
  // Keep The Accepted Follow Position Across Follow Toggles.
  let acceptedCursor: ExecutionFileFollowCursor | null = null;
  let previousRead: { fileKey: string; generation: number; session: ConnectedSession | null } | undefined;

  $effect(() => {
    const key = fileKey;
    const revision = generation;
    const connection = session;
    const following = follow;
    const file = artifactId;
    const name = path;
    const execution = executionId;
    if (!open || !file || !name) {
      previousRead = undefined;
      return;
    }
    const last = previousRead;
    if (last?.fileKey !== key || last.generation !== revision || last.session !== connection) {
      acceptedCursor = null;
      text = "";
      truncated = false;
      status = { kind: "loading" };
      copyStatus = "idle";
    }
    previousRead = { fileKey: key, generation: revision, session: connection };
    return untrack(() => readFile(connection, file, name, execution, following));
  });

  function readFile(
    connection: ConnectedSession | null,
    artifactId: string,
    path: string,
    executionId: string | null,
    following: boolean,
  ) {
    const controller = new AbortController();
    const signal = controller.signal;
    let stream: NativeStream<ExecutionFileFollowEvent> | null = null;

    const readStatic = async () => {
      const connected = requireSession(connection);
      const content = await fetchNativeArtifactText({ ...connected.identity, own: connected.own }, artifactId, signal);
      if (signal.aborted) {
        return;
      }
      const bytes = new TextEncoder().encode(content).length;
      text = content.slice(0, TEXT_FILE_BUFFER_MAX_CHARS);
      truncated = content.length > TEXT_FILE_BUFFER_MAX_CHARS;
      status = { kind: "finished", bytes };
    };

    const readPages = async (execution: string) => {
      const connected = requireSession(connection);
      const page = await pageExecutionFile(
        (offset, maxBytes) => readNativeExecutionFileWindow(connected.identity, execution, path, offset, maxBytes),
        { budgetBytes: TEXT_FILE_PREVIEW_BUDGET_BYTES },
      );
      if (signal.aborted) {
        return;
      }
      text = new TextDecoder().decode(page.data).slice(0, TEXT_FILE_BUFFER_MAX_CHARS);
      truncated = page.startOffset > 0 || page.stop !== "complete";
      if (page.stop === "stalled") {
        status = {
          kind: "error",
          message: `Read stalled at ${formatArtifactSize(page.nextOffset)} of ${formatArtifactSize(page.observedSizeBytes)}. Refresh to retry.`,
        };
      } else if (page.state === "finalized" && page.stop === "complete") {
        status = { kind: "finished", bytes: page.nextOffset };
      } else if (page.state === "interrupted" && page.stop === "complete") {
        status = { kind: "abandoned", bytes: page.nextOffset };
      } else {
        status = { kind: "snapshot", bytes: page.nextOffset, observed: page.observedSizeBytes };
      }
    };

    const readFollowing = async (execution: string) => {
      const connected = requireSession(connection);
      const previous = acceptedCursor;
      let cursor = resumable(previous) ? connectingFollowCursor(previous.identity, previous.offset) : connectingFollowCursor();
      // A fresh follow may jump once to the tail; a resume continues its text.
      let tailJump = !resumable(previous);
      if (tailJump) {
        text = "";
        truncated = false;
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
          if (signal.aborted) {
            return;
          }
          cursor = advanceFollowCursor(cursor, event);
          if (event.kind === "metadata") {
            const tail = followTailOffset(event.availableLength, TEXT_FILE_FOLLOW_TAIL_BYTES);
            if (tailJump && tail > event.offset) {
              cursor = connectingFollowCursor(event.identity, tail);
              tailJump = false;
              jumped = true;
              truncated = true;
              break;
            }
            tailJump = false;
            status = cursor.phase === "producing" ? { kind: "live", bytes: cursor.offset } : { kind: "loading" };
          } else if (event.kind === "chunk") {
            const decoded = decoder.decode(event.data, { stream: true });
            text = appendBoundedText(text, decoded);
            if (cursor.phase === "producing") {
              status = { kind: "live", bytes: cursor.offset };
            }
          } else {
            const rest = decoder.decode();
            if (rest) {
              text = appendBoundedText(text, rest);
            }
            acceptedCursor = cursor;
            status = { kind: event.phase, bytes: cursor.offset };
            return;
          }
          acceptedCursor = cursor;
        }
        if (signal.aborted) {
          return;
        }
        if (!jumped) {
          throw new Error("The file stream ended before completion. Refresh to retry.");
        }
        await subscription.cancel();
      }
    };

    const run = async () => {
      if (!executionId) {
        await readStatic();
      } else if (following) {
        await readFollowing(executionId);
      } else if (acceptedCursor) {
        // Pausing keeps what the follow delivered; resuming appends from there.
        const cursor = acceptedCursor;
        status = {
          kind: cursor.phase === "producing" ? "paused" : cursor.phase === "abandoned" ? "abandoned" : "finished",
          bytes: cursor.offset,
        };
      } else {
        await readPages(executionId);
      }
    };

    void run().catch((caught: unknown) => {
      if (!signal.aborted) {
        const failure = failureOf(caught, "Artifact Unavailable");
        status = { kind: "error", message: failure.message, failure };
      }
    });
    return () => {
      controller.abort();
      void stream?.cancel().catch(() => undefined);
    };
  }

  const title = $derived(artifact ? artifactLabel(artifact.label) : "File");
  const description = $derived(artifact ? [artifact.path, formatArtifactSize(artifact.size_bytes)].filter(Boolean).join(" · ") : "");
  const language = $derived(detectTextArtifactLanguage(artifact));
  const displayText = $derived(formatTextArtifactContent(text, artifact));
  const lines = $derived(occurrenceKeys(displayText.split("\n"), (line) => line));
  const downloadClass = buttonVariants({ size: "icon-sm", variant: "outline" });

  $effect(() => {
    if (follow && codePane && displayText) {
      codePane.scrollTop = codePane.scrollHeight;
    }
  });

  async function copyText() {
    try {
      await navigator.clipboard.writeText(displayText);
      copyStatus = "copied";
    } catch {
      copyStatus = "failed";
    }
  }
</script>

<Dialog.Root {open} {onOpenChange}>
  <DialogContent class="max-h-[min(92vh,--spacing(232))] w-[min(96vw,--spacing(368))] max-w-none overflow-hidden p-0 sm:max-w-none">
    <div data-slot="dialog-header" class="flex flex-col gap-2 border-b px-4 py-3 pr-12">
      <div class="flex min-w-0 items-start justify-between gap-3">
        <div class="min-w-0">
          <Dialog.Title data-slot="dialog-title" class="truncate font-sans text-base leading-none font-medium">{title}</Dialog.Title>
          <Dialog.Description data-slot="dialog-description" class="truncate text-sm text-muted-foreground">{description}</Dialog.Description>
        </div>
        <div class="flex shrink-0 items-center gap-2">
          {#if streamed}
            <div class="flex items-center gap-2">
              <Switch
                checked={follow}
                id="text-artifact-follow"
                size="sm"
                onCheckedChange={(checked) => { followChoice = { fileKey, follow: checked }; }}
              />
              <label class="text-xs text-muted-foreground" for="text-artifact-follow">Follow</label>
            </div>
          {/if}
          <Button
            aria-label={wrap ? "Disable Wrapping" : "Enable Wrapping"}
            aria-pressed={wrap}
            size="icon-sm"
            title={wrap ? "Disable Wrapping" : "Enable Wrapping"}
            variant={wrap ? "default" : "outline"}
            onclick={() => { wrap = !wrap; }}
          ><FileText /></Button>
          <Button aria-label="Copy" disabled={!displayText} size="icon-sm" title="Copy" variant="outline" onclick={copyText}>
            <Clipboard />
          </Button>
          <Button
            aria-label="Refresh"
            disabled={status.kind === "loading"}
            size="icon-sm"
            title="Refresh"
            variant="outline"
            onclick={() => { generation += 1; }}
          ><RefreshCw /></Button>
          {#if artifact?.downloadHref}
            <a aria-label="Download" class={downloadClass} data-slot="button" data-size="icon-sm" data-variant="outline" download href={artifact.downloadHref} title="Download">
              <Download />
            </a>
          {/if}
        </div>
      </div>
    </div>
    <div class="grid min-h-0 gap-0">
      {#if status.kind === "error"}
        <ErrorCard failure={failureOf(status.failure ?? status.message, "Artifact Unavailable")} onRetry={() => { generation += 1; }} />
      {/if}
      <section
        bind:this={codePane}
        aria-label="Text File Contents"
        class="max-h-[min(74vh,--spacing(176))] min-h-120 overflow-auto bg-background font-mono text-xs leading-5 text-foreground"
        data-testid="text-artifact-code-pane"
        data-wrap={String(wrap)}
      >
        <div class={wrap ? "min-w-full" : "min-w-max"}>
          {#each lines as { key, value: line }, index (key)}
            <div class={wrap ? "grid grid-cols-(--layout-record-label) hover:bg-muted/25" : "grid grid-cols-(--layout-record-value) hover:bg-muted/25"}>
              <span aria-hidden="true" class="sticky left-0 z-10 select-none border-r bg-muted/45 px-3 text-right tabular-nums text-muted-foreground" data-testid="text-artifact-line-number">
                {index + 1}
              </span>
              <code class={wrap ? "block min-h-5 whitespace-pre-wrap break-words px-3" : "block min-h-5 whitespace-pre px-3"}>{#each occurrenceKeys(highlightTextLine(line, language), tokenSignature) as { key, value: token } (key)}<span class={tokenClassName(token.kind)}>{token.text}</span>{/each}</code>
            </div>
          {/each}
        </div>
      </section>
      <footer class="flex min-h-9 items-center justify-between gap-3 border-t px-4 py-2 text-xs text-muted-foreground" data-testid="text-artifact-status">
        <span class="min-w-0 truncate" data-state={status.kind}>
          {status.kind === "error" ? describeStatus(status) : `${describeStatus(status)} · ${language.toUpperCase()}`}
        </span>
        <span>{copyStatus === "copied" ? "Copied" : copyStatus === "failed" ? "Copy Failed" : truncated ? "Truncated" : ""}</span>
      </footer>
    </div>
  </DialogContent>
</Dialog.Root>
