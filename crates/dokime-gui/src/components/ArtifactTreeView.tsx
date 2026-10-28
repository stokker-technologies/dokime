/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import {
  Boxes,
  ChevronDown,
  ChevronRight,
  Download,
  FileAudio,
  FileText,
  FileVideo,
  Folder,
  FolderOpen,
  Image as ImageIcon,
} from "lucide-react";
import { type ReactNode, useMemo, useState } from "react";

import { Button } from "@/components/ui/button";
import {
  type ArtifactTreeFile,
  type ArtifactTreeFolder,
  type ArtifactTreeInput,
  artifactDisplayPath,
  buildArtifactTree,
  formatArtifactSize,
  isLocalFileHref,
} from "@/lib/artifact-tree";
import { artifactLabel } from "@/lib/dashboard";
import { isPreviewableTextArtifact } from "@/lib/execution-file-text";
import { isMediaArtifact } from "@/lib/media";

export type ArtifactActions = {
  open: (artifact: ArtifactTreeFile) => Promise<unknown>;
  download: (artifact: ArtifactTreeFile) => Promise<unknown>;
};

type TextPreview = (artifact: ArtifactTreeFile | null, onOpenChange: (open: boolean) => void) => ReactNode;

export type ArtifactTreeProps = {
  artifacts: ArtifactTreeInput[];
  emptyMessage: string;
  runDir?: string | null;
};

type ArtifactTreeViewProps = ArtifactTreeProps & {
  actions?: ArtifactActions;
  renderTextPreview?: TextPreview;
};

export function ArtifactTreeView({ artifacts, emptyMessage, runDir, actions, renderTextPreview }: ArtifactTreeViewProps) {
  const visibleArtifacts = useMemo(() => visibleArtifactInputs(artifacts, runDir), [artifacts, runDir]);
  const tree = useMemo(() => buildArtifactTree(visibleArtifacts, runDir), [visibleArtifacts, runDir]);

  if (!visibleArtifacts.length) {
    return <ArtifactEmptyState message={emptyMessage} />;
  }

  return <ArtifactFileBrowser actions={actions} tree={tree} renderTextPreview={renderTextPreview} />;
}

function ArtifactFileBrowser({
  actions,
  tree,
  renderTextPreview,
}: {
  actions?: ArtifactActions;
  tree: ArtifactTreeFolder;
  renderTextPreview?: TextPreview;
}) {
  const count = folderFileCount(tree);
  const [selectedTextArtifact, setSelectedTextArtifact] = useState<ArtifactTreeFile | null>(null);
  const [downloadingArtifactId, setDownloadingArtifactId] = useState<string | null>(null);
  const [downloadError, setDownloadError] = useState<{
    artifactId: string;
    message: string;
  } | null>(null);
  const [openingArtifactId, setOpeningArtifactId] = useState<string | null>(null);
  const [openError, setOpenError] = useState<{
    artifactId: string;
    message: string;
  } | null>(null);
  const openArtifact = async (artifact: ArtifactTreeFile) => {
    if (!actions || openingArtifactId || downloadingArtifactId) return;
    setOpeningArtifactId(artifact.artifact_id);
    setOpenError(null);
    try {
      await actions.open(artifact);
    } catch (error) {
      setOpenError({
        artifactId: artifact.artifact_id,
        message: error instanceof Error ? error.message : "Unable to open this artifact.",
      });
    } finally {
      setOpeningArtifactId(null);
    }
  };
  const downloadArtifact = async (artifact: ArtifactTreeFile) => {
    if (!actions || downloadingArtifactId || openingArtifactId) return;
    setDownloadingArtifactId(artifact.artifact_id);
    setDownloadError(null);
    try {
      await actions.download(artifact);
    } catch (error) {
      setDownloadError({
        artifactId: artifact.artifact_id,
        message: error instanceof Error ? error.message : "Unable to download this artifact.",
      });
    } finally {
      setDownloadingArtifactId(null);
    }
  };

  return (
    <>
      <div className="artifact-tree overflow-hidden rounded-lg border bg-background text-sm" data-testid="artifact-tree">
        <header className="grid min-h-8 grid-cols-[minmax(0,1fr)_auto] items-center gap-2 border-b bg-muted/35 px-3 py-2 text-xs">
          <strong>Explorer</strong>
          <span className="text-muted-foreground">
            {count} {count === 1 ? "item" : "items"}
          </span>
        </header>
        <div className="py-1">
          {tree.folders.map((folder) => (
            <ArtifactFolder
              folder={folder}
              key={folder.path}
              level={0}
              nativeDownloadEnabled={actions !== undefined}
              nativeDownloadBusy={downloadingArtifactId !== null}
              nativeDownloadError={downloadError}
              nativeOpenEnabled={actions !== undefined}
              nativePreviewEnabled={actions !== undefined && renderTextPreview !== undefined}
              nativeOpenBusy={openingArtifactId !== null}
              nativeOpenError={openError}
              onOpenTextArtifact={setSelectedTextArtifact}
              onOpenArtifact={openArtifact}
              onDownloadArtifact={downloadArtifact}
            />
          ))}
          {tree.files.map((artifact) => (
            <ArtifactFile
              artifact={artifact}
              key={artifact.artifact_id}
              level={0}
              nativeDownloadEnabled={actions !== undefined}
              nativeDownloadBusy={downloadingArtifactId !== null}
              nativeDownloadError={downloadError?.artifactId === artifact.artifact_id ? downloadError.message : null}
              nativeOpenEnabled={actions !== undefined}
              nativePreviewEnabled={actions !== undefined && renderTextPreview !== undefined}
              nativeOpenBusy={openingArtifactId !== null}
              nativeOpenError={openError?.artifactId === artifact.artifact_id ? openError.message : null}
              onOpenTextArtifact={setSelectedTextArtifact}
              onOpenArtifact={openArtifact}
              onDownloadArtifact={downloadArtifact}
            />
          ))}
        </div>
      </div>
      {renderTextPreview?.(selectedTextArtifact, (open) => {
        if (!open) setSelectedTextArtifact(null);
      })}
    </>
  );
}

