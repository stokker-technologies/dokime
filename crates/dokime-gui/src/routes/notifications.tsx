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

import { createFileRoute } from "@tanstack/react-router";

import { NotificationsSurface } from "@/features/notifications/NotificationsSurface";

export const Route = createFileRoute("/notifications")({ component: NotificationsSurface });
