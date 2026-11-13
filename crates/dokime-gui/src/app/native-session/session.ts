// Dokime — SPDX-License-Identifier: Apache-2.0
import type { QueryClient } from "@tanstack/react-query";
import { createDokimeQueryClient } from "@/app/query-client";
import { commands } from "@/lib/desktop/commands";
import type { BackendSelection, DesktopSessionSnapshot } from "@/lib/desktop/session";
import { preferencesCommands } from "@/lib/desktop/settings";
import { toast } from "@/lib/notifications";
import * as rpc from "@/lib/host/client";
import type { SessionIdentity } from "@/lib/rpc/transport";

export interface SessionControls {
  select(selection: BackendSelection): Promise<DesktopSessionSnapshot>;
  disconnect(): Promise<DesktopSessionSnapshot>;
  /** Development only: delete the owned backend's storage and restart it. */
  resetLocalStorage(directory: string): Promise<DesktopSessionSnapshot>;
  backupResetLocalStorage(directory: string, archive: string): Promise<DesktopSessionSnapshot>;
}
export interface ConnectedSession {
  snapshot: DesktopSessionSnapshot;
  identity: SessionIdentity;
  queryClient: QueryClient;
  /** Register a stream/resource disposer with this generation's owner. */
  own(dispose: () => void): () => void;
}
export type SessionState =
  | { phase: "disconnected" }
  /** Explicit resets distinguish backed-up recovery from the development reset. */
  | { phase: "connecting"; reset: false | "reset" | "backup" }
  | { phase: "connected"; session: ConnectedSession }
  | { phase: "failed"; error: unknown };
const nativeControls: SessionControls = {
  select: async (selection) => {
    if (selection.kind !== "profile") return commands.desktopSessionSelect(selection);
    await preferencesCommands.desktopBackendUse(selection.identifier);
    return commands.desktopSessionStatus();
  },
  disconnect: () => commands.desktopSessionDisconnect(),
  resetLocalStorage: (directory) => commands.desktopResetLocalStorage(directory),
  backupResetLocalStorage: (directory, archive) => commands.desktopBackupResetLocalStorage(directory, archive),
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
    await this.connect(() => this.controls.select(selection), false);
  }
  /** Reconnects to the owned backend restarted on an empty data directory,
   * exactly as after any restart: a new generation and fresh queries. */
  async resetLocalStorage(directory: string): Promise<void> {
    await this.connect(() => this.controls.resetLocalStorage(directory), "reset");
  }
  async backupAndResetLocalStorage(directory: string, archive: string): Promise<void> {
    await this.connect(async () => {
      const snapshot = await this.controls.backupResetLocalStorage(directory, archive);
      toast.success("Local Storage Backed Up and Reset", { description: snapshot.archive ?? archive });
      return snapshot;
    }, "backup");
  }
  private async connect(start: () => Promise<DesktopSessionSnapshot>, reset: false | "reset" | "backup"): Promise<void> {
    const revision = this.invalidate();
    this.publish({ phase: "connecting", reset });
    try {
      const snapshot = await start();
      if (revision !== this.revision) return;
      const backend = snapshot.connection.backend;
      if (snapshot.connection.phase !== "connected" || !backend)
        throw snapshot.connection.error ?? new Error("Native connection did not negotiate a backend");
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
      this.release = () => {
        void queryClient.cancelQueries();
        queryClient.clear();
        for (const dispose of disposers) dispose();
        disposers.clear();
      };
      const [ready, cycles] = await Promise.all([
        rpc.statusGet(session.identity, {}),
        rpc.executionListCycles(session.identity, { limit: 1 }),
      ]);
      if (ready.status?.identity?.hostId !== backend.backendId || cycles.cursor?.epoch !== backend.epoch) {
        throw new Error("The bridge reached a different backend identity or epoch.");
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
  async refresh(): Promise<void> {
    await this.connect(() => commands.desktopSessionStatus(), false);
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
