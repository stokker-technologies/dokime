/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Content view models shared by native adapters and the original UI. */

/** Lifecycle of a registered execution file as one bounded read observed it. */
export type ExecutionFileLifecycle = "static" | "streaming" | "finalized" | "interrupted";

/** One bounded `Content.ReadExecutionFile` window, mapped from the wire. */
export type ExecutionFileReadWindow = {
  offset: number;
  observedSizeBytes: number;
  data: Uint8Array;
  state: ExecutionFileLifecycle;
};

/** The retained-content identity a follow resumes from at a nonzero offset. */
export type ExecutionFileIdentity = {
  store: string;
  epoch: string;
  fileId: string;
  executionId: string;
  relativePath: string;
};

/** One `Content.FollowExecutionFile` frame, mapped from the wire. */
export type ExecutionFileFollowEvent =
  | {
      kind: "metadata";
      identity: ExecutionFileIdentity;
      phase: "producing" | "finished" | "abandoned";
      availableLength: number;
      offset: number;
    }
  | { kind: "chunk"; offset: number; data: Uint8Array }
  | {
      kind: "terminal";
      phase: "finished" | "abandoned";
      length: number;
      sha256: string | null;
    };
