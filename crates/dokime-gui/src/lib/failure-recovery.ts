// Dokime — SPDX-License-Identifier: Apache-2.0
import type { ConnectedSession } from "@/app/native-session/session";
import type { NotificationFailure } from "@/lib/notification-details";

/** Native Actions Are Supplied To Host-Free Failure Presentation. */
export function failureRecovery(session: ConnectedSession | null, failure: NotificationFailure): (() => Promise<void>) | undefined {
  if (failure.action === "Open Settings") {
    return async () => {
      window.location.assign(failure.plugin ? `/settings/plugins/${encodeURIComponent(failure.plugin)}` : "/settings");
    };
  }
  if (!session || !failure.plugin || failure.action !== "Restart Plugin") return undefined;
  const plugin = failure.plugin;
  return () => session.restartPlugin(plugin);
}
