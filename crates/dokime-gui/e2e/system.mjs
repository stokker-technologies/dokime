/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Select And Report Independent Native Desktop Stories.
 */

/* === Imports === */

// Standard Library Imports
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { once } from "node:events";
import { mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { parseArgs } from "node:util";

// Third Party Imports

// Local Imports
import "./test-trace.mjs";

/* === Re-Exports === */

/* === Constants === */

const { values: options } = parseArgs({
  options: {
    profile: { type: "string", default: "dev" },
    junit: { type: "string" },
    select: { type: "string" },
  },
});
const stories = ["native-desktop", "desktop-configuration", "two-gui-remote"];
const selected = options.select ? [options.select.replace(/^e2e[/\\]/, "").replace(/\.(mjs|js)$/, "")] : stories;
for (const name of selected) assert.ok(stories.includes(name), `Unknown system story: ${name}`);
const scratch = await mkdtemp(join(tmpdir(), "dokime-desktop-junit-"));
const suites = [];
let failed = false;

/* === Types === */

/* === Classes === */

/* === Public Functions === */

/* === Private Functions === */

/* === Entry Point === */

try {
  for (const name of selected) {
    const started = performance.now();
    const report = join(scratch, `${name}.xml`);
    const script = name === "desktop-configuration" ? "native-desktop" : name;
    const workflow =
      name === "native-desktop" ? ["--workflow", "execution"] : name === "desktop-configuration" ? ["--workflow", "configuration"] : [];
    const child = spawn(process.execPath, [`e2e/${script}.mjs`, "--profile", options.profile, "--junit", report, ...workflow], {
      stdio: "inherit",
      windowsHide: true,
    });
    const [code] = await once(child, "exit");
    process.stdout.write(`\nStory Timing: ${name} ${((performance.now() - started) / 1000).toFixed(3)} s\n`);
    failed ||= code !== 0;
    const document = await readFile(report, "utf8");
    suites.push(document.slice(document.indexOf("<testsuite "), document.lastIndexOf("</testsuites>")));
  }
} finally {
  if (options.junit) {
    await mkdir(dirname(resolve(options.junit)), { recursive: true });
    await writeFile(options.junit, `<?xml version="1.0"?><testsuites>${suites.join("")}</testsuites>`);
  }
  await rm(scratch, { force: true, recursive: true });
}
if (failed) process.exitCode = 1;
