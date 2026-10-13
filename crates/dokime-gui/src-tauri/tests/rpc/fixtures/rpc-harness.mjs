/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Canonical TypeScript Transpilation And Native Adapter Test Transport
 */

/* === Imports === */

// Standard Library Imports
import assert from "node:assert/strict";
import { createInterface } from "node:readline";
import { mkdtempSync, mkdirSync, readdirSync, readFileSync, writeFileSync, rmSync } from "node:fs";
import { resolve, dirname, relative, sep } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { createRequire } from "node:module";

// Third Party Imports

// Local Imports

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Public Functions === */

export function openHarness() {
  const require = createRequire(
    new URL("../../../../package.json", import.meta.url),
  );
  const ts = require("typescript");
  const gui = resolve(dirname(fileURLToPath(import.meta.url)), "../../../..");
  const cache = resolve(gui, "node_modules/.cache");
  mkdirSync(cache, { recursive: true });
  const temporary = mkdtempSync(resolve(cache, "dokime-rpc-interop-"));
  const lines = createInterface({ input: process.stdin });
  const replies = lines[Symbol.asyncIterator]();
  async function invoke(cmd, bytes) {
    process.stdout.write(`${JSON.stringify({ cmd, bytes: [...bytes] })}\n`);
    const line = await replies.next();
    assert.equal(line.done, false);
    const result = JSON.parse(line.value);
    if (result.error) throw result.error;
    return result.bytes ? Uint8Array.from(result.bytes).buffer : result.value;
  }
  globalThis.__DOKIME_UI_MODE__ = "live";
  globalThis.window = { __TAURI_INTERNALS__: { invoke } };
  // Preserve The GUI Tree So Canonical Descriptor Imports Resolve Unchanged.
  function resolvableSpecifiers(source, destination) {
    return source.replace(/(from\s+["'])([^"']+)(["'])/g, (match, prefix, specifier, suffix) => {
      if (specifier.startsWith("@/")) {
        const target = resolve(temporary, "src", specifier.slice(2));
        const path = relative(dirname(destination), target).split(sep).join("/");
        specifier = path.startsWith(".") ? path : `./${path}`;
      }
      if (!specifier.startsWith("./") && !specifier.startsWith("../")) return match;
      const basename = specifier.slice(specifier.lastIndexOf("/") + 1);
      const extension = basename.includes(".") ? "" : ".js";
      return `${prefix}${specifier}${extension}${suffix}`;
    });
  }
  function compile(source, destination) {
    mkdirSync(destination, { recursive: true });
    for (const entry of readdirSync(source, { withFileTypes: true })) {
      if (entry.isDirectory()) {
        compile(resolve(source, entry.name), resolve(destination, entry.name));
        continue;
      }
      if (!entry.name.endsWith(".ts") || entry.name.endsWith(".test.ts") || entry.name.endsWith(".d.ts")) continue;
      const input = resolve(source, entry.name);
      const outputPath = resolve(destination, entry.name.replace(/\.ts$/, ".js"));
      const text = readFileSync(input, "utf8");
      const output = ts.transpileModule(text, {
        compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.ESNext },
      }).outputText;
      writeFileSync(outputPath, resolvableSpecifiers(output, outputPath));
    }
  }
  const rpc = resolve(temporary, "src/lib/rpc");
  const canonical = resolve(temporary, ".generated");
  compile(resolve(gui, "src/lib/rpc"), rpc);
  compile(resolve(gui, ".generated"), canonical);
  const uiMode = resolve(temporary, "src/lib/ui-mode.js");
  const uiSource = readFileSync(resolve(gui, "src/lib/ui-mode.ts"), "utf8");
  const uiOutput = ts.transpileModule(uiSource, {
    compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.ESNext },
  }).outputText;
  writeFileSync(uiMode, resolvableSpecifiers(uiOutput, uiMode));
  return {
    invoke,
    load: (path) => import(pathToFileURL(resolve(rpc, path))),
    loadCanonical: (path) => import(pathToFileURL(resolve(canonical, path))),
    close() {
      lines.close();
      rmSync(temporary, { recursive: true, force: true });
    },
  };
}

/* === Private Functions === */

/* === Entry Point === */
