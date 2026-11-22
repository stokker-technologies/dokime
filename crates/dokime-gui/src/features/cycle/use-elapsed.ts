// Dokime — SPDX-License-Identifier: Apache-2.0
import { useEffect, useState } from "react";
import { observeElapsedSeconds } from "./elapsed";

export function useElapsedSeconds(startedAt: string | null | undefined, running: boolean): number | null {
  const [elapsed, setElapsed] = useState<number | null>(null);
  useEffect(() => observeElapsedSeconds(startedAt, running, setElapsed), [running, startedAt]);
  return elapsed;
}
