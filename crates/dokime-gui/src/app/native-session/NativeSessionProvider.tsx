// Dokime — SPDX-License-Identifier: Apache-2.0

import { QueryClientProvider } from "@tanstack/react-query";
import { type ReactNode, useCallback, useEffect, useState, useSyncExternalStore } from "react";
import { NotificationMessage } from "@/components/NotificationMessage";
import { Button } from "@/components/ui/button";
import { releaseNativeMedia } from "@/lib/api/native-delivery";
import { watchRemoteImports } from "@/lib/desktop/remote-import";
import { watchStopping } from "@/lib/desktop/stopping";
import { type AppearanceResource, loadAppearance } from "@/lib/rpc/appearance";
import { Appearance, Connection, Controller, useNativeSessionController } from "./context";
import { connectionError } from "./errors";
import { type ConnectedSession, NativeSession } from "./session";

/** One native owner and fresh backend query/resource scope per generation. */
export function NativeSessionProvider({
  children,
  disconnected,
  owner: supplied,
}: {
  children: ReactNode;
  disconnected: ReactNode;
  owner?: NativeSession;
}) {
  const [owner] = useState(() => supplied ?? new NativeSession());
  const state = useSyncExternalStore(owner.subscribe, owner.snapshot);
  useEffect(() => watchStopping(), []);
  useEffect(() => watchRemoteImports(owner), [owner]);
  useEffect(
    () => () => {
      void owner.disconnect();
    },
    [owner],
  );
  return (
    <Controller.Provider value={owner}>
      {state.phase === "connected" ? (
        <Connected key={state.session.snapshot.connection.generation} session={state.session}>
          {children}
        </Connected>
      ) : (
        disconnected
      )}
    </Controller.Provider>
  );
}
function Connected({ session, children }: { session: ConnectedSession; children: ReactNode }) {
  const owner = useNativeSessionController();
  const [attempt, setAttempt] = useState(0);
  const [appearance, setAppearance] = useState<AppearanceResource | null>(null);
  const [error, setError] = useState<unknown>();
  const refreshAppearance = useCallback(() => setAttempt((value) => value + 1), []);
  // `Connected` is keyed by generation in `NativeSessionProvider`, so this
  // effect's cleanup fires on exactly the two events that must release the
  // media preview cache: a generation change (remount under a new key) and
  // a disconnect (this subtree unmounts in favor of `disconnected`).
  useEffect(() => {
    const generation = session.identity.generation;
    return () => {
      void releaseNativeMedia(generation);
    };
  }, [session.identity.generation]);
  // biome-ignore lint/correctness/useExhaustiveDependencies: A user retry starts another acquisition in the same connection generation.
  useEffect(() => {
    setError(undefined);
    let active = true;
    let release: (() => void) | undefined;
    void loadAppearance(session.identity)
      .catch((error: unknown) => {
        if (active && session.identity.isCurrent() && needsAppearanceRefresh(error)) return loadAppearance(session.identity);
        throw error;
      })
      .then((resource) => {
        if (!active || !session.identity.isCurrent()) {
          resource.dispose();
          return;
        }
        release = session.own(() => resource.dispose());
        setAppearance(resource);
      })
      .catch((error: unknown) => {
        if (active && session.identity.isCurrent()) setError(error);
      });
    return () => {
      active = false;
      release?.();
    };
  }, [session, attempt]);
  return (
    <Connection.Provider value={session}>
      <QueryClientProvider client={session.queryClient}>
        <Appearance.Provider value={{ resource: appearance, refresh: refreshAppearance }}>
          {error ? (
            <main className="mx-auto max-w-xl space-y-4 p-8">
              <h1 className="font-heading text-2xl">Appearance Unavailable</h1>
              <NotificationMessage>{connectionError(error)}</NotificationMessage>
              <div className="flex gap-3">
                <Button onClick={() => setAttempt((value) => value + 1)}>Retry</Button>
                <Button variant="outline" onClick={() => void owner.disconnect()}>
                  Disconnect
                </Button>
              </div>
            </main>
          ) : (
            children
          )}
        </Appearance.Provider>
      </QueryClientProvider>
    </Connection.Provider>
  );
}
function needsAppearanceRefresh(error: unknown): boolean {
  if (!error || typeof error !== "object" || !("kind" in error) || error.kind !== "grpc" || !("detail" in error)) return false;
  const detail = error.detail;
  return !!detail && typeof detail === "object" && "code" in detail && (detail.code === 5 || detail.code === 9);
}
