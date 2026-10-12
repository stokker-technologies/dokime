/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { z } from "zod";
import { create } from "zustand";
import { createJSONStorage, persist } from "zustand/middleware";

/* === Types === */

type SelectionStoreState = {
  followActive: boolean;
  plotWindowIndex: number;
  plotWindowOverridden: boolean;
  selectedCaseIds: Set<string>;
  selectedExecutionId: string | null;
  selectedQueueItemId: string | null;
};

type SelectionStore = SelectionStoreState & {
  resetSelectionStore: () => void;
  setSelectionState: (patch: Partial<SelectionStoreState>) => void;
};

/* === Constants === */

const DEFAULT_PLOT_WINDOW_INDEX = 2;

const persistedSelectionStateSchema = z.object({
  plotWindowIndex: z.number().int().min(0).max(7).catch(DEFAULT_PLOT_WINDOW_INDEX),
  plotWindowOverridden: z.boolean().catch(false),
});

const initialSelectionStoreState: SelectionStoreState = {
  followActive: true,
  plotWindowIndex: DEFAULT_PLOT_WINDOW_INDEX,
  plotWindowOverridden: false,
  selectedCaseIds: new Set(),
  selectedExecutionId: null,
  selectedQueueItemId: null,
};

/* === Public Functions === */

export const useSelectionStore = create<SelectionStore>()(
  persist(
    (set) => ({
      ...initialSelectionStoreState,
      resetSelectionStore: () => set(initialSelectionStoreState),
      setSelectionState: (patch) => set((state) => ({ ...state, ...patch })),
    }),
    {
      merge: (persisted, current) => {
        const parsed = persistedSelectionStateSchema.safeParse(
          (persisted as {
            plotWindowIndex?: unknown;
            plotWindowOverridden?: unknown;
          }) ?? {},
        );
        if (!parsed.success) {
          return current;
        }
        return {
          ...current,
          ...parsed.data,
        };
      },
      name: "dokime-test-hub.selection-ui",
      partialize: (state) => ({
        plotWindowIndex: state.plotWindowIndex,
        plotWindowOverridden: state.plotWindowOverridden,
      }),
      storage: createJSONStorage(() => window.localStorage),
    },
  ),
);
