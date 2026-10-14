/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

import { readFileSync } from "node:fs";
import { fileURLToPath, URL } from "node:url";
import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";
import { viteSingleFile } from "vite-plugin-singlefile";

/* === Constants === */

const packageJson = JSON.parse(
  readFileSync(new URL("./package.json", import.meta.url), "utf8"),
) as { version?: string };
const appVersion = packageJson.version ?? "dev";

export default defineConfig({
  define: {
    __APP_VERSION__: JSON.stringify(appVersion),
    __DOKIME_UI_MODE__: JSON.stringify("report"),
    __DOKIME_TRANSPORT__: JSON.stringify("ipc"),
  },
  plugins: [react(), tailwindcss(), viteSingleFile()],
  resolve: {
    alias: {
      "@": fileURLToPath(new URL("./src", import.meta.url)),
    },
  },
  build: {
    chunkSizeWarningLimit: 800,
    outDir: "../dokime-app/dist-report",
    emptyOutDir: true,
    rolldownOptions: {
      checks: {
        pluginTimings: false,
      },
      input: fileURLToPath(new URL("./report.html", import.meta.url)),
    },
  },
});
