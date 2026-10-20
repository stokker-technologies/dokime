/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * The active cycle: shell state the Library adds cases to.
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
import { useCallback, useMemo } from "react";

// Local Imports
import { cycleRows, promptCountsByCycle } from "@/features/cycles-home/cycles-model";
import { useCyclesQuery, usePendingPromptsQuery } from "@/features/cycles-home/cycles-queries";
import { type OpenCycle, openCycles, resolveActiveCycle } from "@/features/library/active-cycle-model";
import { useUiStore } from "@/stores/ui-store";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

export type ActiveCycleState = {
  /** The active cycle, once the open cycles have confirmed it is still open. */
  cycle: OpenCycle | null;
  /** Every open cycle, in the server's newest-first order. */
  cycles: OpenCycle[];
  failed: boolean;
  loading: boolean;
  choose: (cycleId: string | null) => void;
  openPicker: () => void;
};

/* === Classes === */

/* === Public Functions === */

/**
 * The persisted active cycle, checked against the open cycles on every read.
 *
 * The store keeps the operator's choice across navigation and restarts; the
 * cycle list decides whether it still stands. A choice that has sealed or
 * disappeared resolves to no active cycle, so the shell falls back to the
 * picker instead of naming a cycle that can no longer receive cases. The id
 * itself stays stored: a cycle created a moment ago is not yet listed.
 */
export function useActiveCycle(): ActiveCycleState {
  const cyclesQuery = useCyclesQuery();
  const prompts = usePendingPromptsQuery();
  const storedId = useUiStore((state) => state.activeCycleId);
  const setUiState = useUiStore((state) => state.setUiState);
  const cycles = useMemo(
    () => openCycles(cycleRows(cyclesQuery.data ?? [], promptCountsByCycle(prompts.data ?? []))),
    [cyclesQuery.data, prompts.data],
  );
  const cycle = resolveActiveCycle(storedId, cycles);

  const choose = useCallback((cycleId: string | null) => setUiState({ activeCycleId: cycleId }), [setUiState]);
  const openPicker = useCallback(() => setUiState({ activeCyclePickerOpen: true }), [setUiState]);

  return { choose, cycle, cycles, failed: cyclesQuery.isError, loading: cyclesQuery.isPending, openPicker };
}

/* === Private Functions === */

/* === Entry Point === */
