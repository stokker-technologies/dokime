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
  // The default live build's constants (see `vite.config.ts`), so a story can import any module
  // the shipped surface imports.
  define: {
    __DOKIME_UI_MODE__: JSON.stringify("live"),
  },
  resolve: {
    dedupe: ["@bufbuild/protobuf"],
    alias: {
      "@": fileURLToPath(new URL("./src", import.meta.url)),
      "@generated": fileURLToPath(new URL("../../target/generated/typescript", import.meta.url)),
    },
  },
  test: {
    environment: "node",
    setupFiles: ["tests/setup.ts"],
    include: ["tests/**/*.test.ts"],
    testTimeout: 120_000,
    hookTimeout: 120_000,
  },
});
