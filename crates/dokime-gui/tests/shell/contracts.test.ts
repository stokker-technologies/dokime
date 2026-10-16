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
import { type SpawnSyncReturns, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  cpSync,
  mkdirSync,
  mkdtempSync,
  readdirSync,
  readFileSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { dirname, relative, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";
import { fromBinary } from "@bufbuild/protobuf";
import { FileDescriptorSetSchema } from "@bufbuild/protobuf/wkt";
import { afterAll, beforeAll, test } from "vitest";

/* === Constants === */

const gui = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const repository = resolve(gui, "../..");

/* === State === */

let temporary: string | undefined;
let selected: string;
let baseline: Map<string, string>;

/* === Fixture === */

function workspace(): string {
  assert.ok(temporary, "the generator workspace was not prepared");
  return temporary;
}

function run(args: string[] = []) {
  const root = workspace();
  return spawnSync(process.execPath, [resolve(root, "tools/bindings.mjs"), selected, ...args], {
    cwd: root,
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
  });
}

function assertSucceeded(result: SpawnSyncReturns<string>) {
  assert.ifError(result.error);
  assert.equal(result.status, 0, result.stderr || result.stdout);
}

function snapshot() {
  const root = workspace();
  const files = new Map<string, string>();
  function visit(directory: string) {
    for (const entry of readdirSync(directory, { withFileTypes: true })) {
      const path = resolve(directory, entry.name);
      if (entry.isDirectory()) visit(path);
      else files.set(relative(root, path).split(sep).join("/"), readFileSync(path, "utf8"));
    }
  }
  visit(resolve(root, ".generated"));
  visit(resolve(root, "src/lib/rpc/generated"));
  return files;
}

function descriptors() {
  const descriptor = readFileSync(resolve(selected, "descriptors.bin"));
  return { descriptor, set: fromBinary(FileDescriptorSetSchema, descriptor) };
}

function generated(path: string): string {
  const content = baseline.get(path);
  assert.ok(content, `Missing generated source: ${path}`);
  return content;
}

beforeAll(() => {
  // Select The Same Explicit Cargo Export That The Repository Binding Task Uses
  if (process.env.DOKIME_CONTRACT_EXPORT) {
    selected = resolve(process.env.DOKIME_CONTRACT_EXPORT);
  } else {
    assert.ok(
      process.env.DOKIME_CONTRACT_BUILD_JSON,
      "Generator tests require DOKIME_CONTRACT_EXPORT or DOKIME_CONTRACT_BUILD_JSON from the selected canonical producer.",
    );
    const selection = spawnSync(
      process.env.UV ?? "uv",
      [
        "run",
        "--no-sync",
        "python",
        "-m",
        "dokime_tools.contracts",
        "locate",
        process.env.DOKIME_CONTRACT_BUILD_JSON,
      ],
      { cwd: repository, encoding: "utf8" },
    );
    assertSucceeded(selection);
    selected = resolve(selection.stdout.trim());
  }

  // Generate Into A Disposable Copy Of The Binding Tool
  const cache = resolve(gui, "node_modules/.cache");
  mkdirSync(cache, { recursive: true });
  temporary = mkdtempSync(resolve(cache, "dokime-generator-"));
  mkdirSync(resolve(temporary, "tools"), { recursive: true });
  mkdirSync(resolve(temporary, "src/lib/rpc/generated"), { recursive: true });
  cpSync(resolve(gui, "package.json"), resolve(temporary, "package.json"));
  cpSync(resolve(gui, "tools/bindings.mjs"), resolve(temporary, "tools/bindings.mjs"));
  symlinkSync(
    resolve(gui, "node_modules"),
    resolve(temporary, "node_modules"),
    process.platform === "win32" ? "junction" : "dir",
  );
  assertSucceeded(run());
  baseline = snapshot();
});

afterAll(() => {
  if (temporary) rmSync(temporary, { recursive: true, force: true });
});

/* === Tests === */

test("Canonical Bindings", () => {
  const { descriptor, set } = descriptors();
  const client = generated("src/lib/rpc/generated/client.ts");

  // Canonical Descriptor Paths
  for (const file of set.file) {
    const path = `.generated/${file.name.replace(/\.proto$/, "_pb.ts")}`;
    assert.ok(baseline.has(path), `Missing generated descriptor: ${path}`);
    assert.equal(path.includes("\\"), false);
  }
  assert.ok(client.includes("../../../../.generated/dokime/v1/"));
  assert.equal(client.includes('from "./dokime/v1/'), false);

  // Canonical Methods
  for (const file of set.file) {
    for (const service of file.service) {
      for (const method of service.method) {
        const path = `/${file.package}.${service.name}/${method.name}`;
        assert.ok(client.includes(JSON.stringify(path)), `Missing canonical method: ${path}`);
        const call = method.clientStreaming
          ? "openClientStream"
          : method.serverStreaming
            ? "openStream"
            : "unary";
        assert.ok(client.includes(`=> ${call}(${JSON.stringify(path)},`));
      }
    }
  }

  // Descriptor Identity
  const identity = generated("src/lib/rpc/generated/identity.ts");
  const expected = createHash("sha256").update(descriptor).digest("hex");
  assert.ok(identity.includes(`descriptorSha256 = "${expected}"`));
});

test("Deterministic Generation", () => {
  // LF Identity
  for (const [path, content] of baseline) assert.equal(content.includes("\r\n"), false, path);

  // Regeneration Reproduces The Baseline And Passes The Check
  assertSucceeded(run());
  assert.deepEqual(snapshot(), baseline);
  assertSucceeded(run(["--check"]));
});

test("Binding Check", () => {
  const root = workspace();

  // CRLF Comparison Preserves The Checkout
  for (const [path, content] of baseline) {
    writeFileSync(resolve(root, path), content.replace(/\n/g, "\r\n"));
  }
  try {
    assertSucceeded(run(["--check"]));
    for (const [path, content] of baseline) {
      assert.equal(readFileSync(resolve(root, path), "utf8"), content.replace(/\n/g, "\r\n"));
    }
  } finally {
    for (const [path, content] of baseline) writeFileSync(resolve(root, path), content);
  }

  // Canonical Check Rejects Changed Output
  const path = resolve(root, "src/lib/rpc/generated/identity.ts");
  const original = readFileSync(path, "utf8");
  writeFileSync(
    path,
    original.replace(/descriptorSha256 = "[a-f0-9]+"/, 'descriptorSha256 = "changed"'),
  );
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
