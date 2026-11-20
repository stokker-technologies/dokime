/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Render Fenced D2 Diagrams Offline
 */

/* === Imports === */

// Standard Library Imports
import child_process from "node:child_process";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";

// Third Party Imports
import type { Root } from "mdast";
import { visit } from "unist-util-visit";

// Local Imports

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Public Functions === */

/** Render Self-Contained D2 Fences Once per Build, Independently of Their Titles. */
export function remarkDiagrams() {
  const cache = new Map<string, string[]>();
  return () => (tree: Root, file: { path?: string; value?: unknown }) => {
    visit(tree, "code", (node, index, parent) => {
      if (node.lang !== "d2" || !parent || index === undefined) {
        return;
      }
      const location = `${file.path ?? "Markdown"}:${node.position?.start.line ?? 1}:${node.position?.start.column ?? 1}`;
      const title = node.meta?.match(/(?:^|\s)title=(?:"([^"]+)"|'([^']+)')/);
      const name = escapeHtml(title?.[1] ?? title?.[2] ?? "Diagram");
      const source = node.value.replaceAll("\r\n", "\n");
      let diagrams = cache.get(source);
      if (!diagrams) {
        try {
          diagrams = [render(source, "light", 0), render(source, "dark", 200)];
        } catch (error) {
          // Astro Removes Frontmatter and Leading Whitespace Before Parsing the Body.
          let line = node.position?.start.line ?? 1;
          if (file.path && typeof file.value === "string") {
            const original = readFileSync(file.path, "utf8").replaceAll("\r\n", "\n");
            const offset = original.lastIndexOf(file.value.replaceAll("\r\n", "\n"));
            if (offset >= 0) {
              line += original.slice(0, offset).split("\n").length - 1;
            }
          }
          const origin = `${file.path ?? "Markdown"}:${line}:${node.position?.start.column ?? 1}`;
          throw new Error(`Render D2 at ${origin}: Fix the diagram or run mise install.\n${String(error)}`);
        }
        cache.set(source, diagrams);
      }
      const prefix = createHash("sha256").update(location).digest("hex").slice(0, 16);
      const themed = diagrams.map((diagram, theme) => {
        const identifier = `diagram-${prefix}-${theme}-`;
        const identifiers = new Set([...diagram.matchAll(/\bid="([^"]+)"/g)].map((match) => match[1]));
        const svg = diagram
          .replace(/\bid="([^"]+)"/g, (_, id) => `id="${identifier}${id}"`)
          .replace(/url\(#([^)]+)\)/g, (_, id) => `url(#${identifier}${id})`)
          .replace(/\b(xlink:)?href="#([^"]+)"/g, (value, namespace, id) =>
            identifiers.has(id) ? `${namespace ?? ""}href="#${identifier}${id}"` : value,
          );
        return svg.replace("<svg", `<svg role="img" aria-label="${name}" data-d2-theme="${theme === 0 ? "light" : "dark"}"`);
      });
      parent.children[index] = { type: "html", value: `<div class="d2-diagram">${themed.join("")}</div>` };
    });
  };
}

/* === Private Functions === */

function render(source: string, theme: string, themeId: number): string {
  // Ignore Ambient Layout and Font Overrides; Keep the Operation's Timeout Policy.
  const env = Object.fromEntries(Object.entries(process.env).filter(([key]) => !key.startsWith("D2_") || key === "D2_TIMEOUT"));
  const result = child_process.spawnSync(
    "d2",
    ["--layout=dagre", `--theme=${themeId}`, "--dark-theme=-1", `--salt=${theme}`, "--bundle=false", "--no-xml-tag", "-", "-"],
    { input: source, encoding: "utf8", env, maxBuffer: 16 * 1024 * 1024, windowsHide: true },
  );
  if (result.error || result.status !== 0) {
    throw result.error ?? new Error(result.stderr);
  }
  return result.stdout;
}

function escapeHtml(value: string): string {
  return value.replaceAll("&", "&amp;").replaceAll('"', "&quot;").replaceAll("<", "&lt;").replaceAll(">", "&gt;");
}

/* === Entry Point === */
