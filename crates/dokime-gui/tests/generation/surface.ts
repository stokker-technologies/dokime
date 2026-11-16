/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Compile the Generated Desktop Client Without Product UI Modules.
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
import { pluginsList } from "../../../../target/generated/host/client";
import type { SessionIdentity } from "../../src/lib/rpc/transport";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Public Functions === */

export async function generatedSurface(session: SessionIdentity): Promise<void> {
  await pluginsList(session, {});
}

/* === Private Functions === */

/* === Entry Point === */
