/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Plot Package Test Configuration
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
import { defineConfig } from "vitest/config";

// Local Imports

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Public Functions === */

/* === Private Functions === */

/* === Entry Point === */

export default defineConfig({
  test: { environment: "node", include: ["tests/**/*.test.ts"] },
});
