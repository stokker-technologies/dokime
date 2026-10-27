/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Local Imports === */

import type { ArtifactRecord } from "@/lib/types";

/* === Constants === */

const GUI_FILE_BUFFER_CHARS = 1_000_000;
const GUI_FILE_PREVIEW_BYTES = 262_144;
const GUI_FILE_TAIL_BYTES = 65_536;

/** Total bytes a non-following preview retains, read in contract-bounded pages. */
export const TEXT_FILE_PREVIEW_BUDGET_BYTES = GUI_FILE_PREVIEW_BYTES;
/** How far back from the live end a follow starts when the file is larger. */
export const TEXT_FILE_FOLLOW_TAIL_BYTES = GUI_FILE_TAIL_BYTES;
export const TEXT_FILE_BUFFER_MAX_CHARS = GUI_FILE_BUFFER_CHARS;

/* === Types === */

export type TextArtifactLanguage = "json" | "log" | "text";

export type TextHighlightKind = "boolean" | "key" | "level" | "null" | "number" | "plain" | "punctuation" | "string" | "timestamp";

export type TextHighlightToken = {
  kind: TextHighlightKind;
  text: string;
};

/* === Public Functions === */

/** Append decoded text, keeping only the newest `TEXT_FILE_BUFFER_MAX_CHARS`. */
export function appendBoundedText(current: string, addition: string): string {
  const text = `${current}${addition}`;
  return text.length > TEXT_FILE_BUFFER_MAX_CHARS ? text.slice(text.length - TEXT_FILE_BUFFER_MAX_CHARS) : text;
}

export function shouldFollowArtifact(artifact: Pick<ArtifactRecord, "stream_state"> | null | undefined): boolean {
  return artifact?.stream_state === "streaming";
}

export function isPreviewableTextArtifact(artifact: Pick<ArtifactRecord, "content_kind" | "media_type" | "path">): boolean {
  if (artifact.content_kind === "text") {
    return true;
  }
  const mediaType = artifact.media_type?.toLowerCase() ?? "";
  if (mediaType === "text/html") {
    return false;
  }
  if (
    mediaType.startsWith("text/") ||
    mediaType === "application/json" ||
    mediaType === "application/x-ndjson" ||
    mediaType === "application/xml"
  ) {
    return true;
  }
  return /\.(log|txt|jsonl?|csv|md|toml|ya?ml|xml)$/iu.test(artifact.path);
}

export function detectTextArtifactLanguage(artifact: Pick<ArtifactRecord, "media_type" | "path"> | null | undefined): TextArtifactLanguage {
  const mediaType = artifact?.media_type?.toLowerCase() ?? "";
  const path = artifact?.path.toLowerCase() ?? "";
  if (mediaType === "application/json" || /\.json$/u.test(path)) {
    return "json";
  }
  if (mediaType === "application/x-ndjson" || /\.(jsonl|log)$/u.test(path)) {
    return "log";
  }
  return "text";
}

export function formatTextArtifactContent(text: string, artifact: Pick<ArtifactRecord, "media_type" | "path"> | null | undefined): string {
  const normalized = text.replace(/\r\n?/gu, "\n");
  if (detectTextArtifactLanguage(artifact) !== "json") {
    return normalized;
  }
  try {
    return `${JSON.stringify(JSON.parse(normalized), null, 2)}\n`;
  } catch {
    return normalized;
  }
}

export function highlightTextLine(line: string, language: TextArtifactLanguage): TextHighlightToken[] {
  if (language === "json") {
    return highlightJsonLine(line);
  }
  if (language === "log") {
    return highlightLogLine(line);
  }
  return [{ kind: "plain", text: line }];
}

/* === Private Functions === */

function highlightJsonLine(line: string): TextHighlightToken[] {
  const tokens: TextHighlightToken[] = [];
  const regex = /"(?:\\.|[^"\\])*"(?=\s*:)|"(?:\\.|[^"\\])*"|-?\d+(?:\.\d+)?(?:e[+-]?\d+)?|\btrue\b|\bfalse\b|\bnull\b|[{}[\],:]/giu;
  let cursor = 0;
  for (const match of line.matchAll(regex)) {
    const index = match.index ?? 0;
    pushPlain(tokens, line.slice(cursor, index));
    const text = match[0] ?? "";
    tokens.push({
      kind: jsonTokenKind(text, line.slice(index + text.length)),
      text,
    });
    cursor = index + text.length;
  }
  pushPlain(tokens, line.slice(cursor));
  return tokens.length ? tokens : [{ kind: "plain", text: line }];
}

function highlightLogLine(line: string): TextHighlightToken[] {
  const tokens: TextHighlightToken[] = [];
  const regex =
    /\b(?:TRACE|DEBUG|INFO|WARN|WARNING|ERROR|FAIL|FAILED|PASS|PASSED)\b|\b\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?Z?\b|[A-Za-z_][\w.-]*=(?:"[^"]*"|'[^']*'|[^\s]*)/gu;
  let cursor = 0;
  for (const match of line.matchAll(regex)) {
    const index = match.index ?? 0;
    pushPlain(tokens, line.slice(cursor, index));
    const text = match[0] ?? "";
    if (text.includes("=")) {
      const equals = text.indexOf("=");
      tokens.push({ kind: "key", text: text.slice(0, equals) });
      tokens.push({ kind: "punctuation", text: "=" });
      tokens.push({ kind: valueTokenKind(text.slice(equals + 1)), text: text.slice(equals + 1) });
    } else if (/^\d{4}-/u.test(text)) {
      tokens.push({ kind: "timestamp", text });
    } else {
      tokens.push({ kind: "level", text });
    }
    cursor = index + text.length;
  }
  pushPlain(tokens, line.slice(cursor));
  return tokens.length ? tokens : [{ kind: "plain", text: line }];
}

function pushPlain(tokens: TextHighlightToken[], text: string): void {
  if (text) {
    tokens.push({ kind: "plain", text });
  }
}

function jsonTokenKind(text: string, after: string): TextHighlightKind {
  if (text.startsWith('"')) {
    if (!text.endsWith('"')) {
      return "plain";
    }
    return after.trimStart().startsWith(":") ? "key" : "string";
  }
  if (/^-?\d/u.test(text)) {
    return "number";
  }
  if (text === "true" || text === "false") {
    return "boolean";
  }
  if (text === "null") {
    return "null";
  }
  return "punctuation";
}

function valueTokenKind(text: string): TextHighlightKind {
  if (/^["']/u.test(text)) {
    return "string";
  }
  if (/^-?\d+(?:\.\d+)?$/u.test(text)) {
    return "number";
  }
  if (text === "true" || text === "false") {
    return "boolean";
  }
  if (text === "null") {
    return "null";
  }
  return "string";
}
