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
import { readFile, readdir } from "node:fs/promises";
import { registerHooks } from "node:module";
import { fileURLToPath } from "node:url";

// Third Party Imports
import type { DescMessage, JsonValue, MessageShape } from "@bufbuild/protobuf";
import { transformWithOxc } from "vite";
import { fromBinary, fromJson, toBinary, toJson } from "@bufbuild/protobuf";

// Local Imports
import type { DesktopSessionSnapshot } from "../src/lib/desktop/session.ts";
import type { Session } from "./native-fixture.ts";

/* === Re-Exports === */

/* === Constants === */

const generated = new URL("../../../target/generated/host/", import.meta.url);
const inventory = await readFile(new URL("client.ts", generated), "utf8");
const identity = await readFile(new URL("identity.ts", generated), "utf8");
const digest = Buffer.from(identity.match(/descriptorSha256 = "([a-f0-9]+)"/)![1], "hex");
const compiled = new Map();
const pending = [generated];
while (pending.length) {
  const directory = pending.pop()!;
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    const source = new URL(entry.name + (entry.isDirectory() ? "/" : ""), directory);
    if (entry.isDirectory()) pending.push(source);
    else if (entry.name.endsWith(".ts")) {
      const result = await transformWithOxc(await readFile(source, "utf8"), fileURLToPath(source), { target: "es2022", lang: "ts" });
      compiled.set(source.href, result.code);
    }
  }
}

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
  load(url, context, next) {
    if (url.startsWith(generated.href) && url.endsWith(".ts")) {
      const source = compiled.get(url);
      assert.ok(source, `Prepared Host Module Exists: ${url}`);
      return { format: "module", source, shortCircuit: true };
    }
    return next(url, context);
  },
});

/* === Types === */

type Host = typeof import("@host/model");
type ServiceName<S extends string> = S extends `${string}.${infer Rest}` ? ServiceName<Rest> : S;
type ResponseSchema<S extends string, M extends string> = Host[Extract<`${ServiceName<S>}${M}ResponseSchema`, keyof Host>];
type OneofJson<T> = T extends { case: infer Name extends string; value: infer Value } ? { [Key in Name]: JsonMessage<Value> } : object;
type Merge<T> = (T extends unknown ? (value: T) => void : never) extends (value: infer Value) => void ? Value : never;
/** ProtoJSON uses base64 bytes, decimal 64-bit integers, enum names and flattened oneofs. */
type JsonMessage<T> = T extends Uint8Array | bigint
  ? string
  : T extends number
    ? number | string
    : T extends readonly (infer Item)[]
      ? JsonMessage<Item>[]
      : T extends { $typeName: "google.protobuf.Timestamp" }
        ? string
        : T extends object
          ? {
              [Key in keyof T as Key extends "$typeName" | "$unknown"
                ? never
                : NonNullable<T[Key]> extends { case: unknown }
                  ? never
                  : Key]-?: JsonMessage<NonNullable<T[Key]>>;
            } & Merge<
              { [Key in keyof T]-?: NonNullable<T[Key]> extends { case: unknown } ? OneofJson<NonNullable<T[Key]>> : object }[keyof T]
            >
          : T;
export type Reply<S extends string, M extends string> = JsonMessage<MessageShape<Extract<ResponseSchema<S, M>, DescMessage>>>;

/* === Classes === */

/* === Public Functions === */

export async function invoke<T = DesktopSessionSnapshot>(desktop: Session, command: string, args: Record<string, unknown> = {}) {
  const result = await desktop.executeAsync<{ failure?: unknown; value: T }>(
    `const done = arguments[arguments.length - 1];
     window.__TAURI_INTERNALS__.invoke(arguments[0], arguments[1]).then(
       value => done({value}), error => done({failure: error}));`,
    [command, args],
  );
  if (result.failure) throw Object.assign(new Error(JSON.stringify(result.failure)), { detail: result.failure });
  return result.value;
}

export async function contract(service: string, method: string) {
  const path = `/${service.includes(".") ? service : `dokime.host.v1.${service}`}/${method}`;
  const declaration = inventory.match(new RegExp(`unary\\("${path}", (\\w+), (\\w+),`));
  assert.ok(declaration, `Canonical unary contract exists for ${path}`);
  const input = await schema(declaration[1]);
  const output = await schema(declaration[2]);
  return { path, input, output };
}

export async function call<S extends string, M extends string>(
  desktop: Session,
  service: S,
  method: M,
  request: unknown = {},
): Promise<Reply<S, M>> {
  const { path, input, output } = await contract(service, method);
  const status = await invoke(desktop, "desktop_session_status");
  const payload = toBinary(input, fromJson(input, request as JsonValue));
  const bytes = Buffer.alloc(40 + payload.length);
  bytes.writeBigUInt64LE(BigInt(status.connection.generation));
  digest.copy(bytes, 8);
  bytes.set(payload, 40);
  const result = await desktop.executeAsync<{ failure?: unknown; bytes: number[] }>(
    `const done = arguments[arguments.length - 1];
     window.__TAURI_INTERNALS__.invoke('rpc', Uint8Array.from(arguments[0]), {headers: {
       'dokime-rpc-path': arguments[1]
     }}).then(value => done({bytes: [...new Uint8Array(value)]}), error => done({failure: error}));`,
    [[...bytes], path],
  );
  if (result.failure) throw Object.assign(new Error(JSON.stringify(result.failure)), { detail: result.failure });
  const reply = Uint8Array.from(result.bytes);
  assert.equal(new DataView(reply.buffer).getBigUint64(0, true), BigInt(status.connection.generation));
  return toJson(output, fromBinary(output, reply.subarray(8)), { alwaysEmitImplicit: true }) as Reply<S, M>;
}

/* === Private Functions === */

async function schema(alias: string): Promise<DescMessage> {
  const declaration = inventory.match(new RegExp(`import \\{ (\\w+) as ${alias} \\} from "\\./([^"]+)"`));
  assert.ok(declaration, `Generated schema import exists for ${alias}`);
  const module = new URL(declaration[2].replace(/\.js$/, ".ts"), generated);
  assert.ok(existsSync(module), `Prepared protobuf source exists: ${fileURLToPath(module)}`);
  return (await import(module.href))[declaration[1]];
}

/* === Entry Point === */
