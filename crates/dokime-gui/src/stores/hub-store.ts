/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { create } from "zustand";

/* === Local Imports === */

/* === Types === */

export type ConnectionState = "connecting" | "connected" | "reconnecting";
export type SubmittingPrompt = { action: string; correlationId: string } | null;

type HubStoreState = {
  connection: ConnectionState;
  cycleBundleBusy: boolean;
  cycleBundleError: string | null;
  error: string | null;
  /** Wall-clock ms of the last frame received on the runtime stream (shell, notice, or heartbeat). */
  lastStreamFrameAt: number | null;
  runtimePolicyBusy: boolean;
  submittingPrompt: SubmittingPrompt;
};

type HubStore = HubStoreState & {
  resetHubStore: () => void;
  setHubState: (patch: Partial<HubStoreState>) => void;
};

/* === Constants === */

const initialHubStoreState: HubStoreState = {
  connection: "connecting",
  cycleBundleBusy: false,
  cycleBundleError: null,
  error: null,
  lastStreamFrameAt: null,
  runtimePolicyBusy: false,
  submittingPrompt: null,
};

/* === Public Functions === */

export const useHubStore = create<HubStore>((set) => ({
  ...initialHubStoreState,
  resetHubStore: () => set(initialHubStoreState),
  setHubState: (patch) => set((state) => ({ ...state, ...patch })),
}));
