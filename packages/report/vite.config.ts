// Dokime — SPDX-License-Identifier: Apache-2.0
import { fileURLToPath } from "node:url";
import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { viteSingleFile } from "vite-plugin-singlefile";
import { defineConfig } from "vitest/config";
import workspace from "../../vitest.config.ts";

export default defineConfig({
  publicDir: false,
  define: { __DOKIME_UI_MODE__: JSON.stringify("report") },
  plugins: [react(), tailwindcss(), viteSingleFile()],
  resolve: workspace.resolve,
  build: {
    outDir: "../../target/generated/report",
    emptyOutDir: true,
    rolldownOptions: { input: fileURLToPath(new URL("./report.html", import.meta.url)), checks: { pluginTimings: false } },
  },
  test: { ...workspace.test },
});
