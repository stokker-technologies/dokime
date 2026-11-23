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

/* === Types === */

type SelectionStoreState = {
  followActive: boolean;
  selectedCaseIds: Set<string>;
  selectedExecutionId: string | null;
  selectedQueueItemId: string | null;
};

type SelectionStore = SelectionStoreState & {
  resetSelectionStore: () => void;
  setSelectionState: (patch: Partial<SelectionStoreState>) => void;
};

/* === Constants === */

const initialSelectionStoreState: SelectionStoreState = {
  followActive: true,
  selectedCaseIds: new Set(),
  selectedExecutionId: null,
  selectedQueueItemId: null,
};

/* === Public Functions === */

export const selectionStore = createStore<SelectionStore>()((set) => ({
  ...initialSelectionStoreState,
  resetSelectionStore: () => set(initialSelectionStoreState),
  setSelectionState: (patch) => set((state) => ({ ...state, ...patch })),
}));