function ArtifactEmptyState({ message }: { message: string }) {
  return (
    <div className="grid min-h-24 place-content-center rounded-lg border border-dashed bg-muted/20 px-3 py-4 text-center text-sm text-muted-foreground">
      {message}
    </div>
  );
}

function ArtifactFolder({
  folder,
  level,
  nativeDownloadEnabled,
  nativeDownloadBusy,
  nativeDownloadError,
  nativeOpenEnabled,
  nativeOpenBusy,
  nativeOpenError,
  nativePreviewEnabled,
  onOpenTextArtifact,
  onOpenArtifact,
  onDownloadArtifact,
}: {
  folder: ArtifactTreeFolder;
  level: number;
  nativeDownloadEnabled: boolean;
  nativeDownloadBusy: boolean;
  nativeDownloadError: { artifactId: string; message: string } | null;
  nativeOpenEnabled: boolean;
  nativeOpenBusy: boolean;
  nativeOpenError: { artifactId: string; message: string } | null;
  nativePreviewEnabled: boolean;
  onOpenTextArtifact: (artifact: ArtifactTreeFile) => void;
  onOpenArtifact: (artifact: ArtifactTreeFile) => void;
  onDownloadArtifact: (artifact: ArtifactTreeFile) => void;
}) {
  const [expanded, setExpanded] = useState(false);
  const count = folderFileCount(folder);

  return (
    <div>
      <button
        className="grid min-h-7 w-full grid-cols-[auto_auto_minmax(0,1fr)_auto] items-center gap-2 px-2 py-1 text-left transition-colors hover:bg-muted/45 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/40"
        style={{ paddingLeft: `${level * 16 + 8}px` }}
        type="button"
        onClick={() => setExpanded((current) => !current)}
      >
        {expanded ? (
          <ChevronDown className="text-muted-foreground" size={14} />
        ) : (
          <ChevronRight className="text-muted-foreground" size={14} />
        )}
        {expanded ? <FolderOpen className="text-muted-foreground" size={15} /> : <Folder className="text-muted-foreground" size={15} />}
        <span className="min-w-0 truncate font-medium">{folder.name}</span>
        <span className="text-xs text-muted-foreground">{count}</span>
      </button>
      {expanded ? (
        <div>
          {folder.folders.map((child) => (
            <ArtifactFolder
              folder={child}
              key={child.path}
              level={level + 1}
              nativeDownloadEnabled={nativeDownloadEnabled}
              nativeDownloadBusy={nativeDownloadBusy}
              nativeDownloadError={nativeDownloadError}
              nativeOpenEnabled={nativeOpenEnabled}
              nativeOpenBusy={nativeOpenBusy}
              nativeOpenError={nativeOpenError}
              nativePreviewEnabled={nativePreviewEnabled}
              onOpenTextArtifact={onOpenTextArtifact}
              onOpenArtifact={onOpenArtifact}
              onDownloadArtifact={onDownloadArtifact}
            />
          ))}
          {folder.files.map((artifact) => (
            <ArtifactFile
              artifact={artifact}
              key={artifact.artifact_id}
              level={level + 1}
              nativeDownloadEnabled={nativeDownloadEnabled}
              nativeDownloadBusy={nativeDownloadBusy && nativeDownloadError === null}
              nativeDownloadError={nativeDownloadError?.artifactId === artifact.artifact_id ? nativeDownloadError.message : null}
              nativeOpenEnabled={nativeOpenEnabled}
              nativeOpenBusy={nativeOpenBusy && nativeOpenError === null}
              nativeOpenError={nativeOpenError?.artifactId === artifact.artifact_id ? nativeOpenError.message : null}
              nativePreviewEnabled={nativePreviewEnabled}
              onOpenTextArtifact={onOpenTextArtifact}
              onOpenArtifact={onOpenArtifact}
              onDownloadArtifact={onDownloadArtifact}
            />
          ))}
        </div>
      ) : null}
    </div>
  );
}

