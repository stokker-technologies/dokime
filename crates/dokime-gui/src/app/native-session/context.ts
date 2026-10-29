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

/* === Imports === */

// Standard Library Imports

// Third Party Imports
import { createContext, useContext } from "react";

// Local Imports
import type { AppearanceResource } from "@/lib/rpc/appearance";
import type { ConnectedSession, NativeSession } from "./session";

/* === Re-Exports === */

/* === Constants === */
export const Controller = createContext<NativeSession | null>(null);
export const Connection = createContext<ConnectedSession | null>(null);
export const Appearance = createContext<NativeAppearanceState | null>(null);

/* === Types === */
type NativeAppearanceState = {
  resource: AppearanceResource | null;
  refresh(): void;
};

/* === Classes === */

/* === Public Functions === */
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
/* === Private Functions === */

/* === Entry Point === */
