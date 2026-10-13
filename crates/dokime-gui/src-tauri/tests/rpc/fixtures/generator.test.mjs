/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import { cpSync, mkdirSync, mkdtempSync, readdirSync, readFileSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, relative, resolve, sep } from "node:path";
import { after, before, test } from "node:test";
import { fileURLToPath } from "node:url";

/* === Constants === */

const gui = resolve(dirname(fileURLToPath(import.meta.url)), "../../../..");
const repository = resolve(gui, "../..");
const require = createRequire(resolve(gui, "package.json"));
const { fromBinary } = require("@bufbuild/protobuf");
const { FileDescriptorSetSchema } = require("@bufbuild/protobuf/wkt");

/* === State === */

let temporary;
let selected;
let baseline;

/* === Fixture === */

function run(args = []) {
  return spawnSync(process.execPath, [resolve(temporary, "tools/bindings.mjs"), selected, ...args], {
    cwd: temporary,
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
  });
}

function assertSucceeded(result) {
  assert.ifError(result.error);
  assert.equal(result.status, 0, result.stderr || result.stdout);
}

function snapshot() {
  const files = new Map();
  function visit(directory) {
    for (const entry of readdirSync(directory, { withFileTypes: true })) {
      const path = resolve(directory, entry.name);
      if (entry.isDirectory()) visit(path);
      else files.set(relative(temporary, path).split(sep).join("/"), readFileSync(path, "utf8"));
    }
  }
  visit(resolve(temporary, ".generated"));
  visit(resolve(temporary, "src/lib/rpc/generated"));
  return files;
}

before(() => {
  // Select The Same Explicit Cargo Export That The Repository Binding Task Uses.
  if (process.env.DOKIME_CONTRACT_EXPORT) {
    selected = resolve(process.env.DOKIME_CONTRACT_EXPORT);
  } else {
    assert.ok(process.env.DOKIME_CONTRACT_BUILD_JSON,
      "Generator tests require DOKIME_CONTRACT_EXPORT or DOKIME_CONTRACT_BUILD_JSON from the selected canonical producer.");
    const selection = spawnSync(process.env.UV ?? "uv", [
      "run", "--no-sync", "python", "-m", "dokime_tools.contracts", "locate", process.env.DOKIME_CONTRACT_BUILD_JSON,
    ], { cwd: repository, encoding: "utf8" });
    assertSucceeded(selection);
    selected = resolve(selection.stdout.trim());
  }
  const cache = resolve(gui, "node_modules/.cache");
  mkdirSync(cache, { recursive: true });
  temporary = mkdtempSync(resolve(cache, "dokime-generator-"));
  mkdirSync(resolve(temporary, "tools"), { recursive: true });
  mkdirSync(resolve(temporary, "src/lib/rpc/generated"), { recursive: true });
  cpSync(resolve(gui, "package.json"), resolve(temporary, "package.json"));
  cpSync(resolve(gui, "tools/bindings.mjs"), resolve(temporary, "tools/bindings.mjs"));
  symlinkSync(resolve(gui, "node_modules"), resolve(temporary, "node_modules"), process.platform === "win32" ? "junction" : "dir");
  assertSucceeded(run());
  baseline = snapshot();
});

after(() => {
  if (temporary) rmSync(temporary, { recursive: true, force: true });
});

/* === Tests === */

test("Canonical Descriptor Paths", () => {
  const descriptor = readFileSync(resolve(selected, "descriptors.bin"));
  const set = fromBinary(FileDescriptorSetSchema, descriptor);
  for (const file of set.file) {
    const path = `.generated/${file.name.replace(/\.proto$/, "_pb.ts")}`;
    assert.ok(baseline.has(path), `Missing generated descriptor: ${path}`);
    assert.equal(path.includes("\\"), false);
  }
  const client = baseline.get("src/lib/rpc/generated/client.ts");
  assert.ok(client);
  assert.ok(client.includes("../../../../.generated/dokime/v1/"));
  assert.equal(client.includes('from "./dokime/v1/'), false);
});

test("Canonical Methods And Descriptor Identity", () => {
  const descriptor = readFileSync(resolve(selected, "descriptors.bin"));
  const set = fromBinary(FileDescriptorSetSchema, descriptor);
  const client = baseline.get("src/lib/rpc/generated/client.ts");
  for (const file of set.file) {
    for (const service of file.service) {
      for (const method of service.method) {
        const path = `/${file.package}.${service.name}/${method.name}`;
        if (method.clientStreaming) {
          if (path === "/dokime.v1.Plugins/Stage") assert.ok(client.includes('"plugin_stage_open"'));
          continue;
        }
        assert.ok(client.includes(JSON.stringify(path)), `Missing canonical method: ${path}`);
        const call = method.serverStreaming ? "openStream" : "unary";
        assert.ok(client.includes(`=> ${call}(${JSON.stringify(path)},`));
      }
    }
  }
  const identity = baseline.get("src/lib/rpc/generated/identity.ts");
  const expected = createHash("sha256").update(descriptor).digest("hex");
  assert.ok(identity.includes(`descriptorSha256 = "${expected}"`));
});

test("LF Identity And Deterministic Generation", () => {
  for (const [path, content] of baseline) assert.equal(content.includes("\r\n"), false, path);
  assertSucceeded(run());
  assert.deepEqual(snapshot(), baseline);
  assertSucceeded(run(["--check"]));
});

test("CRLF Comparison Preserves The Checkout", () => {
  for (const [path, content] of baseline) {
    writeFileSync(resolve(temporary, path), content.replace(/\n/g, "\r\n"));
  }
  try {
    assertSucceeded(run(["--check"]));
    for (const [path, content] of baseline) {
      assert.equal(readFileSync(resolve(temporary, path), "utf8"), content.replace(/\n/g, "\r\n"));
    }
  } finally {
    for (const [path, content] of baseline) writeFileSync(resolve(temporary, path), content);
  }
});

test("Canonical Check Rejects Changed Output", () => {
  const path = resolve(temporary, "src/lib/rpc/generated/identity.ts");
  const original = readFileSync(path, "utf8");
  writeFileSync(path, original.replace(/descriptorSha256 = "[a-f0-9]+"/, 'descriptorSha256 = "changed"'));
  try {
    const result = run(["--check"]);
    assert.ifError(result.error);
    assert.notEqual(result.status, 0);
    assert.match(result.stderr, /Canonical generated source differs/);
    assert.equal(readFileSync(path, "utf8").includes('descriptorSha256 = "changed"'), true);
  } finally {
    writeFileSync(path, original);
  }
});
