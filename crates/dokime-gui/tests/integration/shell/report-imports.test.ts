/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

// Standard Library Imports
import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import { dirname, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

// Third Party Imports
import { test } from "vitest";

// Local Imports

/* === Re-Exports === */

/* === Constants === */

const root = fileURLToPath(new URL("../../../", import.meta.url));
const config = JSON.parse(readFileSync(resolve(root, "tsconfig.json"), "utf8").replace(/^\s*\/\/.*$/gm, "")) as {
  compilerOptions: { paths: Record<string, string[]> };
};

const DESKTOP_MODULES = [
  /^src\/(?:app|routes|stores|lib\/desktop)\//,
  /^src\/lib\/(?:api\/native-document-session|host\/(?:client|suite-session)|rpc\/(?:transport|ipc|artifact-upload))\./,
  /^src\/shell\/(?:AppShell\.|DesktopShell\.|ConsoleDrawer\.|MenuBar\.|Sidebar\.|StatusBar\.|console-(?:buffer|records)\.|focus-mode\.|page-chrome\.|route-scope\.|use-sidebar-state\.)/,
  /(?:^|\/)(?:server-state|[^/]+-queries|live-data|live-document|document-query)\.(?:svelte\.)?[jt]sx?$/,
  /^\.\.\/\.\.\/target\/generated\/host\/client\./,
];
const DESKTOP_DEPENDENCIES =
  /^(?:@tauri-apps\/|@tanstack\/(?:[^/]+-(?:router|query)|query-core|router-core)(?:\/|$)|@connectrpc\/|@grpc\/|(?:react(?:-dom)?|radix-ui|next-themes|sonner|zustand)(?:\/|$))/;

/* === Types === */

/* === Classes === */

/* === Tests === */

test("The Embedded Report Can Render Without Desktop Services or Routing", () => {
  const html = readFileSync(resolve(root, "report.html"), "utf8");
  const entries = [...html.matchAll(/<script\b[^>]*\bsrc="([^"]+)"/g)].map((match) => resolve(root, match[1].replace(/^\//, "")));
  assert.ok(entries.length, "the report must have a module entry");

  const messages = resolveLocal("@generated/host/model", entries[0]);
  assert.ok(messages, "resolve the shared generated message barrel");
  entries.push(messages);

  const visited = new Set<string>();
  const pending = entries.map((entry) => [entry]);
  while (pending.length) {
    const chain = pending.pop();
    assert.ok(chain);
    const file = chain[chain.length - 1];
    const path = relative(root, file).replaceAll("\\", "/");
    const trace = chain.map((item) => relative(root, item)).join(" → ");
    for (const desktopModule of DESKTOP_MODULES) {
      assert.doesNotMatch(path, desktopModule, trace);
    }
    if (visited.has(file)) continue;
    visited.add(file);
    if (!/\.(?:[cm]?[jt]sx?|svelte)$/.test(file)) continue;
    const source = readFileSync(file, "utf8");
    for (const specifier of runtimeImports(source)) {
      assert.doesNotMatch(specifier, DESKTOP_DEPENDENCIES, `${trace} → ${specifier}`);
      const dependency = resolveLocal(specifier, file);
      if (dependency) pending.push([...chain, dependency]);
    }
  }
});

/* === Public Functions === */

/* === Private Functions === */

function runtimeImports(source: string): string[] {
  const imports: string[] = [];
  const declarations = /\b(?:import|export)\s+(?!type\b)(?:([^;"']*?)\s+from\s*)?["']([^"']+)["']/g;
  for (const match of source.matchAll(declarations)) {
    const clause = match[1]?.trim();
    if (clause?.startsWith("{") && clause.endsWith("}")) {
      const members = clause
        .slice(1, -1)
        .split(",")
        .filter((member) => member.trim());
      if (members.length && members.every((member) => /^\s*type\s+/.test(member))) continue;
    }
    imports.push(match[2]);
  }
  for (const match of source.matchAll(/\b(?:import|require)\s*\(\s*["']([^"']+)["']\s*\)/g)) imports.push(match[1]);
  return imports;
}

function resolveLocal(specifier: string, importer: string): string | null {
  let base: string | undefined;
  for (const [alias, paths] of Object.entries(config.compilerOptions.paths)) {
    if (!alias.endsWith("/*") || !specifier.startsWith(alias.slice(0, -1))) continue;
    base = resolve(root, paths[0].replace("*", specifier.slice(alias.length - 1)));
    break;
  }
  if (!base && specifier.startsWith(".")) base = resolve(dirname(importer), specifier);
  if (!base) return null;
  // Assets Have No TypeScript Imports And May Be Produced By A Separate Build.
  if (/\.(?:css|json|svg|png|woff2?)$/.test(base)) return null;
  const typescript = base.replace(/\.[cm]?js$/, ".ts");
  const candidates = [
    typescript,
    base,
    ...[".ts", ".tsx", ".js", ".jsx", ".svelte", "/index.ts", "/index.tsx"].map((suffix) => base + suffix),
  ];
  const found = candidates.find((candidate) => existsSync(candidate) && /\.[^/]+$/.test(candidate));
  assert.ok(found, `unresolved report import ${specifier} from ${relative(root, importer)}`);
  return found;
}

/* === Entry Point === */
