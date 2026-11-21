// Dokime — SPDX-License-Identifier: Apache-2.0
import { defineConfig, mergeConfig } from "vitest/config";

import workspace from "./vite.config.ts";

export default mergeConfig(
  workspace,
  defineConfig({
    test: { pool: "threads", environment: "node", include: ["tests/**/*.test.ts"], testTimeout: 240_000, hookTimeout: 240_000 },
  }),
);
