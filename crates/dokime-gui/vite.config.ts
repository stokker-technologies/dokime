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
import { tanstackRouter } from "@tanstack/router-plugin/vite";
import react from "@vitejs/plugin-react";
import { createLogger } from "vite";
import { viteSingleFile } from "vite-plugin-singlefile";
import { defineConfig } from "vitest/config";

const apiPort = process.env.DOKIME_API_PORT ?? "8765";
const frontendPort = Number(process.env.DOKIME_DEV_PORT ?? "5173");
// The desktop reaches its backend only through the shell's IPC bridge; a
// browser build opts into direct gRPC-Web with DOKIME_TRANSPORT=grpc-web.
const transport = process.env.DOKIME_TRANSPORT === "grpc-web" ? "grpc-web" : "ipc";
const apiHttpTarget = `http://127.0.0.1:${apiPort}`;
const apiWsTarget = `ws://127.0.0.1:${apiPort}`;
const projectRoot = fileURLToPath(new URL(".", import.meta.url));
const packageJson = JSON.parse(readFileSync(new URL("./package.json", import.meta.url), "utf8")) as { version?: string };
const appVersion = packageJson.version ?? "dev";
const backendUnavailableWarningTimes = new Map<string, number>();
const backendUnavailableWarningIntervalMs = 15_000;
const logger = createLogger();
const defaultLoggerError = logger.error.bind(logger);

logger.error = (message, options) => {
  if (
    message.includes("http proxy error:") ||
    message.includes("ws proxy error:") ||
    message.includes("ws proxy socket error:") ||
    message.includes("write ECONNABORTED") ||
    message.includes("read ECONNRESET") ||
    message.includes(`ECONNREFUSED 127.0.0.1:${apiPort}`)
  ) {
    return;
  }
  defaultLoggerError(message, options);
};

function shouldWarnBackendUnavailable(url: string) {
  const now = Date.now();
  const warningKey = url === "/api/runtime/ws" ? "backend-ws" : "backend-http";
  const lastWarning = backendUnavailableWarningTimes.get(warningKey) ?? 0;
  if (now - lastWarning < backendUnavailableWarningIntervalMs) {
    return false;
  }
  backendUnavailableWarningTimes.set(warningKey, now);
  return true;
}

function configureApiProxy(proxy: {
  on: (event: "error", callback: (error: NodeJS.ErrnoException, req?: { url?: string }) => void) => void;
}) {
  proxy.on("error", (error, req) => {
    const url = req?.url ?? "/api";
    if (error.code === "ECONNREFUSED") {
      if (shouldWarnBackendUnavailable(url)) {
        console.warn(`[vite] backend unavailable for ${url}; waiting for ${apiHttpTarget}`);
      }
      return;
    }
    if (error.code === "ECONNABORTED" || error.code === "ECONNRESET") {
      return;
    }
    console.warn(`[vite] proxy error for ${url}: ${error.message}`);
  });
}

// `--mode report` builds the single-file report template that the runtime embeds; Vitest runs in `test` mode.
export default defineConfig(({ mode }) => ({
  customLogger: logger,
  root: projectRoot,
  cacheDir: `node_modules/.vite/${mode}`,
  publicDir: mode === "report" ? false : "public",
  define: {
    __APP_VERSION__: JSON.stringify(appVersion),
    __DOKIME_UI_MODE__: JSON.stringify(mode === "report" ? "report" : "live"),
    __DOKIME_TRANSPORT__: JSON.stringify(transport),
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
    proxy: {
      "/api/runtime/ws": {
        target: apiWsTarget,
        ws: true,
        changeOrigin: true,
        configure: configureApiProxy,
      },
      "/api": {
        target: apiHttpTarget,
        ws: true,
        changeOrigin: true,
        configure: configureApiProxy,
      },
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
