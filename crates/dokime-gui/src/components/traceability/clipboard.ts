/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import { toast } from "sonner";

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
    toast.success("Copied to clipboard");
  } catch {
    toast.error("Could not copy to clipboard");
  }
}
