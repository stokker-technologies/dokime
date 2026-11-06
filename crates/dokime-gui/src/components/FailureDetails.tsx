/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Shared Collapsed Failure Context
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
import { failureText, type NotificationFailure } from "@/lib/notification-details";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

type FailureDetailsProps = { failure: NotificationFailure };

/* === Components === */

export function FailureDetails({ failure }: FailureDetailsProps) {
  return (
    <details>
      <summary className="cursor-pointer text-sm font-medium">Technical Details</summary>
      <pre className="mt-2 max-h-80 overflow-auto whitespace-pre-wrap break-words text-xs">{failureText(failure)}</pre>
    </details>
  );
}

/* === Hooks === */

/* === Public Functions === */

/* === Private Functions === */
