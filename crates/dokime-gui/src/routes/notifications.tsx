/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Notification History Route.
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
import { createFileRoute } from "@tanstack/react-router";

// Local Imports
import { NotificationsSurface } from "@/features/notifications/NotificationsSurface";

/* === Re-Exports === */

/* === Constants === */

export const Route = createFileRoute("/notifications")({ component: NotificationsSurface });

/* === Types === */

/* === Components === */

/* === Hooks === */

/* === Public Functions === */

/* === Private Functions === */
