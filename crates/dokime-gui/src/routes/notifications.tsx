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
import { createFileRoute, useRouter } from "@tanstack/react-router";

// Local Imports
import { SvelteMount } from "@/components/SvelteMount";
import NotificationsSurface from "@/features/notifications/NotificationsSurface.svelte";
import { PageShell } from "@/shell/PageShell";

/* === Re-Exports === */

/* === Constants === */

export const Route = createFileRoute("/notifications")({ component: NotificationsRoute });

/* === Types === */

/* === Components === */

function NotificationsRoute() {
  const router = useRouter();
  return (
    <PageShell trail={["Notifications"]}>
      <SvelteMount component={NotificationsSurface} props={{ onNavigate: (href) => void router.navigate({ href }) }} />
    </PageShell>
  );
}

/* === Hooks === */

/* === Public Functions === */

/* === Private Functions === */
