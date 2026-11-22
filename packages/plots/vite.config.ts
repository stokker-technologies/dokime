// Dokime — SPDX-License-Identifier: Apache-2.0
import { defineConfig } from "vitest/config";
import workspace from "../../tools/config/vite.base.ts";

export default defineConfig({
  resolve: workspace.resolve,
  test: { ...workspace.test },
});
