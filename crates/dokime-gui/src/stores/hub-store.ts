/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import { create } from "zustand";

import type { BackendOperation } from "@/shell/status-bar-operations";

export type ConnectionState = "connecting" | "connected" | "reconnecting";
export type SubmittingPrompt = { action: string; correlationId: string } | null;

type HubStoreState = {
  connection: ConnectionState;
  error: string | null;
  /** Wall-clock ms of the last frame received on the runtime stream (shell, notice, or heartbeat). */
  lastStreamFrameAt: number | null;
  /** The backend's slow operations, as the last runtime frame carried them. */
  operations: BackendOperation[];
  runtimePolicyBusy: boolean;
  submittingPrompt: SubmittingPrompt;
};

/** The fields the runtime stream updates on every frame. */
export type BackendOperationsPatch = Partial<Pick<HubStoreState, "connection" | "error" | "lastStreamFrameAt" | "operations">>;

type HubStore = HubStoreState & {
  resetHubStore: () => void;
  setHubState: (patch: Partial<HubStoreState>) => void;
};

const initialHubStoreState: HubStoreState = {
  connection: "connecting",
  error: null,
  lastStreamFrameAt: null,
  operations: [],
  runtimePolicyBusy: false,
  submittingPrompt: null,
};

export const useHubStore = create<HubStore>((set) => ({
  ...initialHubStoreState,
  resetHubStore: () => set(initialHubStoreState),
  setHubState: (patch) => set((state) => ({ ...state, ...patch })),
}));
