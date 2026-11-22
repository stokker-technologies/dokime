<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script module lang="ts">
  import type { MediaGalleryArtifact, ResolvedMediaGalleryArtifact } from "./media-gallery-model";
  import { artifactLabel } from "../lib/dashboard";

  function mediaArtifactTitle(artifact: Pick<ResolvedMediaGalleryArtifact, "displayPath" | "label">): string {
    return artifact.displayPath.split("/").filter(Boolean).at(-1) ?? artifactLabel(artifact.label);
  }
</script>

<script lang="ts">
  import { Dialog } from "bits-ui";
  import { Download, ExternalLink } from "lucide-svelte";
  import { SvelteSet } from "svelte/reactivity";
  import { artifactDisplayPath, formatArtifactSize } from "../lib/artifact-tree";
  import { mediaArtifactKind } from "../lib/media";
  import { failureOf, type NotificationFailure } from "../lib/notification-details";
  import ErrorCard from "./ErrorCard.svelte";
  import MediaGalleryCard from "./MediaGalleryCard.svelte";
  import { isNativeArtifactSource, mediaBlobType, type MediaGalleryProps } from "./media-gallery-model";
  import { mediaPreviews, type MediaTransfers } from "./media-previews.svelte";
  import { buttonVariants } from "./ui/button-variants";
  import Button from "./ui/Button.svelte";
  import DialogContent from "./ui/DialogContent.svelte";

  type Props = MediaGalleryProps & { transfers?: MediaTransfers };

  let { artifacts, emptyMessage, runDir, transfers }: Props = $props();
  let selectedId = $state<string | null>(null);
  let openingId = $state<string | null>(null);
  let downloadingId = $state<string | null>(null);
  let openError = $state<{ artifactId: string; failure: NotificationFailure } | null>(null);
  let downloadError = $state<{ artifactId: string; failure: NotificationFailure } | null>(null);
  const visible = new SvelteSet<string>();
  const media = $derived(
    artifacts.flatMap((artifact) => {
      const kind = mediaArtifactKind(artifact);
      return kind ? [{ ...artifact, displayPath: artifactDisplayPath(artifact, runDir), kind }] : [];
    }),
  );
  const preview = mediaPreviews(
    () => media,
    () => new Set(selectedId ? [...visible, selectedId] : visible),
    () => transfers,
  );
  const resolved = $derived(
    media.map((artifact) => ({
      ...artifact,
      href: artifact.href ?? artifact.downloadHref ?? preview.get(artifact.artifact_id)?.url ?? null,
      downloadHref: artifact.downloadHref ?? artifact.href ?? null,
      nativeTransfer: Boolean(transfers && !artifact.href && !artifact.downloadHref && isNativeArtifactSource(artifact)),
    })),
  );
  const selected = $derived(resolved.find((artifact) => artifact.artifact_id === selectedId));
  const selectedError = $derived(selected ? preview.get(selected.artifact_id)?.failure : undefined);
  const outlineLink = buttonVariants({ size: "sm", variant: "outline" });

  function markVisible(artifactId: string, isVisible: boolean) {
    if (isVisible) {
      visible.add(artifactId);
    } else {
      visible.delete(artifactId);
    }
  }

  async function openArtifact(artifact: MediaGalleryArtifact) {
    if (!transfers || openingId || downloadingId) {
      return;
    }
    openingId = artifact.artifact_id;
    openError = null;
    try {
      await transfers.open(artifact);
    } catch (error: unknown) {
      openError = { artifactId: artifact.artifact_id, failure: failureOf(error, "Artifact Action Failed") };
    } finally {
      openingId = null;
    }
  }

  async function downloadArtifact(artifact: MediaGalleryArtifact) {
    if (!transfers || downloadingId) {
      return;
    }
    downloadingId = artifact.artifact_id;
    downloadError = null;
    try {
      await transfers.download(artifact);
    } catch (error: unknown) {
      downloadError = { artifactId: artifact.artifact_id, failure: failureOf(error, "Artifact Action Failed") };
    } finally {
      downloadingId = null;
    }
  }

  function closePreview(open: boolean) {
    if (!open) {
      selectedId = null;
    }
  }
</script>

