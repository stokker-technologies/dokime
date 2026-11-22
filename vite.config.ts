// Dokime — SPDX-License-Identifier: Apache-2.0
import { fileURLToPath } from "node:url";
import { defineConfig } from "vite";

export default defineConfig({
  resolve: {
    dedupe: ["@bufbuild/protobuf"],
    alias: {
      "@generated": fileURLToPath(new URL("./target/generated", import.meta.url)),
    },
  },
});
