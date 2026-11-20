// Dokime — SPDX-License-Identifier: Apache-2.0
// Prove Native Loader Watch Configuration Without Browser Timing.

import assert from "node:assert/strict";
import { mkdtempSync, mkdirSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { guides } from "../src/guides.ts";

test("native loader watches the guides root and handles edits, additions and D2 changes", async () => {
  const root = mkdtempSync(join(tmpdir(), "docs-loader-"));
  const docs = join(root, "docs");
  const project = join(root, "sites", "docs");
  const generated = join(root, "target", "docs", "generated");
  const watched: string[] = [];
  const handlers = new Map<string, Array<(path: string) => Promise<void>>>();
  const entries = new Map();
  const errors: unknown[] = [];
  try {
    mkdirSync(docs, { recursive: true });
    mkdirSync(project, { recursive: true });
    mkdirSync(generated, { recursive: true });
    const guide = join(docs, "product-guide.md");
    writeFileSync(guide, "# Start\n\n```d2\nsource -> first\n```\n");
    await guides().load({
      config: { root: pathToFileURL(`${project}/`), srcDir: pathToFileURL(`${project}/src/`) },
      collection: "docs",
      logger: {
        warn() {},
        info() {},
        error(error: unknown) {
          errors.push(error);
        },
      },
      watcher: {
        add(path: string) {
          watched.push(resolve(path));
        },
        on(event: string, handler: (path: string) => Promise<void>) {
          handlers.set(event, [...(handlers.get(event) ?? []), handler]);
        },
      },
      store: {
        keys: () => [...entries.keys()],
        get: (id: string) => entries.get(id),
        set: (entry: { id: string }) => entries.set(entry.id, entry),
        delete: (id: string) => entries.delete(id),
      },
      parseData: async ({ data }: { data: object }) => data,
      generateDigest: (body: string) => body,
      entryTypes: new Map([[".md", { getEntryInfo: ({ contents }: { contents: string }) => ({ body: contents, data: {} }) }]]),
    });
    assert.deepEqual(watched, [docs, generated]);
    assert.deepEqual([...handlers.keys()].sort(), ["add", "change", "unlink"]);
    assert.equal(entries.get("index").data.title, "Start");

    writeFileSync(guide, "# Updated\n\n```d2\nsource -> second\n```\n");
    for (const handler of handlers.get("change") ?? []) {
      await handler(guide);
    }
    assert.equal(entries.get("index").data.title, "Updated");
    assert.match(entries.get("index").body, /source -> second/);

    const added = join(docs, "added.md");
    writeFileSync(added, "# Added\n");
    for (const handler of handlers.get("add") ?? []) {
      await handler(added);
    }
    assert.equal(entries.get("added").data.title, "Added");
    assert.equal(entries.size, 2);
    assert.deepEqual(errors, []);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
