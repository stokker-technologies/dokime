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
import { viteSingleFile } from "vite-plugin-singlefile";
import { defineConfig } from "vitest/config";

/* === Constants === */

const frontendPort = Number(process.env.DOKIME_DEV_PORT ?? "5173");
const projectRoot = fileURLToPath(new URL(".", import.meta.url));
const packageJson = JSON.parse(readFileSync(new URL("./package.json", import.meta.url), "utf8")) as { version?: string };
const appVersion = packageJson.version ?? "dev";

// `--mode report` builds the single-file report template that the runtime embeds; Vitest runs in `test` mode.
export default defineConfig(({ mode }) => ({
  root: projectRoot,
  cacheDir: `node_modules/.vite/${mode}`,
  publicDir: mode === "report" ? false : "public",
  define: {
    __APP_VERSION__: JSON.stringify(appVersion),
    __DOKIME_UI_MODE__: JSON.stringify(mode === "report" ? "report" : "live"),
  },
  plugins: {
    report: [react(), tailwindcss(), viteSingleFile()],
    test: [],
  }[mode] ?? [tanstackRouter({ target: "react", autoCodeSplitting: true }), react(), tailwindcss()],
  resolve: {
    dedupe: ["@bufbuild/protobuf"],
    alias: {
      "@": fileURLToPath(new URL("./src", import.meta.url)),
      "@generated": fileURLToPath(new URL("../../target/generated/typescript", import.meta.url)),
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
    outDir: mode === "report" ? "../dokime-runtime/dist-report" : "dist",
    emptyOutDir: true,
    rolldownOptions: {
      checks: {
        pluginTimings: false,
      },
      input: mode === "report" ? fileURLToPath(new URL("./report.html", import.meta.url)) : undefined,
      output:
        mode === "report"
          ? undefined
          : {
              strictExecutionOrder: true,
              codeSplitting: {
                maxSize: 800_000,
                groups: [
                  { name: "plot-vendor", test: /node_modules.*(?:d3-|uplot)/ },
                  { name: "tauri-vendor", test: /node_modules.*@tauri-apps/ },
                  { name: "react-vendor", test: /node_modules.*(?:react|scheduler)/ },
                  { name: "document-surface", test: /\/src\/(?:components\/document|plots)\// },
                ],
              },
            },
    },
  },
  test: {
    environment: "node",
    setupFiles: ["tests/setup.ts"],
    include: ["tests/**/*.test.ts"],
    testTimeout: 120_000,
    hookTimeout: 120_000,
  },
}));
