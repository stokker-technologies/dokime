/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Coalesce runtime-watch refreshes while preserving one trailing observation. */

/* === Local Imports === */

import type { RuntimeStateFrame } from "@/lib/api/native-observation";

/* === Types === */

export type RuntimeProjectionRefreshCoordinator = {
  request(options?: { ensureTrailing?: boolean }): Promise<void>;
  dispose(): void;
};

/* === Exports === */

export function createRuntimeProjectionRefreshCoordinator(refresh: () => Promise<void>): RuntimeProjectionRefreshCoordinator {
  let inFlight: Promise<void> | undefined;
  let pending = false;
  let trailing = false;
  let disposed = false;

  async function drain(): Promise<void> {
    try {
      while (!disposed) {
        pending = false;
        await refresh();
        if (trailing) {
          trailing = false;
          pending = true;
        }
        if (!pending) return;
      }
    } finally {
      // Returning from the loop and clearing this in one continuation leaves
      // no handoff gap in which a request can observe a finishing promise.
      inFlight = undefined;
    }
  }

  return {
    request(options: { ensureTrailing?: boolean } = {}): Promise<void> {
      if (disposed) return Promise.resolve();
      pending = true;
      trailing ||= Boolean(options.ensureTrailing);
      // Install the promise before starting the drain so synchronous callers
      // share one refresh, while drain owns the full lifecycle cleanup.
      inFlight ??= Promise.resolve().then(drain);
      return inFlight;
    },
    dispose(): void {
      disposed = true;
      pending = false;
      trailing = false;
    },
  };
}

/**
 * Coalesce requests into at most one refresh per interval. A refresh runs one
 * interval after the first request it absorbs, so the last request always has
 * a refresh that starts after it. `now` runs the refresh at once and absorbs
 * the pending request.
 */
export function createThrottledRefresh(refresh: () => void, intervalMs: number): { request(): void; now(): void; dispose(): void } {
  let timer: ReturnType<typeof setTimeout> | undefined;
  let disposed = false;

  return {
    request(): void {
      if (disposed || timer !== undefined) return;
      timer = setTimeout(() => {
        timer = undefined;
        if (!disposed) refresh();
      }, intervalMs);
    },
    now(): void {
      if (disposed) return;
      if (timer !== undefined) clearTimeout(timer);
      timer = undefined;
      refresh();
    },
    dispose(): void {
      disposed = true;
      if (timer !== undefined) clearTimeout(timer);
      timer = undefined;
    },
  };
}

/**
 * The settled cycles a frame reports: those with nothing running or queued,
 * with their status, results and seal. Telemetry growth and runs finishing
 * inside a busy cycle leave it unchanged; a cycle that settles or seals changes it.
 */
export function settledOf(frame: Pick<RuntimeStateFrame, "overview">): string | null {
  const cycles = frame.overview?.cycleSummaries;
  if (!cycles) return null;
  return JSON.stringify(
    cycles
      .filter((cycle) => cycle.sealed || (cycle.runningCount === 0n && cycle.queuedCount === 0n))
      .map((cycle) => [cycle.cycleId, cycle.phase, cycle.status, cycle.result, cycle.sealed]),
    (_, value: unknown) => (typeof value === "bigint" ? value.toString() : value),
  );
}

/** Runtime Projections Follow Lifecycle Changes, Independently of Evidence Growth. */
export function runtimeStateOf(frame: Pick<RuntimeStateFrame, "overview">): string | null {
  const cursor = frame.overview?.observedCursor;
  return cursor ? JSON.stringify(cursor, (_, value: unknown) => (typeof value === "bigint" ? value.toString() : value)) : null;
}

/** Evidence Can Advance Without Changing Runtime State. */
export function createEvidenceRefresh(refresh: () => void, intervalMs: number) {
  const throttled = createThrottledRefresh(refresh, intervalMs);
  let observed: string | undefined;
  return {
    observe(frame: Pick<RuntimeStateFrame, "resolvedAt" | "opening">): void {
      const revision = JSON.stringify(frame.resolvedAt, (_, value: unknown) => (typeof value === "bigint" ? value.toString() : value));
      if (revision === observed && !frame.opening) return;
      observed = revision;
      throttled.request();
    },
    now: throttled.now,
    dispose: throttled.dispose,
  };
}
