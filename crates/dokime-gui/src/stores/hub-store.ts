/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { createStore } from "zustand/vanilla";

/* === Local Imports === */

import type { BackendOperation } from "@/shell/status-bar-operations";

/* === Types === */

export type ConnectionState = "connecting" | "connected" | "reconnecting";

type HubStoreState = {
  connection: ConnectionState;
  error: string | null;
  /** The backend's slow operations, as the last runtime frame carried them. */
  operations: BackendOperation[];
};

/** The fields the runtime stream updates on every frame. */
export type BackendOperationsPatch = Partial<HubStoreState>;

type HubStore = HubStoreState & {
  setHubState: (patch: BackendOperationsPatch) => void;
};

/* === Public Functions === */

export const hubStore = createStore<HubStore>((set) => ({
  connection: "connecting",
  error: null,
  operations: [],
  setHubState: (patch) => set((state) => ({ ...state, ...patch })),
}));