function ArtifactFile({
  artifact,
  level,
  nativeDownloadEnabled,
  nativeDownloadBusy,
  nativeDownloadError,
  nativeOpenEnabled,
  nativeOpenBusy,
  nativeOpenError,
  nativePreviewEnabled,
  onOpenTextArtifact,
  onOpenArtifact,
  onDownloadArtifact,
}: {
  artifact: ArtifactTreeFile;
  level: number;
  nativeDownloadEnabled: boolean;
  nativeDownloadBusy: boolean;
  nativeDownloadError: string | null;
  nativeOpenEnabled: boolean;
  nativeOpenBusy: boolean;
  nativeOpenError: string | null;
  nativePreviewEnabled: boolean;
  onOpenTextArtifact: (artifact: ArtifactTreeFile) => void;
  onOpenArtifact: (artifact: ArtifactTreeFile) => void;
  onDownloadArtifact: (artifact: ArtifactTreeFile) => void;
}) {
  const fileLabel = artifactLabel(artifact.label);
  const size = formatArtifactSize(artifact.size_bytes);
  const primaryHref = artifact.href ?? artifact.downloadHref ?? null;
  const secondaryHref = artifact.downloadHref && artifact.downloadHref !== primaryHref ? artifact.downloadHref : null;
  const opensLocalFile = isLocalFileHref(primaryHref);
  /* The in-page preview reads the file through the backend. A frozen report
     has no backend and carries the bytes in the row's own href, so there the
     row stays a link rather than offering a viewer that cannot read. */
  const previewText = nativePreviewEnabled && isPreviewableTextArtifact(artifact);
  const nativeTransfer =
    !primaryHref && (artifact.group_id !== "execution-files" || Boolean(artifact.scope?.test_execution_id?.trim() && artifact.path.trim()));
  const title = [artifact.fileName, fileLabel, artifact.description, artifact.displayPath, size].filter(Boolean).join("\n");

  return (
    <div className="grid min-h-7 grid-cols-[minmax(0,1fr)_auto] items-center gap-1" style={{ paddingLeft: `${level * 16 + 28}px` }}>
      {previewText ? (
        <button
          aria-label={[artifact.fileName, fileLabel].filter(Boolean).join(" ")}
          className="artifact grid min-h-7 min-w-0 grid-cols-[auto_minmax(0,1fr)_auto] items-center gap-2 rounded-lg px-2 py-1 text-left transition-colors hover:bg-muted/45 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/40"
          title={title}
          type="button"
          onClick={() => onOpenTextArtifact(artifact)}
        >
          <ArtifactIcon mediaType={artifact.media_type} />
          <span className="min-w-0 truncate">{artifact.fileName}</span>
          <span className="text-xs tabular-nums text-muted-foreground">{size}</span>
        </button>
      ) : primaryHref ? (
        <a
          aria-label={[artifact.fileName, fileLabel].filter(Boolean).join(" ")}
          className="artifact grid min-h-7 min-w-0 grid-cols-[auto_minmax(0,1fr)_auto] items-center gap-2 rounded-lg px-2 py-1 transition-colors hover:bg-muted/45 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/40"
          href={primaryHref}
          rel="noreferrer"
          target="_blank"
          title={title}
        >
          <ArtifactIcon mediaType={artifact.media_type} />
          <span className="min-w-0 truncate">{artifact.fileName}</span>
          <span className="text-xs tabular-nums text-muted-foreground">{size}</span>
        </a>
      ) : nativeOpenEnabled && nativeTransfer && !previewText ? (
        <button
          aria-label={[artifact.fileName, fileLabel].filter(Boolean).join(" ")}
          className="artifact grid min-h-7 min-w-0 grid-cols-[auto_minmax(0,1fr)_auto] items-center gap-2 rounded-lg px-2 py-1 text-left transition-colors hover:bg-muted/45 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/40"
          disabled={nativeOpenBusy || nativeDownloadBusy}
          title={nativeOpenError ?? title}
          type="button"
          onClick={() => onOpenArtifact(artifact)}
        >
          <ArtifactIcon mediaType={artifact.media_type} />
          <span className="min-w-0 truncate">{artifact.fileName}</span>
          <span className="text-xs tabular-nums text-muted-foreground">{size}</span>
        </button>
      ) : (
        <div className="grid min-h-7 min-w-0 grid-cols-[auto_minmax(0,1fr)_auto] items-center gap-2 rounded-lg px-2 py-1" title={title}>
          <ArtifactIcon mediaType={artifact.media_type} />
          <span className="min-w-0 truncate">{artifact.fileName}</span>
          <span className="text-xs tabular-nums text-muted-foreground">{size}</span>
        </div>
      )}

      {nativeDownloadEnabled && nativeTransfer ? (
        <span className="flex items-center gap-1">
          <Button
            aria-label={nativeDownloadError ? `Retry download ${fileLabel}` : `Download ${fileLabel}`}
            className="grid size-8 place-items-center rounded-lg text-muted-foreground transition-colors hover:bg-muted/45 hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/40"
            disabled={nativeDownloadBusy || nativeOpenBusy}
            size="icon-sm"
            title={nativeDownloadError ?? "Download artifact"}
            type="button"
            variant="ghost"
            onClick={() => onDownloadArtifact(artifact)}
          >
            <Download size={14} />
          </Button>
        </span>
      ) : secondaryHref ? (
        <a
          className="grid size-8 place-items-center rounded-lg text-muted-foreground transition-colors hover:bg-muted/45 hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/40"
          download
          href={secondaryHref}
          title={opensLocalFile ? "Download artifact" : "Open download link"}
        >
          <Download size={14} />
        </a>
      ) : null}
    </div>
  );
}

