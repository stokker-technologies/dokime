/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Prove Diagram Rendering
 */

/* === Imports === */

// Standard Library Imports
import assert from "node:assert/strict";
import child_process from "node:child_process";
import { readFileSync, readdirSync, rmSync } from "node:fs";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

// Third Party Imports
import type { Root } from "mdast";

// Local Imports
import { remarkDiagrams } from "../src/diagrams.ts";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Public Functions === */

/* === Private Functions === */

function build(fixture: string) {
  return child_process.spawnSync(process.execPath, ["node_modules/astro/bin/astro.mjs", "build", "--root", "tests"], {
    cwd: fileURLToPath(new URL("..", import.meta.url)),
    env: { ...process.env, ASTRO_TELEMETRY_DISABLED: "1", DOCS_FIXTURE: fixture },
    encoding: "utf8",
    windowsHide: true,
  });
}

/* === Entry Point === */

test("identical sources share rendering across titles and files", (context) => {
  const runner = context.mock.method(child_process, "spawnSync", () => ({
    status: 0,
    stdout: '<svg><defs><marker id="arrow"/></defs><path marker-end="url(#arrow)"/><a href="#guide"><text>host</text></a></svg>',
  }));
  const transform = remarkDiagrams()();
  const first: Root = {
    type: "root",
    children: [{ type: "code", lang: "d2", meta: 'title="A & B"', value: "host -> plugin" }],
  };
  const second: Root = {
    type: "root",
    children: [{ type: "code", lang: "d2", meta: 'title="Another Launch"', value: "host -> plugin" }],
  };
  transform(first, { path: "first.md" });
  transform(second, { path: "second.md" });
  assert.equal(runner.mock.callCount(), 2);
  assert.match(JSON.stringify(first), /A &amp; B/);
  assert.match(JSON.stringify(first), /href=\\"#guide\\"/);
  const identifiers = [...JSON.stringify([first, second]).matchAll(/id=\\"([^\\]+)\\"/g)].map((match) => match[1]);
  assert.equal(identifiers.length, 4);
  assert.equal(new Set(identifiers).size, identifiers.length);
  const edited: Root = {
    type: "root",
    children: [{ type: "code", lang: "d2", value: "source -> updated" }],
  };
  transform(edited, { path: "first.md" });
  assert.equal(runner.mock.callCount(), 4);
  const options = runner.mock.calls[2].arguments[2];
  assert.ok(options);
  assert.equal(options.input, "source -> updated");
});

test("fixture build inlines both themes with selectable text and unique references", () => {
  // Clear Only the Owned Fixture Cache So This Proof Exercises the Compiler.
  rmSync(new URL("../../../target/docs/fixture-cache", import.meta.url), { recursive: true, force: true });
  const result = build("valid");
  assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
  const output = new URL("../../../target/docs/fixtures/", import.meta.url);
  const html = readFileSync(new URL("index.html", output), "utf8");
  const diagrams = [...html.matchAll(/<svg\b[^>]*data-d2-theme="(light|dark)"[^>]*>/g)];
  assert.deepEqual(
    diagrams.map((match) => match[1]),
    ["light", "dark", "light", "dark"],
  );
  assert.ok(diagrams.every((match) => /role="img"/.test(match[0]) && /aria-label="[^"]+"/.test(match[0])));
  assert.match(html, /<text\b/);
  assert.match(html, /data:application\/font/);
  const identifiers = [...html.matchAll(/\bid="([^"]+)"/g)].map((match) => match[1]);
  assert.equal(new Set(identifiers).size, identifiers.length);
  for (const reference of html.matchAll(/url\(#([^)]+)\)/g)) {
    assert.ok(identifiers.includes(reference[1]), `Missing SVG Reference: ${reference[1]}`);
  }
  const assets = new URL("_astro/", output);
  const css = readdirSync(assets)
    .filter((name) => name.endsWith(".css"))
    .map((name) => readFileSync(new URL(name, assets), "utf8"))
    .join("\n");
  assert.match(css, /\[data-theme=light\][^}]*\[data-d2-theme=dark\][^}]*display:none/);
  assert.match(css, /\[data-theme=dark\][^}]*\[data-d2-theme=light\][^}]*display:none/);
  assert.match(css, /\.d2-diagram text\{[^}]*user-select:text/);
});

test("invalid fixture fails the build at the fence's file and line", () => {
  const result = build("invalid");
  assert.notEqual(result.status, 0);
  assert.match(`${result.stdout}\n${result.stderr}`, /fixtures[/\\]invalid[/\\]index\.md:5:1/);
});
