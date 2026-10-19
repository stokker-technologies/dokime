// Dokime — SPDX-License-Identifier: Apache-2.0
// Import Smoke Over The Generated Client: Every Operation Module Loads And Exports Its Calls.

import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { createServer } from "vite";

const gui = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const server = await createServer({
  root: gui,
  configFile: resolve(gui, "vite.config.ts"),
  logLevel: "error",
  appType: "custom",
  server: { middlewareMode: true, hmr: false },
});
try {
  const client = await server.ssrLoadModule("/src/lib/rpc/generated/client.ts");
  const identity = await server.ssrLoadModule("/src/lib/rpc/generated/identity.ts");
  const calls = Object.values(client).filter((value) => typeof value === "function");
  if (calls.length === 0) throw new Error("the generated client exports no operations");
  if (Object.keys(identity).length === 0) throw new Error("the generated identity module exports nothing");
  console.log(`generated client: ${calls.length} operations load`);
} finally {
  await server.close();
}
