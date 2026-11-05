/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Failure Destinations And Deduplicated Console Reporting
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
import { failureText, type NotificationFailure } from "@/lib/notification-details";

/* === Re-Exports === */

/* === Constants === */

const reported = new Set<string>();

/* === Types === */

export type FailureContext = "view" | "action" | "background";

/* === Classes === */

/* === Public Functions === */

export function failureDestinations(context: FailureContext) {
  return { inline: context === "view", toast: context !== "view", history: context !== "view", console: true };
}

export function reportFailure(failure: NotificationFailure, context: string): void {
  if (reported.has(failure.id)) return;
  reported.add(failure.id);
  console.groupCollapsed(failure.title, context);
  try {
    console.error(failureText(failure));
  } finally {
    console.groupEnd();
  }
}

export function routeFailure(failure: NotificationFailure, context: FailureContext) {
  reportFailure(failure, context);
  return failureDestinations(context);
}

/* === Private Functions === */

/* === Entry Point === */
