/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import { dirname, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { test } from "vitest";

const root = fileURLToPath(new URL("../../../", import.meta.url));
const config = JSON.parse(readFileSync(resolve(root, "tsconfig.json"), "utf8").replace(/^\s*\/\/.*$/gm, "")) as {
  compilerOptions: { paths: Record<string, string[]> };
};

test("the embedded report cannot reach generated RPC or message modules", () => {
  const vite = readFileSync(resolve(root, "vite.report.config.ts"), "utf8");
  const input = vite.match(/input:\s*fileURLToPath\(new URL\("(.+?)", import.meta.url\)\)/)?.[1];
  assert.ok(input, "resolve the report HTML from the Vite entry configuration");
  const html = readFileSync(resolve(root, input), "utf8");
  const entries = [...html.matchAll(/<script\b[^>]*\bsrc="([^"]+)"/g)].map((match) => resolve(root, match[1].replace(/^\//, "")));
  assert.ok(entries.length, "the report must have a module entry");

  const visited = new Set<string>();
  const pending = entries.map((entry) => [entry]);
  while (pending.length) {
    const chain = pending.pop();
    assert.ok(chain);
    const file = chain[chain.length - 1];
    const path = relative(root, file).replaceAll("\\", "/");
    const trace = chain.map((item) => relative(root, item)).join(" → ");
    assert.doesNotMatch(path, /^(?:src\/lib\/rpc\/generated\/|src\/lib\/api\/native-documents\.|\.\.\/\.\.\/target\/generated\/)/, trace);
    if (visited.has(file)) continue;
    visited.add(file);
    if (!/\.[cm]?[jt]sx?$/.test(file)) continue;
    const source = readFileSync(file, "utf8");
    for (const specifier of runtimeImports(source)) {
      const dependency = resolveLocal(specifier, file);
      if (dependency) pending.push([...chain, dependency]);
    }
  }
});

test("the graph walker follows re-exports and lazy imports while ignoring erased types", () => {
  assert.deepEqual(
    runtimeImports(`
      import type { Message } from "types";
      import { type Other } from "other-types";
      export type { More } from "more-types";
      import { type A, value } from "mixed";
      import "side-effect";
      export { value } from "re-export";
      export * from "barrel";
      const live = () => import("lazy");
    `),
    ["mixed", "side-effect", "re-export", "barrel", "lazy"],
  );
});

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
  // Preserve The Forbidden Target Even When Generation Has Never Run.
  if (base.startsWith(resolve(root, "../../target/generated"))) return base;
  // Assets Have No TypeScript Imports And May Be Produced By A Separate Build.
  if (/\.(?:css|json|svg|png|woff2?)$/.test(base)) return null;
  const candidates = [base, ...[".ts", ".tsx", ".js", ".jsx", "/index.ts", "/index.tsx"].map((suffix) => base + suffix)];
  const found = candidates.find((candidate) => existsSync(candidate) && /\.[^/]+$/.test(candidate));
  assert.ok(found, `unresolved report import ${specifier} from ${relative(root, importer)}`);
  return found;
}
