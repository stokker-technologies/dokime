/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Remote Session Rows For The Recent Sessions Table
 */

import type { Wire } from "@/lib/rpc/canonical";
import type * as rpc from "@/lib/rpc/generated/client";

const GUI_REMOTE_SESSION_PAGE_ITEMS = 20;

export const SESSION_PAGE_SIZE = GUI_REMOTE_SESSION_PAGE_ITEMS;

export type RemoteSessionPage = Wire<Awaited<ReturnType<typeof rpc.remoteListSessions>>>;
export type RemoteSession = RemoteSessionPage["sessions"][number];

export type RemoteSessionRow = {
  key: string;
  peer: string;
  principal: string;
  connected: string;
  lastSeen: string;
  duration: string;
  calls: string;
  outcome: "Admitted" | "Rejected" | "Unknown";
  /** The host's reason for a rejected session. */
  rejection?: string;
  open: boolean;
};

/** Cursors of the pages before the current one; the last entry is the current page's cursor. */
export type SessionPager = { cursors: (string | undefined)[] };

export function sessionRow(session: RemoteSession, format: (millis: number) => string = formatTime): RemoteSessionRow {
  const connected = Number(session.connectedAt);
  const lastSeen = Number(session.lastSeenAt);
  const ended = session.disconnectedAt === undefined ? undefined : Number(session.disconnectedAt);
  const variant = session.outcome?.HarmosVariant.case;
  return {
    key: session.id,
    peer: session.peerAddress,
    principal: session.principal === "access-key" ? "Access Key" : "Anonymous",
    connected: format(connected),
    lastSeen: format(lastSeen),
    duration: formatDuration((ended ?? lastSeen) - connected),
    calls: session.calls.toString(),
    outcome: variant === "Admitted" || variant === "Rejected" ? variant : "Unknown",
    rejection: session.rejection,
    open: ended === undefined,
  };
}

export function formatDuration(millis: number): string {
  const seconds = Math.max(0, Math.round(millis / 1000));
  if (seconds < 60) return `${seconds} s`;
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return `${minutes} min ${seconds % 60} s`;
  return `${Math.floor(minutes / 60)} h ${String(minutes % 60).padStart(2, "0")} min`;
}

export const firstPage: SessionPager = { cursors: [undefined] };

export function currentCursor(pager: SessionPager): string | undefined {
  return pager.cursors.at(-1);
}

export function olderPage(pager: SessionPager, next: string): SessionPager {
  return { cursors: [...pager.cursors, next] };
}

export function newerPage(pager: SessionPager): SessionPager {
  return pager.cursors.length > 1 ? { cursors: pager.cursors.slice(0, -1) } : pager;
}

function formatTime(millis: number): string {
  return new Date(millis).toLocaleString(undefined, { dateStyle: "short", timeStyle: "medium" });
}
