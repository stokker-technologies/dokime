<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script module lang="ts">
  import type { ArtifactTreeFile, ArtifactTreeInput } from "../lib/artifact-tree";

  export type ArtifactActions = {
    open: (artifact: ArtifactTreeFile) => Promise<unknown>;
    download: (artifact: ArtifactTreeFile) => Promise<unknown>;
  };

  export type ArtifactTreeProps = {
    artifacts: ArtifactTreeInput[];
    emptyMessage: string;
    runDir?: string | null;
  };
</script>

<script lang="ts">
  import { Boxes, ChevronDown, ChevronRight, Download, FileAudio, FileText, FileVideo, Folder, FolderOpen, Image } from "lucide-svelte";
  import { SvelteSet } from "svelte/reactivity";
  import { type ArtifactTreeFolder, artifactDisplayPath, buildArtifactTree, formatArtifactSize, isLocalFileHref } from "../lib/artifact-tree";
  import { artifactLabel } from "../lib/dashboard";
  import { isPreviewableTextArtifact } from "../lib/execution-file-text";
  import { isMediaArtifact } from "../lib/media";
  import { failureOf } from "../lib/notification-details";
  import { toast } from "../lib/notifications";
  import { cn } from "../lib/utils";
  import Button from "./ui/Button.svelte";

  type Props = ArtifactTreeProps & {
    actions?: ArtifactActions;
    onPreview?: (artifact: ArtifactTreeFile) => void;
  };

  let { artifacts, emptyMessage, runDir, actions, onPreview }: Props = $props();
  let pending = $state(false);
  const expandedFolders = new SvelteSet<string>();
  let openError = $state<{ artifactId: string; message: string } | null>(null);
  let downloadError = $state<{ artifactId: string; message: string } | null>(null);
  const tree = $derived.by(() => {
    const visible = new Map<string, ArtifactTreeInput>();
    for (const artifact of artifacts) {
      if (isMediaArtifact(artifact)) {
        continue;
      }
      const path = artifactDisplayPath(artifact, runDir).toLowerCase();
      const label = artifactLabel(artifact.label).toLowerCase();
      visible.set(`${path}::${label}`, artifact);
    }
    return buildArtifactTree([...visible.values()], runDir);
  });
  const count = $derived(folderFileCount(tree));
  const rowClass = "grid min-h-7 min-w-0 grid-cols-[auto_minmax(0,1fr)_auto] items-center gap-2 rounded-lg px-2 py-1";
  const downloadClass = "grid size-8 place-items-center rounded-lg text-muted-foreground hover:bg-muted/45 hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring/40";

  function folderFileCount(folder: ArtifactTreeFolder): number {
    return folder.files.length + folder.folders.reduce((total, child) => total + folderFileCount(child), 0);
  }

  function toggleFolder(path: string) {
    if (!expandedFolders.has(path)) {
      expandedFolders.add(path);
      return;
    }
    for (const expanded of expandedFolders) {
      if (expanded === path || expanded.startsWith(`${path}/`)) {
        expandedFolders.delete(expanded);
      }
    }
  }

  function fileIcon(mediaType?: string | null) {
    const normalized = mediaType?.toLowerCase() ?? "";
    if (normalized.startsWith("image/")) {
      return Image;
    }
    if (normalized.startsWith("video/")) {
      return FileVideo;
    }
    if (normalized.startsWith("audio/")) {
      return FileAudio;
    }
    return normalized.includes("html") || normalized.includes("text") ? FileText : Boxes;
  }

  async function transfer(action: keyof ArtifactActions, artifact: ArtifactTreeFile) {
    if (!actions || pending) {
      return;
    }
    pending = true;
    if (action === "open") {
      openError = null;
    } else {
      downloadError = null;
    }
    try {
      await actions[action](artifact);
    } catch (error: unknown) {
      const title = action === "open" ? "Artifact Could Not Be Opened" : "Artifact Could Not Be Saved";
      const failure = failureOf(error, title);
      toast.error(failure);
      const rowError = { artifactId: artifact.artifact_id, message: failure.message };
      if (action === "open") {
        openError = rowError;
      } else {
        downloadError = rowError;
      }
    } finally {
      pending = false;
    }
  }
</script>

