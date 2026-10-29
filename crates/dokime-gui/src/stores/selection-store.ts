/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import { create } from "zustand";

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

const initialSelectionStoreState: SelectionStoreState = {
  followActive: true,
  selectedCaseIds: new Set(),
  selectedExecutionId: null,
  selectedQueueItemId: null,
};

export const useSelectionStore = create<SelectionStore>()((set) => ({
  ...initialSelectionStoreState,
  resetSelectionStore: () => set(initialSelectionStoreState),
  setSelectionState: (patch) => set((state) => ({ ...state, ...patch })),
}));
