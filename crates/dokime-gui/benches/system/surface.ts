// Dokime — SPDX-License-Identifier: Apache-2.0
import { mount } from "svelte";
import { NativeSession } from "@/app/native-session/session";
import { commands } from "@/lib/desktop/commands";
import BenchSurface from "./BenchSurface.svelte";
import "@/theme/styles.css";

const owner = new NativeSession();
const launch = await commands.desktopSessionLaunch();
if (!launch.target) {
  throw new Error("The benchmark requires its owned fixture endpoint");
}
await owner.select({ kind: "attach", request: { target: launch.target, expectedBackendId: launch.expectedBackendId } });
const connected = owner.snapshot();
if (connected.phase !== "connected") {
  throw new Error(`Native fixture connection failed: ${JSON.stringify(connected)}`);
}
const container = document.getElementById("root");
if (!container) {
  throw new Error("The benchmark entry point is missing its root");
}
mount(BenchSurface, { target: container, props: { session: connected.session } });
