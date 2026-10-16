/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

import { fileURLToPath } from "node:url";
import { defineConfig } from "vitest/config";

/* === Configuration === */

// Stories exercise the desktop client's models in Node; the evidence runner adds the JUnit
// reporter and the per-test deadline on its command line.
export default defineConfig({
  resolve: {
    alias: { "@": fileURLToPath(new URL("./src", import.meta.url)) },
  },
  test: {
    environment: "node",
    include: ["tests/**/*.test.ts"],
    testTimeout: 120_000,
    hookTimeout: 120_000,
  },
});
