// Dokime — SPDX-License-Identifier: Apache-2.0

import { RouterProvider } from "@tanstack/react-router";
import { useEffect, useState } from "react";
import { router } from "@/app/router";
import { commands } from "@/lib/desktop/commands";
import type { DesktopLaunch } from "@/lib/desktop/session";
import { AppProviders } from "./AppProviders";
import { ConnectionScreen } from "./native-session/ConnectionScreen";
import { NativeSessionProvider } from "./native-session/NativeSessionProvider";
import { NativeSession } from "./native-session/session";

export function AppRoot() {
  const [owner] = useState(() => new NativeSession());
  const [launch, setLaunch] = useState<DesktopLaunch | null>(null);
  const [error, setError] = useState<unknown>();
  useEffect(() => {
    let active = true;
    void commands
      .desktopSessionLaunch()
      .then((configuration) => {
        if (!active) return;
        setLaunch(configuration);
        if (configuration.error) return;
        if (configuration.target) {
          void owner.select({
            kind: "attach",
            request: {
              target: configuration.target,
              expectedBackendId: configuration.expectedBackendId,
              credentials: { kind: configuration.target.kind === "remote" ? "saved" : "none" },
            },
          });
        } else if (configuration.localBackendAvailable) {
          void owner.select({ kind: "owned_local" });
        }
      })
      .catch((error: unknown) => {
        if (active) setError(error);
      });
    return () => {
      active = false;
    };
  }, [owner]);
  return (
    <NativeSessionProvider
      owner={owner}
      disconnected={
        <AppProviders>
          <ConnectionScreen launch={launch} launchError={error} />
        </AppProviders>
      }
    >
      <RouterProvider router={router} />
    </NativeSessionProvider>
  );
}
