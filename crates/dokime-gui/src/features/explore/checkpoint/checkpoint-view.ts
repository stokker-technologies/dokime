// Dokime — SPDX-License-Identifier: Apache-2.0
import type { QueryClient } from "@tanstack/svelte-query";
import type { NativeCheckpointSession } from "@/lib/api/native-checkpoints";
import type { ExecutionCheckpoint } from "@/lib/types/browse";

export type CheckpointProps = {
  checkpoint: ExecutionCheckpoint;
  connection: { session: NativeCheckpointSession; queryClient: QueryClient };
};

export function formatStructured(value: unknown): string {
  try {
    return JSON.stringify(value, (_key, item: unknown) => (typeof item === "bigint" ? item.toString() : item), 2) ?? "";
  } catch {
    return String(value);
  }
}
