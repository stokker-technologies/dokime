<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { AudioLines, Download, ExternalLink, Image, Search, Video } from "lucide-svelte";
  import { formatArtifactSize } from "../lib/artifact-tree";
  import { artifactLabel } from "../lib/dashboard";
  import type { ResolvedMediaGalleryArtifact } from "./media-gallery-model";
  import { buttonVariants } from "./ui/button-variants";
  import Button from "./ui/Button.svelte";

  type Props = {
    artifact: ResolvedMediaGalleryArtifact;
    title: string;
    nativeBusy: boolean;
    nativeOpenError: string | null;
    nativeDownloadError: string | null;
    onInspect: () => void;
    onNativeOpen: () => void;
    onNativeDownload: () => void;
    onRetry: () => void;
    onVisibilityChange?: (artifactId: string, visible: boolean) => void;
    previewError: string | null;
  };

  let {
    artifact,
    title,
    nativeBusy,
    nativeOpenError,
    nativeDownloadError,
    onInspect,
    onNativeOpen,
    onNativeDownload,
    onRetry,
    onVisibilityChange,
    previewError,
  }: Props = $props();
  let element = $state<HTMLElement>();
  let failedImageHref = $state<string | null>(null);
  const meta = $derived(artifact.group_label ? artifactLabel(artifact.group_label) : formatArtifactSize(artifact.size_bytes));
  const Icon = $derived(artifact.kind === "image" ? Image : artifact.kind === "audio" ? AudioLines : Video);
  const artifactId = $derived(artifact.artifact_id);
  const ghostLink = buttonVariants({ size: "icon-xs", variant: "ghost" });

  $effect(() => {
    const notify = onVisibilityChange;
    const id = artifactId;
    if (!element || !notify) {
      return;
    }
    if (typeof IntersectionObserver === "undefined") {
      notify(id, true);
      return () => notify(id, false);
    }
    const observer = new IntersectionObserver(([entry]) => notify(id, entry?.isIntersecting ?? false), { rootMargin: "160px" });
    observer.observe(element);
    return () => {
      observer.disconnect();
      notify(id, false);
    };
  });
</script>

<article class="grid min-w-0 overflow-hidden rounded-lg border bg-background" bind:this={element}>
  <div class="flex aspect-video min-w-0 flex-col overflow-hidden bg-muted/45">
    <div class="relative flex min-h-0 min-w-0 flex-1 items-center justify-center overflow-hidden">
      {#if artifact.kind === "image" && artifact.href && artifact.href !== failedImageHref}
        <Button variant="plain" size="inline" aria-label={title} class="absolute inset-0" onclick={onInspect}>
          <img
            alt={title}
            class="h-full w-full object-cover"
            decoding="async"
            loading="lazy"
            src={artifact.href}
            onerror={() => {
              failedImageHref = artifact.href;
            }}
          >
        </Button>
      {:else if artifact.href && artifact.kind === "audio"}
        <!-- svelte-ignore a11y_media_has_caption (Operator Recordings Have No Caption Sidecars) -->
        <audio aria-label={title} class="w-full min-w-0 px-2" controls preload="metadata" src={artifact.href}></audio>
      {:else if artifact.href && artifact.kind === "video"}
        <!-- svelte-ignore a11y_media_has_caption (Captured Videos Have No Caption Sidecars) -->
        <video aria-label={title} class="h-full w-full object-contain" controls preload="metadata" src={artifact.href}></video>
      {:else}
        <span class="flex min-w-0 items-center justify-center p-3">
          <Icon class="text-muted-foreground" size={18} />
        </span>
      {/if}
    </div>
    <Button
      variant="plain"
      size="inline"
      class="flex shrink-0 items-center justify-end gap-1 px-2 py-1 text-xs font-semibold"
      title={artifact.displayPath}
      onclick={onInspect}
    >
      <Search size={12} />
      Inspect
    </Button>
  </div>
  <div class="grid min-w-0 gap-1 px-2 py-2">
    <div class="flex min-w-0 items-start gap-2">
      <Icon class="mt-1 shrink-0" size={14} />
      <div class="min-w-0 flex-1">
        <h3 class="truncate text-xs font-semibold" {title}>{title}</h3>
        <p class="truncate text-xs text-muted-foreground" title={artifact.displayPath}>{artifact.displayPath}</p>
      </div>
    </div>
    <div class="flex min-w-0 items-center justify-between gap-2">
      {#if meta}
        <span class="truncate text-xs text-muted-foreground">{meta}</span>
      {:else}
        <span></span>
      {/if}
      {#if previewError}
        <Button class="h-auto px-1 py-0 text-xs" size="sm" variant="ghost" onclick={onRetry}>Retry</Button>
      {/if}
      <span class="flex shrink-0 items-center gap-1">
        {#if artifact.nativeTransfer}
          <Button
            aria-label={nativeOpenError ? `Retry Open ${title}` : `Open ${title}`}
            disabled={nativeBusy}
            size="icon-xs"
            title={nativeOpenError ?? "Open Artifact"}
            variant="ghost"
            onclick={onNativeOpen}
          >
            <ExternalLink size={13} />
          </Button>
          <Button
            aria-label={nativeDownloadError ? `Retry Download ${title}` : `Download ${title}`}
            disabled={nativeBusy}
            size="icon-xs"
            title={nativeDownloadError ?? "Download Artifact"}
            variant="ghost"
            onclick={onNativeDownload}
          >
            <Download size={13} />
          </Button>
        {:else}
          {#if artifact.href}
            <a
              class={ghostLink}
              data-slot="button"
              data-size="icon-xs"
              data-variant="ghost"
              aria-label={`Open ${title}`}
              href={artifact.href}
              ><ExternalLink size={13} /></a
            >
          {/if}
          {#if artifact.downloadHref}
            <a
              class={ghostLink}
              data-slot="button"
              data-size="icon-xs"
              data-variant="ghost"
              aria-label={`Download ${title}`}
              href={artifact.downloadHref}
              ><Download size={13} /></a
            >
          {/if}
        {/if}
      </span>
    </div>
  </div>
</article>
