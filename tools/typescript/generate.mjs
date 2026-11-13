// Dokime — SPDX-License-Identifier: Apache-2.0
// Generate Frontend Protobuf Modules From the Authoritative Descriptor Set.

import { spawnSync } from "node:child_process";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const gui = resolve(root, "crates/dokime-gui");
const require = createRequire(resolve(gui, "package.json"));
const { create, fromBinary, toBinary } = require("@bufbuild/protobuf");
const { CodeGeneratorRequestSchema, CodeGeneratorResponseSchema, FileDescriptorSetSchema } = require("@bufbuild/protobuf/wkt");
const set = fromBinary(FileDescriptorSetSchema, readFileSync(resolve(root, "crates/dokime-protocol/src/descriptors.bin")));
const request = create(CodeGeneratorRequestSchema, {
  protoFile: set.file,
  fileToGenerate: set.file.map((file) => file.name),
  parameter: "target=ts,import_extension=js",
});
const generated = spawnSync(process.execPath, [resolve(gui, "node_modules/@bufbuild/protoc-gen-es/bin/protoc-gen-es")], {
  input: toBinary(CodeGeneratorRequestSchema, request),
  maxBuffer: 64 * 1024 * 1024,
});
if (generated.status !== 0) throw new Error(generated.stderr.toString());
const response = fromBinary(CodeGeneratorResponseSchema, generated.stdout);
if (response.error) throw new Error(response.error);
const output = resolve(root, "target/generated/typescript");
for (const file of response.file) {
  const path = resolve(output, file.name);
  if (!path.startsWith(output + "/") && !path.startsWith(output + "\\")) throw new Error("Generate Messages: Invalid Protobuf Path");
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, file.content);
}

const protocolSource = readFileSync(resolve(gui, "src-tauri/src/rpc/protocol.rs"), "utf8");
const protocolConstants = [...protocolSource.matchAll(/^pub const ([A-Z_]+): u64 = ([0-9_]+);$/gm)];
if (protocolConstants.length !== 3) throw new Error("Native IPC protocol constants are incomplete.");
writeFileSync(
  resolve(output, "desktop-protocol.ts"),
  "// Generated From The Native IPC Protocol Owner. Do Not Edit.\n" +
    protocolConstants.map(([, name, value]) => `export const ${name} = ${value};`).join("\n") +
    "\n",
);
const nativeSource = readFileSync(resolve(gui, "src-tauri/src/bindings/mod.rs"), "utf8");
const nativePattern =
  /async\s+(desktop_(?:backends_list|backend_import_pairing|backend_remove|backend_use|settings_read|settings_update))\s*\[[\s\S]*?\]\s*\(([\s\S]*?)\)\s*->\s*Result<([^,]+),/g;
const nativeBindings = [...nativeSource.matchAll(nativePattern)];
if (nativeBindings.length !== 6) throw new Error("Native preferences command inventory is incomplete.");
const nativeType = (rust) => {
  const type = rust.trim();
  if (type.startsWith("Option<") && type.endsWith(">")) return `${nativeType(type.slice(7, -1))} | null`;
  return type === "String" ? "string" : type.split("::").at(-1);
};
const nativeLines = [
  "// Generated From The Native Command Inventory. Do Not Edit.",
  'import type { Document, SettingsScope, SettingsSnapshot, UpdateSettings } from "./application-settings";',
  'import type { ConnectionSnapshot } from "@/lib/desktop/connection";',
  "export function createPreferencesCommands(invoke: <T>(command: string, args?: Record<string, unknown>) => Promise<T>) {",
  "  return {",
];
for (const [, command, parameters, result] of nativeBindings) {
  const args = parameters
    .split(",")
    .filter((value) => value.trim())
    .map((value) => {
      const colon = value.indexOf(":");
      return { name: value.slice(0, colon).trim(), type: nativeType(value.slice(colon + 1)) };
    });
  const name = command.replace(/_([a-z])/g, (_, letter) => letter.toUpperCase());
  nativeLines.push(
    `    ${name}: (${args.map((arg) => `${arg.name}: ${arg.type}`).join(", ")}) => invoke<${nativeType(result)}>(${JSON.stringify(command)}${args.length ? `, { ${args.map((arg) => arg.name).join(", ")} }` : ""}),`,
  );
}
nativeLines.push("  };", "}");
writeFileSync(resolve(output, "native-preferences.ts"), `${nativeLines.join("\n")}\n`);
