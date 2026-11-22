/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Local Imports === */

import { occurrenceKeys } from "@/lib/utils";

/* === Types === */

type MarkdownTextProps = {
  text: string;
  compact?: boolean;
  title?: string;
};

/* === Components === */

export function MarkdownText({ text, compact = false, title }: MarkdownTextProps) {
  if (compact) {
    const preview = text.trim().replace(/\s+/g, " ");
    return (
      <div className="min-w-0 truncate text-sm leading-6" title={title ?? text}>
        {preview ? inlineMarkdown(preview) : <span className="text-muted-foreground">Empty note.</span>}
      </div>
    );
  }

  const blocks = markdownBlocks(text);
  return (
    <div className="min-w-0 space-y-1 text-sm leading-6">
      {blocks.length ? (
        occurrenceKeys(blocks, blockSignature).map(({ key, value }) => renderBlock(value, key))
      ) : (
        <p className="text-muted-foreground">Empty note.</p>
      )}
    </div>
  );
}

/* === Private Functions === */

type MarkdownBlock = { kind: "paragraph"; text: string } | { kind: "list"; items: string[] };
function markdownBlocks(text: string): MarkdownBlock[] {
  const blocks: MarkdownBlock[] = [];
  let listItems: string[] = [];
  text
    .trim()
    .split(/\r?\n/)
    .forEach((line) => {
      const trimmed = line.trim();
      if (!trimmed) {
        flushList(blocks, listItems);
        listItems = [];
        return;
      }
      if (trimmed.startsWith("- ") || trimmed.startsWith("* ")) {
        listItems.push(trimmed.slice(2));
        return;
      }
      flushList(blocks, listItems);
      listItems = [];
      blocks.push({ kind: "paragraph", text: trimmed });
    });
  flushList(blocks, listItems);
  return blocks;
}

function flushList(blocks: MarkdownBlock[], listItems: string[]) {
  if (listItems.length) {
    blocks.push({ kind: "list", items: [...listItems] });
  }
}

function renderBlock(block: MarkdownBlock, key: string) {
  if (block.kind === "list") {
    return (
      <ul className="list-disc space-y-1 pl-4" key={key}>
        {occurrenceKeys(block.items, (item) => item).map(({ key: itemKey, value: item }) => (
          <li key={itemKey}>{inlineMarkdown(item)}</li>
        ))}
      </ul>
    );
  }
  return <p key={key}>{inlineMarkdown(block.text)}</p>;
}

function inlineMarkdown(text: string) {
  const parts = text.split(/(`[^`]+`|\*\*[^*]+\*\*|\*[^*]+\*)/g);
  return occurrenceKeys(parts, (part) => part).map(({ key, value: part }) => {
    if (part.startsWith("`") && part.endsWith("`")) {
      return (
        <code className="rounded bg-muted px-1 py-1 text-xs" key={key}>
          {part.slice(1, -1)}
        </code>
      );
    }
    if (part.startsWith("**") && part.endsWith("**")) {
      return <strong key={key}>{part.slice(2, -2)}</strong>;
    }
    if (part.startsWith("*") && part.endsWith("*")) {
      return <em key={key}>{part.slice(1, -1)}</em>;
    }
    return part;
  });
}

function blockSignature(block: MarkdownBlock): string {
  return block.kind === "list" ? `list:${block.items.join("\u001f")}` : `paragraph:${block.text}`;
}
