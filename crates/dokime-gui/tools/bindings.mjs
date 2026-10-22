// Dokime — SPDX-License-Identifier: Apache-2.0
// Project Standard ES Bindings From The dokime-server Contract Export Of One Cargo Build.

import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, isAbsolute, relative, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";

const gui = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const require = createRequire(resolve(gui, "package.json"));
const { create, fromBinary, toBinary } = require("@bufbuild/protobuf");
const { CodeGeneratorRequestSchema, CodeGeneratorResponseSchema, FileDescriptorSetSchema } = require("@bufbuild/protobuf/wkt");
const compilerMessages = process.argv[2];
const check = process.argv[3] === "--check";
if (!compilerMessages || (process.argv.length !== 3 && !(process.argv.length === 4 && check)))
  throw new Error("Usage: node tools/bindings.mjs <cargo-compiler-messages.jsonl> [--check]");
const root = resolve(exported(resolve(compilerMessages)), "grpc/operations");
const descriptor = readFileSync(resolve(root, "descriptors.bin"));
const set = fromBinary(FileDescriptorSetSchema, descriptor);
const files = new Map(set.file.map((file) => [file.name, file]));
const ordered = [];
const active = new Set();
const added = new Set();
function include(name) {
  if (added.has(name)) return;
  if (active.has(name)) throw new Error(`Cyclic protobuf file dependency: ${name}`);
  const file = files.get(name);
  if (!file) throw new Error(`Missing protobuf dependency: ${name}`);
  active.add(name);
  for (const dependency of file.dependency) include(dependency);
  active.delete(name);
  added.add(name);
  ordered.push(file);
}
for (const name of files.keys()) include(name);
const request = create(CodeGeneratorRequestSchema, {
  protoFile: ordered,
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
// The Ignored Message Modules Are Always Regenerated; --check Compares Only The Tracked Client.
const output = resolve(gui, "../../target/generated/typescript");
rmSync(output, { recursive: true, force: true });
mkdirSync(output, { recursive: true });
for (const file of response.file) {
  const path = resolve(output, file.name);
  const child = relative(output, path);
  if (!child || isAbsolute(child) || child === ".." || child.startsWith(`..${sep}`)) {
    throw new Error("Invalid generated file path.");
  }
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, file.content.replace(/\r\n/g, "\n"));
}
const contracts = JSON.parse(readFileSync(resolve(root, "contracts.json"), "utf8")).contracts;
const deadlines = new Map(
  contracts
    .filter((contract) => contract.grpc)
    .map((contract) => [
      `/${contract.grpc.package}.${contract.grpc.service}/${contract.grpc.method}`,
      contract.operation.policy.timeout.secs * 1000 + Math.ceil(contract.operation.policy.timeout.nanos / 1_000_000),
    ]),
);
const messages = new Map();
for (const file of set.file) {
  for (const message of file.messageType) messages.set(`.${file.package}.${message.name}`, { file, message });
}
function schema(type, alias, lines) {
  const owner = messages.get(type);
  if (!owner) throw new Error(`Missing generated descriptor ${type}`);
  lines.push(`import { ${owner.message.name}Schema as ${alias} } from "@generated/${owner.file.name.replace(/\.proto$/, "_pb.js")}";`);
  return alias;
}
function fieldType(type, name) {
  const owner = messages.get(type);
  const field = owner?.message.field.find((field) => field.name === name);
  if (!field?.typeName) throw new Error(`Missing canonical ${type}.${name}`);
  return field.typeName;
}
const lines = [
  "// Generated From Canonical Harmos Descriptors. Do Not Edit.",
  'import { unary, openStream, openClientStream, type SessionIdentity } from "../transport";',
];
for (const file of set.file)
  for (const service of file.service)
    for (const method of service.method) {
      const name = service.name[0].toLowerCase() + service.name.slice(1) + method.name;
      const path = `/${file.package}.${service.name}/${method.name}`;
      let input = method.inputType;
      let output = method.outputType;
      let envelope;
      if (method.clientStreaming && method.serverStreaming)
        throw new Error(`${path}: bidirectional operations have no generated binding yet`);
      if (method.clientStreaming) {
        envelope = input;
        input = fieldType(input, "item");
      }
      if (method.serverStreaming) {
        envelope = output;
        output = fieldType(output, "item");
      }
      const requestSchema = schema(input, `${name}Request`, lines);
      const responseSchema = schema(output, `${name}Response`, lines);
      if (envelope) schema(envelope, `${name}Envelope`, lines);
      const call = method.clientStreaming ? "openClientStream" : method.serverStreaming ? "openStream" : "unary";
      lines.push(
        `export const ${name} = (session: SessionIdentity, request: Parameters<typeof ${call}<typeof ${requestSchema}, typeof ${responseSchema}>>[4]) => ${call}(${JSON.stringify(path)}, ${requestSchema}, ${responseSchema}, session, request${envelope ? `, ${name}Envelope` : ""}, ${deadlines.get(path)});`,
      );
    }
emit(resolve(gui, "src/lib/rpc/generated/client.ts"), `${lines.join("\n")}\n`);
emit(
  resolve(gui, "src/lib/rpc/generated/identity.ts"),
  `// Generated From Canonical Harmos Descriptors. Do Not Edit.\nexport const descriptorSha256 = "${createHash("sha256").update(descriptor).digest("hex")}";\n`,
);

/** The `OUT_DIR` Of The One `dokime-server` Build Script Of A Successful Cargo Build. */
function exported(path) {
  const servers = new Set();
  const scripts = new Map();
  const finished = [];
  for (const line of readFileSync(path, "utf8").split("\n")) {
    if (!line.trim()) continue;
    const event = JSON.parse(line);
    if (event.reason === "compiler-artifact" && event.target.name === "dokime-server" && event.target.kind.includes("bin"))
      servers.add(event.package_id);
    else if (event.reason === "build-script-executed")
      scripts.set(event.package_id, [...(scripts.get(event.package_id) ?? []), event.out_dir]);
    else if (event.reason === "build-finished") finished.push(event.success);
  }
  if (finished.length !== 1 || !finished[0] || servers.size !== 1)
    throw new Error("expected one dokime-server contract producer from a successful Cargo build");
  const outputs = new Set(scripts.get([...servers][0]) ?? []);
  if (outputs.size !== 1) throw new Error("the selected build has no single dokime-server build-script output");
  return [...outputs][0];
}

function emit(path, content) {
  if (!check) {
    writeFileSync(path, content);
    return;
  }
  if (readFileSync(path, "utf8").replace(/\r\n/g, "\n") !== content) {
    throw new Error(`Canonical generated source differs: ${path}`);
  }
}
