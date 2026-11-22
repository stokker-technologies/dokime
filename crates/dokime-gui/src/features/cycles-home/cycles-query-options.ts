// Dokime — SPDX-License-Identifier: Apache-2.0
import { ArchiveScope, CycleOrderKey } from "@generated/host/model";
import { listNativeCycles, type NativeCycleSelection } from "../../lib/api/native-planning";
import { fetchNativePendingPrompts } from "../../lib/api/native-prompts";
import type { SessionIdentity } from "../../lib/rpc/transport";
import { workflowQueryKeys } from "../cycle/workflow-query-keys";

export type CyclesQuerySelection = {
  /* True lists only archived cycles, false only the others. False is the
     default so the status bar and Cycles home share one cached read. */
  archived?: boolean;
};

export const cyclesHomeQueryKeys = {
  all: ["cycles-home"] as const,
  cycles: (generation: string, selection: CyclesQuerySelection = {}) =>
    [...cyclesHomeQueryKeys.all, "cycles", generation, { archived: selection.archived ?? false }] as const,
};

/**
 * The selection and order every page of one read repeats.
 *
 * The server's default order is UPDATED_AT ascending, so newest-first is a
 * request the client has to make.
 */
function cycleListSelection(selection: CyclesQuerySelection): NativeCycleSelection {
  return {
    filter: {
      text: "",
      statuses: [],
      archive: selection.archived ? ArchiveScope.ARCHIVED : ArchiveScope.UNARCHIVED,
    },
    order: { key: CycleOrderKey.UPDATED_AT, descending: true },
  };
}

/** The Runtime Stream Invalidates Both Lists On Change. */
export function cyclesQueryOptions(identity: SessionIdentity, selection: CyclesQuerySelection = {}) {
  return {
    queryFn: () => listNativeCycles(identity, cycleListSelection(selection)),
    queryKey: cyclesHomeQueryKeys.cycles(identity.generation, selection),
    staleTime: Number.POSITIVE_INFINITY,
  };
}

export function pendingPromptsQueryOptions(identity: SessionIdentity) {
  return {
    queryFn: () => fetchNativePendingPrompts(identity),
    queryKey: [...workflowQueryKeys.prompts(), identity.generation],
    staleTime: Number.POSITIVE_INFINITY,
  };
}
