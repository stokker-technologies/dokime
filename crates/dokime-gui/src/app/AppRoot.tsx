// Dokime — SPDX-License-Identifier: Apache-2.0

import { RouterProvider } from "@tanstack/react-router";
import { useEffect, useState } from "react";
import { router } from "@/app/router";
import { SvelteMount } from "@/components/SvelteMount";
import { DisconnectedSettings } from "@/features/settings/SettingsSurface";
import { useBrand } from "@/lib/brand-context";
import { commands } from "@/lib/desktop/commands";
import type { DesktopLaunch } from "@/lib/desktop/session";
import { AppProviders } from "./AppProviders";
import ConnectionScreen from "./native-session/ConnectionScreen.svelte";
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
      .then(async (configuration) => {
        if (!active) {
          return;
        }
        setLaunch(configuration);
        if (configuration.error) {
          return;
        }
        const current = await commands.desktopSessionStatus();
        if (!active) {
          return;
        }
        if (current.connection.phase === "connected" && current.connection.backend) {
          await owner.refresh();
        } else if (configuration.target) {
          void owner.select({
            kind: "attach",
            request: {
              target: configuration.target,
              expectedBackendId: configuration.expectedBackendId,
            },
          });
        }
      })
      .catch((error: unknown) => {
        if (active) {
          setError(error);
        }
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
          <DisconnectedDesktop owner={owner} launch={launch} launchError={error} />
        </AppProviders>
      }
    >
      <RouterProvider router={router} />
    </NativeSessionProvider>
  );
}

function DisconnectedDesktop({ owner, launch, launchError }: { owner: NativeSession; launch: DesktopLaunch | null; launchError: unknown }) {
  const [settings, setSettings] = useState(false);
  const brand = useBrand();
  return settings ? (
    <DisconnectedSettings close={() => setSettings(false)} />
  ) : (
    <SvelteMount component={ConnectionScreen} props={{ owner, brand, launch, launchError, onSettings: () => setSettings(true) }} />
  );
}
