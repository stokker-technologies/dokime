// Dokime — SPDX-License-Identifier: Apache-2.0
import { fileURLToPath } from "node:url";
import { defineConfig } from "vite";

export default defineConfig({
  resolve: {
    dedupe: ["@bufbuild/protobuf"],
    alias: {
      "@host": fileURLToPath(new URL("./target/generated/host", import.meta.url)),
      "@generated": fileURLToPath(new URL("./target/generated/typescript", import.meta.url)),
    },
  },
});
