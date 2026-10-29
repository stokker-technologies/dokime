/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Connection Context Values And Hooks, Independent Of The Desktop Owner.
 */

import { createContext, useContext } from "react";

import type { AppearanceResource } from "@/lib/rpc/appearance";
import type { ConnectedSession, NativeSession } from "./session";

export const Controller = createContext<NativeSession | null>(null);
export const Connection = createContext<ConnectedSession | null>(null);
export const Appearance = createContext<NativeAppearanceState | null>(null);

type NativeAppearanceState = {
  resource: AppearanceResource | null;
  refresh(): void;
};

export function useNativeSessionController() {
  const value = useContext(Controller);
  if (!value) throw new Error("Native session provider is required");
  return value;
}
export function useNativeSession() {
  const value = useContext(Connection);
  if (!value) throw new Error("A connected native backend is required");
  return value;
}

/** Optional form for shell providers that also render while disconnected. */
export function useNativeSessionOptional() {
  return useContext(Connection);
}
export function useNativeAppearance() {
  return useContext(Appearance)?.resource ?? null;
}