{#snippet entries(folder: ArtifactTreeFolder, level: number)}
  {#each folder.folders as child (child.path)}
    {@render folderRow(child, level)}
  {/each}
  {#each folder.files as artifact (artifact.artifact_id)}
    {@render fileRow(artifact, level)}
  {/each}
{/snippet}

{#snippet folderRow(folder: ArtifactTreeFolder, level: number)}
  <div>
    <Button
      variant="plain"
      size="inline"
      aria-expanded={expandedFolders.has(folder.path)}
      class="grid min-h-7 w-full grid-cols-[auto_auto_minmax(0,1fr)_auto] items-center gap-2 px-2 py-1 text-left hover:bg-muted/45"
      style={`padding-left: ${level * 16 + 8}px`}
      onclick={() => toggleFolder(folder.path)}
    >
      {#if expandedFolders.has(folder.path)}
        <ChevronDown class="text-muted-foreground" size={14} />
        <FolderOpen class="text-muted-foreground" size={15} />
      {:else}
        <ChevronRight class="text-muted-foreground" size={14} />
        <Folder class="text-muted-foreground" size={15} />
      {/if}
      <span class="min-w-0 truncate font-medium">{folder.name}</span>
      <span class="text-xs text-muted-foreground">{folderFileCount(folder)}</span>
    </Button>
    {#if expandedFolders.has(folder.path)}
      {@render entries(folder, level + 1)}
    {/if}
  </div>
{/snippet}

{#snippet fileRow(artifact: ArtifactTreeFile, level: number)}
  {@const label = artifactLabel(artifact.label)}
  {@const accessibleLabel = [artifact.fileName, label].filter(Boolean).join(" ")}
  {@const state = artifact.stream_state === "streaming" ? "Live" : artifact.stream_state === "finalized" ? "Finished" : artifact.stream_state === "interrupted" ? "Interrupted" : null}
  {@const details = [state, formatArtifactSize(artifact.size_bytes)].filter(Boolean).join(" · ")}
  {@const href = artifact.href ?? artifact.downloadHref ?? null}
  {@const downloadHref = artifact.downloadHref !== href ? artifact.downloadHref : null}
  {@const canTransfer = !href && (artifact.group_id !== "execution-files" || Boolean(artifact.scope?.executionId?.trim() && artifact.path.trim()))}
  {@const title = [artifact.fileName, label, artifact.description, artifact.displayPath, details].filter(Boolean).join("\n")}
  {@const fileOpenError = openError?.artifactId === artifact.artifact_id ? openError.message : null}
  {@const fileDownloadError = downloadError?.artifactId === artifact.artifact_id ? downloadError.message : null}
  {@const Icon = fileIcon(artifact.media_type)}
  <div class="grid min-h-7 grid-cols-[minmax(0,1fr)_auto] items-center gap-1" style:padding-left={`${level * 16 + 28}px`}>
    {#snippet fileLabel()}
      <Icon class="shrink-0" size={16} />
      <span class="min-w-0 truncate">{artifact.fileName}</span>
      <span class="text-xs tabular-nums text-muted-foreground">{details}</span>
    {/snippet}
    {#if actions && onPreview && isPreviewableTextArtifact(artifact)}
      <Button variant="plain" size="inline" aria-label={accessibleLabel} class={cn("artifact text-left hover:bg-muted/45", rowClass)} {title} onclick={() => onPreview?.(artifact)}>
        {@render fileLabel()}
      </Button>
    {:else if href}
      <a aria-label={accessibleLabel} class={cn("artifact hover:bg-muted/45 focus-visible:ring-2 focus-visible:ring-ring/40", rowClass)} {href} rel="noreferrer" target="_blank" {title}>
        {@render fileLabel()}
      </a>
    {:else if actions && canTransfer}
      <Button variant="plain" size="inline" aria-label={accessibleLabel} class={cn("artifact text-left hover:bg-muted/45", rowClass)} disabled={pending} title={fileOpenError ?? title} onclick={() => transfer("open", artifact)}>
        {@render fileLabel()}
      </Button>
    {:else}
      <div class={rowClass} {title}>{@render fileLabel()}</div>
    {/if}

    {#if actions && canTransfer}
      <span class="flex items-center gap-1">
        <Button aria-label={fileDownloadError ? `Retry Download ${label}` : `Download ${label}`} class={downloadClass} disabled={pending} size="icon-sm" title={fileDownloadError ?? "Download Artifact"} variant="ghost" onclick={() => transfer("download", artifact)}>
          <Download size={14} />
        </Button>
      </span>
    {:else if downloadHref}
      <a class={downloadClass} download href={downloadHref} title={isLocalFileHref(href) ? "Download Artifact" : "Open Download Link"}>
        <Download size={14} />
      </a>
    {/if}
  </div>
{/snippet}

{#if count}
  <div class="artifact-tree overflow-hidden rounded-lg border bg-background text-sm" data-testid="artifact-tree">
    <header class="grid min-h-8 grid-cols-[minmax(0,1fr)_auto] items-center gap-2 border-b bg-muted/35 px-3 py-2 text-xs">
      <strong>Explorer</strong>
      <span class="text-muted-foreground">{count} {count === 1 ? "item" : "items"}</span>
    </header>
    <div class="py-1">{@render entries(tree, 0)}</div>
  </div>
{:else}
  <div class="grid min-h-24 place-content-center rounded-lg border border-dashed bg-muted/20 px-3 py-4 text-center text-sm text-muted-foreground">
    {emptyMessage}
  </div>
{/if}
