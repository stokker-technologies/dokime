// Dokime — SPDX-License-Identifier: Apache-2.0
import { commands } from "../../lib/desktop/commands";
import type { DesktopLaunch } from "../../lib/desktop/session";
import type { NativeSession } from "./session";

export async function launchDesktopSession(owner: NativeSession, active: () => boolean, publish: (launch: DesktopLaunch) => void) {
  const launch = await commands.desktopSessionLaunch();
  if (!active()) {
    return;
  }
  publish(launch);
  if (launch.error) {
    return;
  }
  const current = await commands.desktopSessionStatus();
  if (!active()) {
    return;
  }
  if (current.connection.phase === "connected" && current.connection.backend) {
    await owner.refresh();
  } else if (launch.target) {
    void owner.select({ kind: "attach", request: { target: launch.target, expectedBackendId: launch.expectedBackendId } });
  }
}
