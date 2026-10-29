/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Read And Clear Notification History A Bounded Page At A Time.
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
import { useEffect, useState } from "react";
import { NotificationMessage } from "@/components/NotificationMessage";
// Local Imports
import { Button } from "@/components/ui/button";
import {
  clearNotifications,
  markNotificationsRead,
  type NotificationCursor,
  type NotificationPage,
  notificationPage,
  useNotificationSummary,
} from "@/lib/notifications";
import { PageShell } from "@/shell/PageShell";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

type Selection = { cursor?: NotificationCursor; direction: "older" | "newer" };

/* === Components === */

export function NotificationsSurface() {
  const summary = useNotificationSummary();
  const [selection, setSelection] = useState<Selection>({ direction: "older" });
  const [page, setPage] = useState<NotificationPage>({ records: [], older: false, newer: false });
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(true);
  useEffect(() => {
    if (!summary.revision || !summary.unread) return;
    void markNotificationsRead().catch((failure: unknown) => setError(String(failure)));
  }, [summary.revision, summary.unread]);
  useEffect(() => {
    if (!summary.revision) return;
    let active = true;
    setLoading(true);
    void notificationPage(selection.cursor, selection.direction)
      .then((value) => {
        if (active) {
          setPage(value);
          setError("");
        }
      })
      .catch((failure: unknown) => {
        if (active) setError(String(failure));
      })
      .finally(() => {
        if (active) setLoading(false);
      });
    return () => {
      active = false;
    };
  }, [selection, summary.revision]);
  return (
    <PageShell trail={["Notifications"]}>
      <div className="flex items-center justify-between gap-4">
        <span className="text-sm text-muted-foreground">{summary.count} Notifications</span>
        <Button variant="outline" disabled={loading && !summary.error} onClick={() => setSelection((current) => ({ ...current }))}>
          Refresh
        </Button>
        <Button
          variant="outline"
          disabled={!summary.count || loading}
          onClick={() => {
            void clearNotifications()
              .then(() => setSelection({ direction: "older" }))
              .catch((failure: unknown) => setError(String(failure)));
          }}
        >
          Clear Notifications
        </Button>
      </div>
      {error && <NotificationMessage source="Notifications">{error}</NotificationMessage>}
      <ol className="divide-y rounded-lg border bg-card" aria-busy={loading}>
        {page.records.map((record) => (
          <li key={record.id} className="grid gap-1 px-4 py-3" data-tone={record.tone}>
            <div className="flex flex-wrap items-baseline justify-between gap-2">
              <strong className="text-sm">{record.title}</strong>
              <time className="text-xs text-muted-foreground" dateTime={new Date(record.at).toISOString()}>
                {new Date(record.at).toLocaleString()}
              </time>
            </div>
            <span className="text-xs text-muted-foreground">{record.source}</span>
            {record.description && <p className="break-words whitespace-pre-wrap text-sm">{record.description}</p>}
          </li>
        ))}
        {!page.records.length && (
          <li className="px-4 py-8 text-center text-muted-foreground">{loading ? "Loading Notifications" : "No Notifications"}</li>
        )}
      </ol>
      <div className="flex justify-end gap-2">
        <Button
          variant="outline"
          disabled={!page.newer || loading}
          onClick={() => setSelection({ cursor: page.records[0], direction: "newer" })}
        >
          Newer
        </Button>
        <Button
          variant="outline"
          disabled={!page.older || loading}
          onClick={() => setSelection({ cursor: page.records[page.records.length - 1], direction: "older" })}
        >
          Older
        </Button>
      </div>
    </PageShell>
  );
}

/* === Hooks === */

/* === Public Functions === */

/* === Private Functions === */
