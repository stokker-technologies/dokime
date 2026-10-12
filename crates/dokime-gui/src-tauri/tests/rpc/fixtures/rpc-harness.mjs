/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Shared TypeScript transpilation and native IPC test transport
 */

/* === Imports === */

// Standard Library Imports
import assert from "node:assert/strict";
import { createInterface } from "node:readline";
import { mkdtempSync, mkdirSync, readdirSync, readFileSync, writeFileSync, rmSync } from "node:fs";
import { resolve, dirname } from "node:path";
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
  // The source tree is written under tsconfig's moduleResolution "Bundler", so
  // hand-authored relative imports (e.g. "./errors") omit extensions; Vite's
  // bundler resolves those fine in the real app. Plain Node ESM does not: it
  // requires an explicit extension on every relative specifier. Rewrite every
  // extensionless relative specifier to end in ".js" instead of hardcoding each
  // offending import, so a newly added extensionless import keeps working here.
  function resolvableSpecifiers(source) {
    return source
      .replaceAll('"@/lib/ui-mode"', '"./ui-mode.js"')
      .replace(/(from\s+["'])(\.\.?\/[^"']+)(["'])/g, (match, prefix, specifier, suffix) => {
        const basename = specifier.slice(specifier.lastIndexOf("/") + 1);
        return basename.includes(".") ? match : `${prefix}${specifier}.js${suffix}`;
      });
  }
  function compile(source, destination) {
    mkdirSync(destination, { recursive: true });
    for (const entry of readdirSync(source, { withFileTypes: true })) {
      if (entry.isDirectory())
        compile(resolve(source, entry.name), resolve(destination, entry.name));
      else if (entry.name.endsWith(".ts") && !entry.name.endsWith(".test.ts")) {
        const text = readFileSync(resolve(source, entry.name), "utf8");
        const output = resolvableSpecifiers(
          ts.transpileModule(text, {
            compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.ESNext },
          }).outputText,
        );
        writeFileSync(resolve(destination, entry.name.replace(/\.ts$/, ".js")), output);
      }
    }
  }
  compile(resolve(gui, "src/lib/rpc"), temporary);
  writeFileSync(
    resolve(temporary, "ui-mode.js"),
    ts.transpileModule(readFileSync(resolve(gui, "src/lib/ui-mode.ts"), "utf8"), {
      compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.ESNext },
    }).outputText,
  );
  return {
    invoke,
    load: (path) => import(pathToFileURL(resolve(temporary, path))),
    close() {
      lines.close();
      rmSync(temporary, { recursive: true, force: true });
    },
  };
}

/* === Private Functions === */

/* === Entry Point === */
