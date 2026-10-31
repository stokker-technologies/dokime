/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Canonical Protobuf Calls Through A Real Native WebView.
 */

/* === Imports === */

// Standard Library Imports
import assert from "node:assert/strict";
import { existsSync } from "node:fs";
import { readFile } from "node:fs/promises";
import { registerHooks } from "node:module";
import { fileURLToPath } from "node:url";

// Third Party Imports
import { fromBinary, fromJson, toBinary, toJson } from "@bufbuild/protobuf";

// Local Imports

/* === Re-Exports === */

/* === Constants === */

const generated = new URL("../../../target/generated/typescript/", import.meta.url);
const inventory = await readFile(new URL("../src/lib/rpc/generated/client.ts", import.meta.url), "utf8");
const identity = await readFile(new URL("../src/lib/rpc/generated/identity.ts", import.meta.url), "utf8");
const digest = Buffer.from(identity.match(/descriptorSha256 = "([a-f0-9]+)"/)[1], "hex");

// Generated Buf Modules Use JavaScript Specifiers For Their TypeScript Siblings.
registerHooks({
  resolve(specifier, context, next) {
    if (specifier.startsWith("@bufbuild/")) return next(specifier, { ...context, parentURL: import.meta.url });
    if (specifier.endsWith(".js") && context.parentURL?.startsWith(generated.href)) {
      const source = new URL(specifier.replace(/\.js$/, ".ts"), context.parentURL);
      if (existsSync(source)) return { url: source.href, shortCircuit: true };
    }
    return next(specifier, context);
  },
});

/* === Types === */

/* === Classes === */

/* === Public Functions === */

export async function invoke(desktop, command, args = {}) {
  const result = await desktop.executeAsync(
    `const done = arguments[arguments.length - 1];
     window.__TAURI_INTERNALS__.invoke(arguments[0], arguments[1]).then(
       value => done({value}), error => done({failure: error}));`,
    [command, args],
  );
  if (result.failure) throw Object.assign(new Error(JSON.stringify(result.failure)), { detail: result.failure });
  return result.value;
}

export async function call(desktop, service, method, request = {}) {
  const path = `/dokime.v1.${service}/${method}`;
  const declaration = inventory.match(new RegExp(`unary\\("${path}", (\\w+), (\\w+),`));
  assert.ok(declaration, `Canonical unary contract exists for ${path}`);
  const input = await schema(declaration[1]);
  const output = await schema(declaration[2]);
  const status = await invoke(desktop, "desktop_session_status");
  const payload = toBinary(input, fromJson(input, request));
  const bytes = Buffer.alloc(40 + payload.length);
  bytes.writeBigUInt64LE(BigInt(status.connection.generation));
  digest.copy(bytes, 8);
  bytes.set(payload, 40);
  const result = await desktop.executeAsync(
    `const done = arguments[arguments.length - 1];
     window.__TAURI_INTERNALS__.invoke('rpc', Uint8Array.from(arguments[0]), {headers: {
       'dokime-rpc-path': arguments[1], 'dokime-rpc-deadline-ms': '30000'
     }}).then(value => done({bytes: [...new Uint8Array(value)]}), error => done({failure: error}));`,
    [[...bytes], path],
  );
  if (result.failure) throw Object.assign(new Error(JSON.stringify(result.failure)), { detail: result.failure });
  const reply = Uint8Array.from(result.bytes);
  assert.equal(new DataView(reply.buffer).getBigUint64(0, true), BigInt(status.connection.generation));
  return toJson(output, fromBinary(output, reply.subarray(8)), { alwaysEmitImplicit: true });
}

/* === Private Functions === */

async function schema(alias) {
  const declaration = inventory.match(new RegExp(`import \\{ (\\w+) as ${alias} \\} from "@generated/([^"]+)"`));
  assert.ok(declaration, `Generated schema import exists for ${alias}`);
  const module = new URL(declaration[2].replace(/\.js$/, ".ts"), generated);
  assert.ok(existsSync(module), `Prepared protobuf source exists: ${fileURLToPath(module)}`);
  return (await import(module.href))[declaration[1]];
}

/* === Entry Point === */
