// Dokime — SPDX-License-Identifier: Apache-2.0
import { fileURLToPath } from "node:url";
import { defineConfig } from "vitest/config";

export default defineConfig({
  resolve: {
    dedupe: ["@bufbuild/protobuf"],
    alias: {
      "@generated": fileURLToPath(new URL("../../target/generated", import.meta.url)),
    },
  },
  test: { pool: "threads", environment: "node", include: ["tests/**/*.test.ts"], testTimeout: 240_000, hookTimeout: 240_000 },
});
