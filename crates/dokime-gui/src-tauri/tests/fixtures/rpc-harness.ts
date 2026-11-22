/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Canonical TypeScript Transpilation and Native Adapter Test Transport
 */

/* === Imports === */

// Standard Library Imports
import assert from "node:assert/strict";
import { createInterface } from "node:readline";
import { spawn } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { resolve, dirname, relative, sep } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

// Third Party Imports
import { transformWithOxc } from "vite";

// Local Imports

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Public Functions === */

export async function openHarness() {
  const gui = resolve(dirname(fileURLToPath(import.meta.url)), "../../..");
  const generated = resolve(gui, "../../target/generated/typescript");
  assert.ok(existsSync(generated), `${generated} is missing; run \`mise run generate\``);
  const owner = spawn("uv", ["run", "--locked", "--project", "tools/python", "python", "-m", "dokime_tools.test.fixture"], {
    cwd: resolve(gui, "../.."),
    stdio: ["pipe", "pipe", "inherit"],
  });
  const stopped = new Promise<void>((resolve, reject) => {
    owner.once("error", reject);
    owner.once("exit", (code, signal) => {
      if (code === 0) {
        resolve();
      } else {
        reject(new Error(`Temporary fixture owner exited with ${code ?? signal}`));
      }
    });
  });
  const readiness = createInterface({ input: owner.stdout });
  const temporary = await Promise.race([
    new Promise<string>((resolve) => readiness.once("line", resolve)),
    stopped.then(() => {
      throw new Error("Temporary fixture owner exited before readiness");
    }),
  ]);
  readiness.close();
  writeFileSync(resolve(temporary, "package.json"), JSON.stringify({ type: "module" }));
  const lines = createInterface({ input: process.stdin });
  const pending = new Map<number, { resolve: (value: unknown) => void; reject: (error: unknown) => void }>();
  const callbacks = new Map<number, (frame: { message: ArrayBuffer; index: number }) => void>();
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
      if (message.error !== undefined) {
        call.reject(message.error);
      } else {
        call.resolve(message.bytes !== undefined ? buffer(message.bytes) : message.value);
      }
    }
  })();
  function invoke(cmd: string, args: unknown, options?: { headers?: Record<string, string> }) {
    const id = nextInvoke++;
    const bytes = args instanceof Uint8Array ? args : new Uint8Array(0);
    const headers = options?.headers ?? {};
    return new Promise((resolve, reject) => {
      pending.set(id, { resolve, reject });
      process.stdout.write(`${JSON.stringify({ id, cmd, bytes: Buffer.from(bytes).toString("base64"), headers })}\n`);
    });
  }
  (globalThis as typeof globalThis & { __DOKIME_UI_MODE__: string }).__DOKIME_UI_MODE__ = "live";
  (
    globalThis as unknown as {
      window: {
        __TAURI_INTERNALS__: {
          invoke: typeof invoke;
          transformCallback(callback: (frame: { message: ArrayBuffer; index: number }) => void): number;
          unregisterCallback(id: number): void;
        };
      };
    }
  ).window = {
    __TAURI_INTERNALS__: {
      invoke,
      transformCallback(callback: (frame: { message: ArrayBuffer; index: number }) => void) {
        const id = nextCallback++;
        callbacks.set(id, callback);
        return id;
      },
      unregisterCallback(id: number) {
        callbacks.delete(id);
      },
    },
  };
  // Follow Runtime Imports After Type Erasure, Including Workspace Package Exports.
  const root = resolve(gui, "../..");
  const host = resolve(root, "target/generated/host");
  const compiled = new Set<string>();
  async function compile(input: string): Promise<string> {
    if (!existsSync(input)) {
      input = input.replace(/\.js$/, ".ts");
    }
    if (!input.endsWith(".ts")) {
      input += ".ts";
    }
    const output = resolve(temporary, relative(root, input).replace(/\.ts$/, ".js"));
    if (compiled.has(input)) {
      return output;
    }
    compiled.add(input);
    const result = await transformWithOxc(readFileSync(input, "utf8"), input, { target: "es2022", lang: "ts" });
    let source = result.code;
    for (const match of source.matchAll(/\b(?:from\s*|import\s*)(["'])([^"']+)\1/g)) {
      const specifier = match[2];
      let target: string;
      if (specifier.startsWith("@/")) {
        target = resolve(gui, "src", specifier.slice(2));
      } else if (specifier.startsWith("@generated/")) {
        target = resolve(generated, specifier.slice("@generated/".length));
      } else if (specifier.startsWith("@host/")) {
        target = resolve(host, specifier.slice("@host/".length));
      } else if (specifier.startsWith(".")) {
        target = resolve(dirname(input), specifier);
      } else {
        const url = import.meta.resolve(specifier);
        if (!url.startsWith("file:") || !url.endsWith(".ts")) {
          source = source.replaceAll(`${match[1]}${specifier}${match[1]}`, `${match[1]}${url}${match[1]}`);
          continue;
        }
        target = fileURLToPath(url);
      }
      const dependency = await compile(target);
      const path = relative(dirname(output), dependency).split(sep).join("/");
      const replacement = path.startsWith(".") ? path : `./${path}`;
      source = source.replaceAll(`${match[1]}${specifier}${match[1]}`, `${match[1]}${replacement}${match[1]}`);
    }
    mkdirSync(dirname(output), { recursive: true });
    writeFileSync(output, source);
    return output;
  }
  async function load(input: string) {
    const output = await compile(input);
    return import(pathToFileURL(output).href);
  }
  return {
    invoke,
    load: (path: string) => load(resolve(gui, "src/lib/rpc", path)),
    loadCanonical: (path: string) => load(resolve(generated, path)),
    loadHost: <P extends "client.js" | "model.js">(path: P) =>
      load(resolve(host, path)) as Promise<P extends "client.js" ? typeof import("@host/client") : typeof import("@host/model")>,
    /** Report one observation to the driving test without awaiting a reply. */
    report(value: unknown) {
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

/* === Private Functions === */

function buffer(text: string) {
  const bytes = Buffer.from(text, "base64");
  return bytes.buffer.slice(bytes.byteOffset, bytes.byteOffset + bytes.byteLength);
}

/* === Entry Point === */
