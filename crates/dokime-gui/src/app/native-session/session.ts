// Dokime — SPDX-License-Identifier: Apache-2.0
import type { QueryClient } from "@tanstack/react-query";
import { createDokimeQueryClient } from "@/app/query-client";
import { commands } from "@/lib/desktop/commands";
import type { BrowserAccess } from "@/lib/desktop/connection";
import { bindConnection } from "@/lib/rpc/grpc-web";
import * as rpc from "@/lib/rpc/generated/client";
import type { BackendSelection, DesktopSessionSnapshot } from "@/lib/desktop/session";
import type { SessionIdentity } from "@/lib/rpc/transport";

export interface SessionControls {
  select(selection: BackendSelection): Promise<DesktopSessionSnapshot>;
  disconnect(): Promise<DesktopSessionSnapshot>;
  transport(generation: string): Promise<BrowserAccess>;
}
export interface ConnectedSession {
  snapshot: DesktopSessionSnapshot;
  identity: SessionIdentity;
  queryClient: QueryClient;
  /** Register a stream/resource disposer with this generation's owner. */
  own(dispose: () => void): () => void;
}
export type SessionState =
  | { phase: "disconnected" | "connecting" }
  | { phase: "connected"; session: ConnectedSession }
  | { phase: "failed"; error: unknown };
const nativeControls: SessionControls = {
  select: (selection) => commands.desktopSessionSelect(selection),
  disconnect: () => commands.desktopSessionDisconnect(),
  transport: (generation) => commands.desktopSessionTransport(generation),
};

/** One owner per desktop window. Inputs containing secrets are never retained. */
export class NativeSession {
  private revision = 0;
  private state: SessionState = { phase: "disconnected" };
  private listeners = new Set<() => void>();
  private release: (() => void) | undefined;
  constructor(private controls: SessionControls = nativeControls) {}
  snapshot = () => this.state;
  subscribe = (listener: () => void) => {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  };
  private publish(state: SessionState) {
    this.state = state;
    for (const listener of this.listeners) listener();
  }
  private invalidate() {
    this.revision++;
    this.release?.();
    this.release = undefined;
    return this.revision;
  }
  async select(selection: BackendSelection): Promise<void> {
    const revision = this.invalidate();
    this.publish({ phase: "connecting" });
    try {
      const snapshot = await this.controls.select(selection);
      if (revision !== this.revision) return;
      if (snapshot.connection.phase !== "connected" || !snapshot.connection.backend)
        throw new Error("Native connection did not negotiate a backend");
      const access = await this.controls.transport(snapshot.connection.generation);
      if (revision !== this.revision) {
        access.authorization = null;
        return;
      }
      if (
        access.backendId !== snapshot.connection.backend.backendId ||
        access.epoch !== snapshot.connection.backend.epoch
      ) {
        access.authorization = null;
        throw new Error("Backend identity changed during desktop connection.");
      }
      const queryClient = createDokimeQueryClient();
      const disposers = new Set<() => void>();
      const session: ConnectedSession = {
        snapshot,
        queryClient,
        identity: {
          generation: snapshot.connection.generation,
          isCurrent: () => revision === this.revision,
        },
        own: (dispose) => {
          if (revision !== this.revision) {
            dispose();
            return () => {};
          }
          disposers.add(dispose);
          return () => {
            if (disposers.delete(dispose)) dispose();
          };
        },
      };
      const releaseTransport = bindConnection(session.identity, access);
      this.release = () => {
        releaseTransport();
        void queryClient.cancelQueries();
        queryClient.clear();
        for (const dispose of disposers) dispose();
        disposers.clear();
      };
      const identity = await rpc.backendGetIdentity(session.identity, {});
      if (identity.backendId !== access.backendId || identity.epoch !== access.epoch) {
        throw new Error("The browser connected to a different backend identity or epoch.");
      }
      if (revision !== this.revision) return;
      this.publish({ phase: "connected", session });
    } catch (error) {
      if (revision === this.revision) {
        this.release?.();
        this.release = undefined;
        this.publish({ phase: "failed", error });
      }
    }
  }
  async disconnect(): Promise<void> {
    const revision = this.invalidate();
    this.publish({ phase: "disconnected" });
    try {
      await this.controls.disconnect();
    } catch (error) {
      if (revision === this.revision) this.publish({ phase: "failed", error });
    }
  }
}