{#if resolved.length}
  <div class="grid grid-cols-[repeat(auto-fill,minmax(min(100%,--spacing(36)),1fr))] gap-2" data-testid="media-gallery">
    {#each resolved as artifact (artifact.artifact_id)}
      <MediaGalleryCard
        {artifact}
        title={mediaArtifactTitle(artifact)}
        onInspect={() => {
          selectedId = artifact.artifact_id;
        }}
        onNativeOpen={() => openArtifact(artifact)}
        onNativeDownload={() => downloadArtifact(artifact)}
        nativeBusy={openingId === artifact.artifact_id || downloadingId === artifact.artifact_id}
        nativeOpenError={openError?.artifactId === artifact.artifact_id ? openError.failure.message : null}
        nativeDownloadError={downloadError?.artifactId === artifact.artifact_id ? downloadError.failure.message : null}
        onRetry={() => preview.retry(artifact.artifact_id)}
        previewError={preview.get(artifact.artifact_id)?.failure?.message ?? null}
        onVisibilityChange={transfers ? markVisible : undefined}
      />
    {/each}
  </div>
  <Dialog.Root open={selected !== undefined} onOpenChange={closePreview}>
    <DialogContent class="sm:max-w-5xl">
      {#if selected}
        {@const title = mediaArtifactTitle(selected)}
        {@const nativeBusy = openingId === selected.artifact_id || downloadingId === selected.artifact_id}
        <div class="grid gap-4">
          <div data-slot="dialog-header" class="flex flex-col gap-2 text-center sm:text-left">
            <Dialog.Title data-slot="dialog-title" class="truncate text-base leading-none font-medium" {title}>{title}</Dialog.Title>
            <Dialog.Description data-slot="dialog-description" class="truncate text-sm text-muted-foreground" title={selected.displayPath}>
              {selected.displayPath}
            </Dialog.Description>
          </div>
          <div class="overflow-hidden rounded-lg border bg-muted/20 p-2">
            {#if selected.href}
              {#if selected.kind === "image"}
                <img alt={title} class="max-h-[70vh] w-full object-contain" src={selected.href}>
              {:else if selected.kind === "audio"}
                <!-- svelte-ignore a11y_media_has_caption (Operator Recordings Have No Caption Sidecars) -->
                <audio class="w-full" controls preload="metadata" src={selected.href}></audio>
              {:else}
                <!-- svelte-ignore a11y_media_has_caption (Captured Videos Have No Caption Sidecars) -->
                <video autoplay class="max-h-[70vh] w-full" controls preload="metadata" src={selected.href}></video>
              {/if}
            {:else if selected.nativeTransfer && mediaBlobType(selected)}
              <div class="grid justify-items-center gap-3 p-8 text-center text-sm text-muted-foreground">
                {#if selectedError}
                  <ErrorCard
                    failure={selectedError}
                    onRetry={() => preview.retry(selected.artifact_id)}
                  />
                {:else}
                  <p>Loading media preview…</p>
                {/if}
              </div>
            {:else}
              <p class="p-8 text-center text-sm text-muted-foreground">Preview unavailable for this media artifact.</p>
            {/if}
          </div>
          <div class="flex flex-wrap items-center justify-between gap-3">
            <div class="text-sm text-muted-foreground">
              {selected.group_label ? artifactLabel(selected.group_label) : "Media Artifact"}
              {selected.size_bytes ? ` • ${formatArtifactSize(selected.size_bytes)}` : ""}
            </div>
            <div class="flex flex-wrap items-center gap-2">
              {#if selected.nativeTransfer}
                <Button disabled={nativeBusy} size="sm" variant="outline" onclick={() => openArtifact(selected)}>
                  <ExternalLink data-icon="inline-start" size={14} />
                  {openingId === selected.artifact_id ? "Opening…" : "Open"}
                </Button>
                <Button disabled={nativeBusy} size="sm" variant="outline" onclick={() => downloadArtifact(selected)}>
                  <Download data-icon="inline-start" size={14} />
                  {downloadingId === selected.artifact_id ? "Saving…" : "Download"}
                </Button>
              {:else}
                {#if selected.href}
                  <a
                    class={outlineLink}
                    data-slot="button"
                    data-size="sm"
                    data-variant="outline"
                    href={selected.href}
                    rel="noreferrer"
                    target="_blank"
                  >
                    <ExternalLink data-icon="inline-start" size={14} />
                    Open
                  </a>
                {/if}
                {#if selected.downloadHref}
                  <a class={outlineLink} data-slot="button" data-size="sm" data-variant="outline" href={selected.downloadHref}>
                    <Download data-icon="inline-start" size={14} />
                    Download
                  </a>
                {/if}
              {/if}
              {#if openError?.artifactId === selected.artifact_id}
                <ErrorCard notificationContext="action" failure={openError.failure} />
              {/if}
              {#if downloadError?.artifactId === selected.artifact_id}
                <ErrorCard notificationContext="action" failure={downloadError.failure} />
              {/if}
            </div>
          </div>
        </div>
      {/if}
    </DialogContent>
  </Dialog.Root>
{:else}
  <p class="text-sm text-muted-foreground">{emptyMessage}</p>
{/if}
