/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { failureOf } from "@/lib/notification-details";
import { toast } from "@/lib/notifications";

/* === Local Imports === */

/* === Constants === */

/* === Types === */

/* === Public Functions === */

/**
 * Copy traceability text and say so in a toast.
 *
 * Kept out of the component modules so each of them exports components only,
 * which is what lets Fast Refresh replace them in place.
 */
export async function copyTraceabilityText(value: string): Promise<void> {
  try {
    if (!navigator.clipboard?.writeText) {
      throw new Error("Clipboard API is unavailable.");
    }
    await navigator.clipboard.writeText(value);
    toast.success("Copied to Clipboard");
  } catch (error: unknown) {
    toast.error(failureOf(error, "Could Not Copy to Clipboard"));
  }
}

/* === Private Functions === */
