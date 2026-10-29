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


import assert from "node:assert/strict";
import { createInterface } from "node:readline";
import { spawn } from "node:child_process";
import { existsSync, mkdirSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { resolve, dirname, relative, sep } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { transformWithOxc } from "vite";


export async function openHarness() {
  const gui = resolve(dirname(fileURLToPath(import.meta.url)), "../../..");
  const generated = resolve(gui, "../../target/generated/typescript");
  assert.ok(existsSync(generated), `${generated} is missing; run \`mise run prepare:contracts\``);
  const owner = spawn("uv", ["run", "--locked", "--project", "tools/python", "python", "-m", "dokime_tools.fixture"], {
    cwd: resolve(gui, "../.."),
    stdio: ["pipe", "pipe", "inherit"],
  });
  const stopped = new Promise((resolve, reject) => {
    owner.once("error", reject);
    owner.once("exit", (code, signal) => {
      if (code === 0) resolve();
      else reject(new Error(`Temporary fixture owner exited with ${code ?? signal}`));
    });
  });
  const readiness = createInterface({ input: owner.stdout });
  const temporary = await Promise.race([
    new Promise((resolve) => readiness.once("line", resolve)),
    stopped.then(() => { throw new Error("Temporary fixture owner exited before readiness"); }),
  ]);
  readiness.close();
  writeFileSync(resolve(temporary, "package.json"), JSON.stringify({ type: "module" }));
  const lines = createInterface({ input: process.stdin });
  const pending = new Map();
  const callbacks = new Map();
  let nextInvoke = 0;
  let nextCallback = 1;
  // Replies and channel frames interleave: every reply names its invoke, and
  // every frame names the channel callback it belongs to.
  const reading = (async () => {
    for await (const line of lines) {
      const message = JSON.parse(line);
      if ("channel" in message) {
        callbacks.get(message.channel)?.({ message: buffer(message.bytes), index: message.index });
        continue;
      }
      const call = pending.get(message.id);
      assert.ok(call, `unexpected reply ${message.id}`);
      pending.delete(message.id);
      if (message.error !== undefined) call.reject(message.error);
      else call.resolve(message.bytes !== undefined ? buffer(message.bytes) : message.value);
    }
  })();
  function invoke(cmd, args, options) {
    const id = nextInvoke++;
    const bytes = args instanceof Uint8Array ? args : new Uint8Array(0);
    const headers = options?.headers ?? {};
    return new Promise((resolve, reject) => {
      pending.set(id, { resolve, reject });
      process.stdout.write(
        `${JSON.stringify({ id, cmd, bytes: Buffer.from(bytes).toString("base64"), headers })}\n`,
      );
    });
  }
  globalThis.__DOKIME_UI_MODE__ = "live";
  globalThis.__DOKIME_TRANSPORT__ = "ipc";
  globalThis.window = {
    __TAURI_INTERNALS__: {
      invoke,
      transformCallback(callback) {
        const id = nextCallback++;
        callbacks.set(id, callback);
        return id;
      },
      unregisterCallback(id) {
        callbacks.delete(id);
      },
    },
  };
  // Preserve The GUI Tree So Canonical Descriptor Imports Resolve Unchanged.
  function resolvableSpecifiers(source, destination) {
    return source.replace(/(from\s+["'])([^"']+)(["'])/g, (match, prefix, specifier, suffix) => {
      if (specifier.startsWith("@/")) {
        const target = resolve(temporary, "src", specifier.slice(2));
        const path = relative(dirname(destination), target).split(sep).join("/");
        specifier = path.startsWith(".") ? path : `./${path}`;
      }
      if (specifier.startsWith("@generated/")) {
        const target = resolve(temporary, "canonical", specifier.slice("@generated/".length));
        const path = relative(dirname(destination), target).split(sep).join("/");
        specifier = path.startsWith(".") ? path : `./${path}`;
      }
      if (!specifier.startsWith("./") && !specifier.startsWith("../")) return `${prefix}${import.meta.resolve(specifier)}${suffix}`;
      const basename = specifier.slice(specifier.lastIndexOf("/") + 1);
      const extension = basename.includes(".") ? "" : ".js";
      return `${prefix}${specifier}${extension}${suffix}`;
    });
  }
  async function transpile(source, path) {
    const result = await transformWithOxc(source, path, { target: "es2022", lang: "ts" });
    return result.code;
  }
  async function compile(source, destination) {
    mkdirSync(destination, { recursive: true });
    for (const entry of readdirSync(source, { withFileTypes: true })) {
      if (entry.isDirectory()) {
        await compile(resolve(source, entry.name), resolve(destination, entry.name));
        continue;
      }
      if (!entry.name.endsWith(".ts") || entry.name.endsWith(".test.ts") || entry.name.endsWith(".d.ts")) continue;
      const input = resolve(source, entry.name);
      const outputPath = resolve(destination, entry.name.replace(/\.ts$/, ".js"));
      const text = readFileSync(input, "utf8");
      const output = await transpile(text, input);
      writeFileSync(outputPath, resolvableSpecifiers(output, outputPath));
    }
  }
  const rpc = resolve(temporary, "src/lib/rpc");
  const canonical = resolve(temporary, "canonical");
  await compile(resolve(gui, "src/lib/rpc"), rpc);
  await compile(generated, canonical);
  // The transport's runtime imports outside `src/lib/rpc`; type-only imports erase.
  for (const module of ["src/lib/ui-mode", "src/lib/desktop/stopping"]) {
    const input = resolve(gui, `${module}.ts`);
    const output = resolve(temporary, `${module}.js`);
    mkdirSync(dirname(output), { recursive: true });
    writeFileSync(output, resolvableSpecifiers(await transpile(readFileSync(input, "utf8"), input), output));
  }
  return {
    invoke,
    load: (path) => import(pathToFileURL(resolve(rpc, path))),
    loadCanonical: (path) => import(pathToFileURL(resolve(canonical, path))),
    /** Report one observation to the driving test without awaiting a reply. */
    report(value) {
      process.stdout.write(`${JSON.stringify(value)}\n`);
    },
    async close() {
      process.stdout.end();
      lines.close();
      await reading.catch(() => undefined);
      owner.stdin.end();
      await stopped;
    },
  };
}


function buffer(text) {
  const bytes = Buffer.from(text, "base64");
  return bytes.buffer.slice(bytes.byteOffset, bytes.byteOffset + bytes.byteLength);
}
