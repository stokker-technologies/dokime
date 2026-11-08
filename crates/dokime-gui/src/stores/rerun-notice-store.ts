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

/* === Types === */

/** A source case the installed plugin no longer declares, or no longer declares fully. */
export type RerunNotice = { caseId: string; suiteId: string; reason: string };

/** What a Rerun Cycle reply reported about the cycle it created. */
export type RerunNotices = { skipped: readonly RerunNotice[]; droppedParameters: readonly RerunNotice[] };

type RerunNoticeStore = {
  /**
   * The notices of each cycle this session created with Rerun Cycle, by the
   * new cycle's id. Session state only: the reply is the only statement of
   * them, so a later session shows none.
   */
  notices: Readonly<Record<string, RerunNotices>>;
  setRerunNotices: (cycleId: string, notices: RerunNotices) => void;
};

/* === Stores === */

export const useRerunNoticeStore = create<RerunNoticeStore>()((set) => ({
  notices: {},
  setRerunNotices: (cycleId, notices) => set((state) => ({ notices: { ...state.notices, [cycleId]: notices } })),
}));
