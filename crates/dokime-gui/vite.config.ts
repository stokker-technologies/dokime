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
import { tanstackRouter } from "@tanstack/router-plugin/vite";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vitest/config";
import workspace from "../../tools/config/vite.base.ts";

/* === Constants === */

const frontendPort = Number(process.env.DOKIME_DEV_PORT ?? "5173");
const projectRoot = fileURLToPath(new URL(".", import.meta.url));
const packageJson = JSON.parse(readFileSync(new URL("./package.json", import.meta.url), "utf8")) as { version?: string };
const appVersion = packageJson.version ?? "dev";

// The Desktop Builds Live Assets; The Report Package Owns Its Standalone Template.
export default defineConfig(({ mode }) => ({
  root: projectRoot,
  // Concurrent Lanes Own Separate Vite Dependency And Vitest Result Caches.
  cacheDir: `node_modules/.vite/${mode}${mode === "test" ? `/${process.env.DOKIME_TEST_LANE ?? "all"}` : ""}`,
  publicDir: "public",
  define: {
    __APP_VERSION__: JSON.stringify(appVersion),
    __DOKIME_UI_MODE__: JSON.stringify("live"),
  },
  plugins: {
    test: [],
  }[mode] ?? [tanstackRouter({ target: "react", autoCodeSplitting: true }), react(), tailwindcss()],
  resolve: {
    dedupe: workspace.resolve?.dedupe,
    alias: {
      "@": fileURLToPath(new URL("./src", import.meta.url)),
      ...workspace.resolve?.alias,
    },
  },
  server: {
    host: "127.0.0.1",
    port: frontendPort,
    strictPort: true,
    watch: {
      ignored: ["**/.git/**", "**/.playwright-mcp/**", "**/.run/**", "**/.tmp/**", "**/target/**"],
    },
  },
  build: {
    chunkSizeWarningLimit: 800,
    outDir: "dist",
    emptyOutDir: true,
    rolldownOptions: {
      checks: {
        pluginTimings: false,
      },
      output: {
        strictExecutionOrder: true,
        codeSplitting: {
          maxSize: 800_000,
          groups: [
            { name: "plot-vendor", test: /node_modules.*(?:d3-|uplot)/ },
            { name: "tauri-vendor", test: /node_modules.*@tauri-apps/ },
            { name: "react-vendor", test: /node_modules.*(?:react|scheduler)/ },
            {
              name: "document-surface",
              test: /\/src\/(?:components\/document|plots)\//,
            },
          ],
        },
      },
    },
  },
  test: { ...workspace.test, setupFiles: ["tests/setup.ts"] },
}));
