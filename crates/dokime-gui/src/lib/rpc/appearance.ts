// Dokime — SPDX-License-Identifier: Apache-2.0
import type { AppearanceSnapshot } from "@/lib/native-appearance";
import { required } from "@/lib/rpc/canonical";
import * as rpc from "../host/client";
import type { SessionIdentity } from "./transport.js";

/** Read the canonical host appearance; its logos arrive inline as `data:` URIs. */
export async function loadAppearance(session: SessionIdentity): Promise<AppearanceSnapshot> {
  const appearance = required((await rpc.backendAppearance(session, {})).appearance, "Read Appearance");
  if (!session.isCurrent()) throw { kind: "session", detail: { kind: "superseded" } };
  return appearance;
}