function ArtifactIcon({ mediaType }: { mediaType?: string | null }) {
  const normalized = mediaType?.toLowerCase() ?? "";
  if (normalized.startsWith("image/")) {
    return <ImageIcon className="shrink-0" size={16} />;
  }
  if (normalized.startsWith("video/")) {
    return <FileVideo className="shrink-0" size={16} />;
  }
  if (normalized.startsWith("audio/")) {
    return <FileAudio className="shrink-0" size={16} />;
  }
  if (normalized.includes("html") || normalized.includes("text")) {
    return <FileText className="shrink-0" size={16} />;
  }
  return <Boxes className="shrink-0" size={16} />;
}

function visibleArtifactInputs(artifacts: ArtifactTreeInput[], runDir?: string | null): ArtifactTreeInput[] {
  const visible = artifacts.filter((artifact) => !isMediaArtifact(artifact));
  const deduped = new Map<string, ArtifactTreeInput>();
  visible.forEach((artifact) => {
    const displayPath = artifactDisplayPath(artifact, runDir).toLowerCase();
    const key = `${displayPath}::${artifactLabel(artifact.label).toLowerCase()}`;
    deduped.set(key, artifact);
  });
  return Array.from(deduped.values());
}

function folderFileCount(folder: ArtifactTreeFolder): number {
  return folder.files.length + folder.folders.reduce((total, child) => total + folderFileCount(child), 0);
}
