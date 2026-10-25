// Dokime — SPDX-License-Identifier: Apache-2.0
import type { QueryClient } from "@tanstack/react-query";
import { createDokimeQueryClient } from "@/app/query-client";
import { commands } from "@/lib/desktop/commands";
import type { BackendChoice, BackendSelection, DesktopSessionSnapshot } from "@/lib/desktop/session";
import * as rpc from "@/lib/rpc/generated/client";
import type { SessionIdentity } from "@/lib/rpc/transport";

export interface SessionControls {
  select(selection: BackendSelection): Promise<DesktopSessionSnapshot>;
  disconnect(): Promise<DesktopSessionSnapshot>;
  status(): Promise<DesktopSessionSnapshot>;
  /** Development only: delete the owned backend's storage and restart it. */
  resetLocalStorage(directory: string): Promise<DesktopSessionSnapshot>;
  /** After a failed owned start: archive the storage, reset it and restart. */
  backupResetLocalStorage(archive: string): Promise<DesktopSessionSnapshot>;
}
export interface ConnectedSession {
  snapshot: DesktopSessionSnapshot;
  identity: SessionIdentity;
  queryClient: QueryClient;
  /** Register a stream/resource disposer with this generation's owner. */
  own(dispose: () => void): () => void;
}
type SessionStatus =
  | { phase: "disconnected" }
  /** `reset` marks a local storage reset: stop, delete, then start again;
   * `archive` names the backup a backup reset writes first. */
  | { phase: "connecting"; reset: boolean; archive: string | null }
  | { phase: "connected"; session: ConnectedSession; selectionError?: unknown }
  | { phase: "failed"; error: unknown };
export type SessionState = SessionStatus & { selection: BackendChoice | null };

const nativeControls: SessionControls = {
  select: (selection) => commands.desktopSessionSelect(selection),
  disconnect: () => commands.desktopSessionDisconnect(),
  status: () => commands.desktopSessionStatus(),
  resetLocalStorage: (directory) => commands.desktopResetLocalStorage(directory),
  backupResetLocalStorage: (archive) => commands.desktopBackupResetLocalStorage(archive),
};

/** One owner per desktop window. Inputs containing secrets are never retained. */
export class NativeSession {
  private revision = 0;
  private selection: BackendChoice | null = null;
  private state: SessionState = { phase: "disconnected", selection: null };
  private listeners = new Set<() => void>();
  private release: (() => void) | undefined;
  constructor(private controls: SessionControls = nativeControls) {}
  snapshot = () => this.state;
  subscribe = (listener: () => void) => {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  };
  private publish(state: SessionStatus) {
    this.state = { ...state, selection: this.selection };
    for (const listener of this.listeners) listener();
  }
  private dispose() {
    const release = this.release;
    this.release = undefined;
    release?.();
  }
  private invalidate() {
    this.revision++;
    this.dispose();
    return this.revision;
  }
  async select(selection: BackendSelection): Promise<void> {
    this.selection = selection.kind === "attach" ? null : selection;
    await this.connect(() => this.controls.select(selection), false, null, true);
  }
  /** Reconnects to the owned backend restarted on an empty data directory,
   * exactly as after any restart: a new generation and fresh queries. */
  async resetLocalStorage(directory: string): Promise<void> {
    await this.connect(() => this.controls.resetLocalStorage(directory), true);
  }
  /** Archives the owned backend's storage to `archive`, then resets it and
   * reconnects as `resetLocalStorage` does. */
  async backupResetLocalStorage(archive: string): Promise<void> {
    await this.connect(() => this.controls.backupResetLocalStorage(archive), true, archive);
  }
  private async connect(
    start: () => Promise<DesktopSessionSnapshot>,
    reset: boolean,
    archive: string | null = null,
    recoverSelection = false,
  ): Promise<void> {
    const revision = this.invalidate();
    this.publish({ phase: "connecting", reset, archive });
    try {
      const snapshot = await start();
      if (revision !== this.revision) return;
      await this.establish(snapshot, revision);
    } catch (error) {
      if (revision === this.revision) {
        this.dispose();
        if (recoverSelection) {
          const actual = await this.controls.status().catch(() => null);
          if (revision !== this.revision) return;
          if (actual) this.selection = actual.selection;
          if (actual?.connection.phase === "connected" && actual.connection.backend) {
            try {
              await this.establish(actual, revision);
              if (revision !== this.revision) return;
              if (this.state.phase === "connected") {
                this.publish({ phase: "connected", session: this.state.session, selectionError: error });
                return;
              }
            } catch {
              this.dispose();
            }
          }
        }
        if (revision === this.revision) this.publish({ phase: "failed", error });
      }
    }
  }
  private async establish(snapshot: DesktopSessionSnapshot, revision: number): Promise<void> {
    this.selection = snapshot.selection;
    const backend = snapshot.connection.backend;
    if (snapshot.connection.phase !== "connected" || !backend) throw new Error("Native connection did not negotiate a backend");
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
    const identity = await rpc.backendGetIdentity(session.identity, {});
    if (identity.backendId !== backend.backendId || identity.epoch !== backend.epoch) {
      throw new Error("The bridge reached a different backend identity or epoch.");
    }
    if (revision !== this.revision) return;
    this.publish({ phase: "connected", session });
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
