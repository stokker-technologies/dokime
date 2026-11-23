// Dokime — SPDX-License-Identifier: Apache-2.0
import { getContext, setContext } from "svelte";
import type { NotificationFailure } from "../lib/notification-details";

type ErrorRecovery = (failure: NotificationFailure) => (() => Promise<void>) | undefined;
export const recoveryContext = Symbol("error-recovery");

export function provideErrorRecovery(resolve: ErrorRecovery) {
  setContext(recoveryContext, resolve);
}

export function getErrorRecovery(): ErrorRecovery | undefined {
  return getContext(recoveryContext);
}
