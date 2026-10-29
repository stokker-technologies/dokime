/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import { artifactLabel } from "@/lib/dashboard";

export type ArtifactTreeInput = {
  artifact_id: string;
  label: string;
  path: string;
  href?: string | null;
  downloadHref?: string | null;
  media_type?: string | null;
  content_kind?: "text" | "binary" | "media" | "unknown" | null;
  stream_state?: "static" | "streaming" | "finalized" | "interrupted" | null;
  description?: string | null;
  size_bytes?: number | null;
  group_id?: string | null;
  group_label?: string | null;
  scope?: {
    test_cycle_id?: string | null;
    test_execution_id?: string | null;
    queue_item_id?: string | null;
    case_id?: string | null;
    template_id?: string | null;
  } | null;
};

export type ArtifactTreeFile = ArtifactTreeInput & {
  displayPath: string;
  fileName: string;
};

export type ArtifactTreeFolder = {
  kind: "folder";
  name: string;
  path: string;
  folders: ArtifactTreeFolder[];
  files: ArtifactTreeFile[];
};

export function buildArtifactTree(artifacts: ArtifactTreeInput[], runDir?: string | null): ArtifactTreeFolder {
  const root: ArtifactTreeFolder = {
    kind: "folder",
    name: "Artifacts",
    path: "",
    folders: [],
    files: [],
  };

  artifacts.forEach((artifact) => {
    const displayPath = artifactDisplayPath(artifact, runDir);
    const parts = displayPath.split("/").filter(Boolean);
    const fileName = parts.pop() ?? artifactLabel(artifact.label);
    let folder = root;

    parts.forEach((part) => {
      let child = folder.folders.find((candidate) => candidate.name === part);
      if (!child) {
        const childPath = folder.path ? `${folder.path}/${part}` : part;
        child = { kind: "folder", name: part, path: childPath, folders: [], files: [] };
        folder.folders.push(child);
      }
      folder = child;
    });

    folder.files.push({
      ...artifact,
      displayPath,
      fileName,
    });
  });

  sortArtifactFolder(root);
  return root;
}

export function artifactDisplayPath(artifact: Pick<ArtifactTreeInput, "label" | "path">, runDir?: string | null): string {
  const normalizedPath = normalizeArtifactPath(artifact.path);
  const normalizedRunDir = normalizeArtifactPath(runDir ?? "");
  const stripped = stripRunDir(normalizedPath, normalizedRunDir);
  return stripped || artifactLabel(artifact.label);
}

export function formatArtifactSize(sizeBytes?: number | null): string {
  if (sizeBytes == null || !Number.isFinite(sizeBytes) || sizeBytes < 0) {
    return "";
  }
  if (sizeBytes < 1024) {
    return `${sizeBytes} B`;
  }
  if (sizeBytes < 1024 * 1024) {
    return `${Math.ceil(sizeBytes / 1024)} KiB`;
  }
  return `${(sizeBytes / (1024 * 1024)).toFixed(1)} MiB`;
}

export function localFileHref(path: string | null | undefined): string | null {
  const normalized = normalizeArtifactPath(path ?? "");
  if (!normalized || !isAbsolutePath(normalized)) {
    return null;
  }
  const escaped = normalized.replace(/%/gu, "%25").replace(/#/gu, "%23").replace(/\?/gu, "%3F").replace(/ /gu, "%20");
  return normalized.startsWith("/") ? `file://${escaped}` : `file:///${escaped}`;
}

/** Resolve a relative artifact path against a native run directory. */
export function localArtifactHref(path: string | null | undefined, runDir?: string | null): string | null {
  const normalizedPath = normalizeArtifactPath(path ?? "");
  if (!normalizedPath) return null;
  if (isAbsolutePath(normalizedPath)) return localFileHref(normalizedPath);
  const normalizedRunDir = normalizeArtifactPath(runDir ?? "");
  return normalizedRunDir ? localFileHref(`${normalizedRunDir}/${normalizedPath}`) : null;
}

export function isLocalFileHref(href: string | null | undefined): boolean {
  return typeof href === "string" && href.startsWith("file://");
}

export function normalizeArtifactPath(path: string): string {
  return path
    .trim()
    .replaceAll("\\", "/")
    .replace(/^file:\/+/u, "")
    .replace(/\/+/gu, "/")
    .replace(/\/$/u, "");
}

function stripRunDir(path: string, runDir: string): string {
  if (!path) {
    return "";
  }
  if (!isAbsolutePath(path)) {
    return path;
  }
  if (!runDir) {
    return path.split("/").pop() ?? path;
  }

  const comparablePath = path.toLowerCase();
  const comparableRunDir = runDir.toLowerCase();
  if (comparablePath === comparableRunDir) {
    return "";
  }
  if (comparablePath.startsWith(`${comparableRunDir}/`)) {
    return path.slice(runDir.length + 1);
  }
  return path.split("/").pop() ?? path;
}

function isAbsolutePath(path: string): boolean {
  return path.startsWith("/") || /^[a-zA-Z]:\//u.test(path);
}

function sortArtifactFolder(folder: ArtifactTreeFolder): void {
  folder.folders.sort((left, right) => left.name.localeCompare(right.name));
  folder.files.sort((left, right) => left.fileName.localeCompare(right.fileName, undefined, { numeric: true }));
  folder.folders.forEach(sortArtifactFolder);
}
