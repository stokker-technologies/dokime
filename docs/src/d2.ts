// Dokime — SPDX-License-Identifier: Apache-2.0
// Render ::d2{src="..."} Directives to Dual Light/Dark Inline SVGs Offline.

import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import type { SatteriProcessorOptions } from "@astrojs/markdown-satteri";

type MdastPluginEntry = NonNullable<SatteriProcessorOptions["mdastPlugins"]>[number];

interface D2Options {
  title?: string;
  layout?: string;
  theme?: number;
  darkTheme?: number;
  sketch?: boolean;
  pad?: number;
}

const cache = new Map<string, [string, string]>();

/** Sätteri MDAST Plugin Expanding ::d2{src="..."} Leaf Directives Into Light/Dark SVGs. */
export function d2Plugin(): MdastPluginEntry {
  return ({ fileURL, source, sourceFormat }) => {
    if (!source.includes("::d2")) {
      return null;
    }
    return {
      name: "dokime-d2",
      leafDirective(node, ctx) {
        if (node.name !== "d2") {
          return;
        }
        const attrs = node.attributes ?? {};
        const src = attrs.src;
        if (!src) {
          throw new Error(`::d2 directive requires a src attribute (${fileURL?.pathname ?? "unknown"})`);
        }
        if (!fileURL) {
          throw new Error(`::d2 directive cannot resolve "${src}" without a document path`);
        }
        const documentPath = fileURLToPath(fileURL);
        const diagramPath = resolve(dirname(documentPath), src);
        const diagramSource = readFileSync(diagramPath, "utf8");
        const label = ctx.textContent(node).trim();
        const options: D2Options = {
          title: attrs.title || label || undefined,
          layout: attrs.layout || undefined,
          theme: attrs.theme ? Number(attrs.theme) : undefined,
          darkTheme: attrs["dark-theme"] ? Number(attrs["dark-theme"]) : undefined,
          pad: attrs.pad ? Number(attrs.pad) : undefined,
          sketch: attrs.sketch !== undefined && attrs.sketch !== "false",
        };
        const identity = `${documentPath}:${node.position?.start.line ?? 1}:${node.position?.start.column ?? 1}:${src}`;
        const html = renderD2(diagramSource, options, identity);
        if (sourceFormat === "mdx") {
          return {
            type: "mdxJsxFlowElement",
            name: "Fragment",
            attributes: [{ type: "mdxJsxAttribute", name: "set:html", value: html }],
            children: [],
          };
        }
        return { type: "html", value: html };
      },
    };
  };
}

function renderD2(source: string, options: D2Options, identity: string): string {
  const normalized = source.replaceAll("\r\n", "\n");
  const layout = options.layout ?? "dagre";
  const lightTheme = options.theme ?? 0;
  const darkTheme = options.darkTheme ?? 200;
  const sketch = options.sketch ?? false;
  const pad = options.pad;
  const key = JSON.stringify([normalized, layout, lightTheme, darkTheme, sketch, pad]);
  let pair = cache.get(key);
  if (!pair) {
    pair = [compile(normalized, layout, lightTheme, sketch, pad, "light"), compile(normalized, layout, darkTheme, sketch, pad, "dark")];
    cache.set(key, pair);
  }
  const label = escapeHtml(options.title ?? "Diagram");
  const prefix = createHash("sha256").update(identity).digest("hex").slice(0, 16);
  const svgs = pair.map((diagram, index) => {
    const scope = `diagram-${prefix}-${index}-`;
    const ids = new Set([...diagram.matchAll(/\bid="([^"]+)"/g)].map((match) => match[1]));
    const scoped = diagram
      .replace(/\bid="([^"]+)"/g, (_, id) => `id="${scope}${id}"`)
      .replace(/url\(#([^)]+)\)/g, (_, id) => `url(#${scope}${id})`)
      .replace(/\b(xlink:)?href="#([^"]+)"/g, (raw, ns, id) => (ids.has(id) ? `${ns ?? ""}href="#${scope}${id}"` : raw));
    return scoped.replace("<svg", `<svg role="img" aria-label="${label}" data-d2-theme="${index === 0 ? "light" : "dark"}"`);
  });
  return `<div class="d2-diagram">${svgs.join("")}</div>`;
}

function compile(source: string, layout: string, theme: number, sketch: boolean, pad: number | undefined, mode: "light" | "dark"): string {
  const env = Object.fromEntries(Object.entries(process.env).filter(([key]) => !key.startsWith("D2_") || key === "D2_TIMEOUT"));
  const args = [`--layout=${layout}`, `--theme=${theme}`, "--dark-theme=-1", `--salt=${mode}`, "--bundle=false", "--no-xml-tag"];
  if (sketch) args.push("--sketch");
  if (pad !== undefined) args.push(`--pad=${pad}`);
  args.push("-", "-");
  const result = spawnSync("d2", args, {
    input: source,
    encoding: "utf8",
    env,
    maxBuffer: 16 * 1024 * 1024,
    windowsHide: true,
  });
  if (result.error || result.status !== 0) {
    throw result.error ?? new Error(result.stderr || "d2 failed");
  }
  return result.stdout;
}

function escapeHtml(value: string): string {
  return value.replaceAll("&", "&amp;").replaceAll('"', "&quot;").replaceAll("<", "&lt;").replaceAll(">", "&gt;");
}
