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
import { existsSync, readdirSync, readFileSync } from "node:fs";
import { dirname, relative, resolve, sep } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import type { DescFile } from "@bufbuild/protobuf";
import { beforeAll, test } from "vitest";

/* === Constants === */

const gui = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const descriptors = resolve(gui, ".generated");
const facadePath = "src/lib/rpc/generated/client.ts";

/* === State === */

let files: DescFile[];
let sources: Map<string, string>;

/* === Fixture === */

function generatedSources(directory: string, found = new Map<string, string>()) {
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const path = resolve(directory, entry.name);
    if (entry.isDirectory()) generatedSources(path, found);
    else found.set(relative(gui, path).split(sep).join("/"), readFileSync(path, "utf8"));
  }
  return found;
}

function isFile(value: unknown): value is DescFile {
  return typeof value === "object" && value !== null && (value as DescFile).kind === "file";
}

beforeAll(async () => {
  // The `prepare:contracts` Task Generates The Bindings These Stories Read
  assert.ok(existsSync(descriptors), `${descriptors} is missing; run \`mise run prepare:contracts\``);
  sources = generatedSources(descriptors);
  sources.set(facadePath, readFileSync(resolve(gui, facadePath), "utf8"));

  // Canonical Descriptors, As The Generated Modules Embed Them
  files = [];
  for (const path of sources.keys()) {
    if (!path.endsWith("_pb.ts")) continue;
    const module: Record<string, unknown> = await import(pathToFileURL(resolve(gui, path)).href);
    files.push(...Object.values(module).filter(isFile));
  }
});

/* === Tests === */

test("Canonical Bindings", () => {
  const facade = sources.get(facadePath) ?? "";
  assert.ok(files.length > 0, "the generated descriptors declare no files");

  // Canonical Descriptor Paths
  assert.ok(facade.includes("../../../../.generated/dokime/v1/"));
  assert.equal(facade.includes('from "./dokime/v1/'), false);

  // Canonical Methods
  for (const file of files) {
    for (const service of file.services) {
      for (const method of service.methods) {
        const path = `/${service.typeName}/${method.name}`;
        assert.ok(facade.includes(JSON.stringify(path)), `Missing canonical method: ${path}`);
        const call =
          method.methodKind === "client_streaming" ? "openClientStream" : method.methodKind === "server_streaming" ? "openStream" : "unary";
        assert.ok(facade.includes(`=> ${call}(${JSON.stringify(path)},`), path);
      }
    }
  }

  // LF Identity
  for (const [path, content] of sources) assert.equal(content.includes("\r\n"), false, path);
});
