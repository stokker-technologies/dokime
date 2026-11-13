// Dokime — SPDX-License-Identifier: Apache-2.0
// Generate Frontend Protobuf Modules From the Authoritative Descriptor Set.

import { spawnSync } from "node:child_process";
import { mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { parseArgs } from "node:util";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const gui = resolve(root, "crates/dokime-gui");
const require = createRequire(resolve(gui, "package.json"));
const { create, fromBinary, toBinary } = require("@bufbuild/protobuf");
const { CodeGeneratorRequestSchema, CodeGeneratorResponseSchema, FileDescriptorSetSchema } = require("@bufbuild/protobuf/wkt");
const set = fromBinary(FileDescriptorSetSchema, readFileSync(resolve(root, "crates/dokime-protocol/src/descriptors.bin")));
const { values: options } = parseArgs({ options: { output: { type: "string" }, defaults: { type: "string" } } });
const output = resolve(options.output ?? resolve(root, "target/generated/typescript"));
if (output === root || output === gui) throw new Error("Generate Messages: Output Must Be a Generated Module Directory");
rmSync(output, { recursive: true, force: true });
mkdirSync(output, { recursive: true });

// The Standard gRPC Error Envelope Is a Frontend Input Alongside Product Messages.
const statusDescriptor = resolve(output, "status.bin");
const status = spawnSync("protoc", [
  `--proto_path=${resolve(gui, "tools/proto")}`,
  `--descriptor_set_out=${statusDescriptor}`,
  "--include_imports",
  "google/rpc/status.proto",
], { encoding: "utf8" });
if (status.status !== 0) throw new Error(status.stderr || "Generate Messages: Status Descriptor Export Failed");
for (const file of fromBinary(FileDescriptorSetSchema, readFileSync(statusDescriptor)).file) {
  if (!set.file.some((existing) => existing.name === file.name)) set.file.push(file);
}
rmSync(statusDescriptor);

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
  if (type === "Vec<u8>") return "number[]";
  if (type.startsWith("Vec<") && type.endsWith(">")) return `Array<${nativeType(type.slice(4, -1))}>`;
  if (type === "tauri::ipc::Response") return "ArrayBuffer";
  return type === "String" ? "string" : type.split("::").at(-1);
};
const nativeLines = [
  "// Generated From The Native Command Inventory. Do Not Edit.",
  'import type { Document, SettingsScope } from "./application-settings";',
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

// Project Preference View Models From Their Canonical Protobuf and Native Declarations.
const messageTypes = new Map();
const enumTypes = new Map();
for (const file of set.file) {
  for (const value of file.messageType) messageTypes.set(`.${file.package}.${value.name}`, value);
  for (const value of file.enumType) enumTypes.set(`.${file.package}.${value.name}`, value);
}
const models = new Map();
function preferenceType(name) {
  const typeName = name.split(".").at(-1);
  if (models.has(typeName)) return typeName;
  const enumeration = enumTypes.get(name);
  if (enumeration) {
    const prefix = enumeration.value[0].name.replace(/UNSPECIFIED$/, "");
    const values = enumeration.value.filter((item) => !["ThemeMode", "Density"].includes(typeName) || item.number !== 0);
    const variants = values.map((item) => JSON.stringify(item.name.slice(prefix.length).toLowerCase()));
    models.set(typeName, `export type ${typeName} = ${variants.join(" | ")};`);
    return typeName;
  }
  const message = messageTypes.get(name);
  if (!message) throw new Error(`Generate Preferences: Unknown Type ${name}`);
  models.set(typeName, "");
  const fields = message.field.map((field) => {
    let type = field.typeName ? preferenceType(field.typeName) : field.type === 9 ? "string" : field.type === 8 ? "boolean" : "number";
    if (field.label === 3) type = `Array<${type}>`;
    if (field.proto3Optional) type += " | null";
    return `  ${field.name}: ${type};`;
  });
  models.set(typeName, `export interface ${typeName} {\n${fields.join("\n")}\n}`);
  return typeName;
}
for (const name of ["ApplicationSettings", "SettingsSnapshot", "UpdateSettings"]) preferenceType(`.dokime.v1.${name}`);
const registry = readFileSync(resolve(gui, "src-tauri/src/connection/registry.rs"), "utf8");
for (const name of ["Document", "Profile"]) {
  const declaration = registry.match(new RegExp(`pub struct ${name} \{([^}]+)\}`));
  if (!declaration) throw new Error(`Generate Preferences: Missing Native ${name}`);
  const fields = [...declaration[1].matchAll(/pub (\w+): ([^,]+),/g)].map(([, field, type]) => `  ${field}: ${nativeType(type).replace(/^Vec<(.+)>$/, "Array<$1>")};`);
  models.set(name, `export interface ${name} {\n${fields.join("\n")}\n}`);
}
const preferences = readFileSync(resolve(gui, "src-tauri/src/app/preferences.rs"), "utf8");
const scope = preferences.match(/pub enum SettingsScope \{([^}]+)\}/);
if (!scope) throw new Error("Generate Preferences: Missing Native Scope");
models.set("SettingsScope", `export type SettingsScope = ${scope[1].split(",").map((value) => value.trim()).filter(Boolean).map((value) => JSON.stringify(value.toLowerCase())).join(" | ")};`);
writeFileSync(resolve(output, "application-settings.ts"), `// Generated From Host Preferences and Native Registry Declarations. Do Not Edit.\n${[...models.values()].join("\n")}\n`);

// Read Defaults Exported Through the Desktop's Real Embedded Host.
const { createFileRegistry } = require("@bufbuild/protobuf");
const registryTypes = createFileRegistry(set);
const snapshotType = registryTypes.getMessage("dokime.v1.SettingsSnapshot");
const defaults = fromBinary(snapshotType, readFileSync(options.defaults ?? resolve(root, "target/generated/preferences/defaults.bin")));
function project(message, descriptor) {
  const value = {};
  for (const field of descriptor.fields) {
    const current = message[field.localName];
    if (field.fieldKind === "message") value[field.name] = project(current, field.message);
    else if (field.fieldKind === "enum") {
      const enumeration = enumTypes.get(`.${field.enum.typeName}`);
      const prefix = enumeration.value[0].name.replace(/UNSPECIFIED$/, "");
      value[field.name] = field.enum.values.find((item) => item.number === current).name.slice(prefix.length).toLowerCase();
    } else value[field.name] = typeof current === "bigint" ? Number(current) : current ?? null;
  }
  return value;
}
const settingsField = snapshotType.fields.find((field) => field.name === "settings");
writeFileSync(resolve(output, "application-settings.json"), `${JSON.stringify(project(defaults.settings, settingsField.message), null, 2)}\n`);
