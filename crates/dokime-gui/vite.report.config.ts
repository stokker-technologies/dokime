/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import { readFileSync } from "node:fs";
import { fileURLToPath, URL } from "node:url";
import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";
import { viteSingleFile } from "vite-plugin-singlefile";

const packageJson = JSON.parse(readFileSync(new URL("./package.json", import.meta.url), "utf8")) as { version?: string };
const appVersion = packageJson.version ?? "dev";

export default defineConfig({
  publicDir: false,
  define: {
    __APP_VERSION__: JSON.stringify(appVersion),
    __DOKIME_UI_MODE__: JSON.stringify("report"),
    __DOKIME_TRANSPORT__: JSON.stringify("ipc"),
  },
  plugins: [react(), tailwindcss(), viteSingleFile()],
  resolve: {
    dedupe: ["@bufbuild/protobuf"],
    alias: {
      "@": fileURLToPath(new URL("./src", import.meta.url)),
      "@generated": fileURLToPath(new URL("../../target/generated/typescript", import.meta.url)),
    },
  },
  build: {
    chunkSizeWarningLimit: 800,
    outDir: "../dokime-runtime/dist-report",
    emptyOutDir: true,
    rolldownOptions: {
      checks: {
        pluginTimings: false,
      },
      input: fileURLToPath(new URL("./report.html", import.meta.url)),
    },
  },
});
