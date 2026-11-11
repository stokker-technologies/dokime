/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

import { createInterface } from "node:readline";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Public Functions === */

/* === Private Functions === */

/* === Entry Point === */

// Reuse The Interface's Installed Vite Harness And Its Source Aliases.
const { createServer } = await import(pathToFileURL(resolve("node_modules/vite/dist/node/index.js")).href);
const server = await createServer({
  mode: "test",
  server: { middlewareMode: true },
  logLevel: "error",
  resolve: { alias: { "@generated": process.argv[4].replaceAll("\\", "/") } },
});
try {
  const fixture = await server.ssrLoadModule(resolve("../dokime-server/benches/plots/interface.ts"));
  let measure;
  console.log("ready");
  for await (const line of createInterface({ input: process.stdin })) {
    const [operation, iterations] = line.split(" ");
    if (operation === "prepare") {
      measure = fixture.prepare(process.argv[2], process.argv[3]);
      console.log("prepared");
      continue;
    }
    if (!measure) throw new Error("Frame fixtures are not prepared.");
    console.log(measure(operation, Number(iterations)));
  }
} finally {
  await server.close();
}
