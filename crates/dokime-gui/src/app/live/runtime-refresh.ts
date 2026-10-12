/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Coalesce runtime-watch refreshes while preserving one trailing observation. */

/* === Types === */

export type RuntimeProjectionRefreshCoordinator = {
  request(options?: { ensureTrailing?: boolean }): Promise<void>;
  dispose(): void;
};

/* === Exports === */

export function createRuntimeProjectionRefreshCoordinator(
  refresh: () => Promise<void>,
): RuntimeProjectionRefreshCoordinator {
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
