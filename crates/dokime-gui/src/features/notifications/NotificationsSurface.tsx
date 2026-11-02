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
import { CircleAlert, CircleCheck, Info, TriangleAlert } from "lucide-react";
import { useEffect, useState } from "react";
import { Link } from "@tanstack/react-router";
import { relativeTime } from "@/components/document/document-overview-model";
import { NotificationMessage } from "@/components/NotificationMessage";
import { FailureConsequences } from "@/components/FailureConsequences";
// Local Imports
import { Button } from "@/components/ui/button";
import {
  clearNotifications,
  markNotificationsRead,
  type NotificationCursor,
  type NotificationPage,
  type NotificationRecord,
  notificationPage,
  openNotificationDetails,
  useNotificationSummary,
} from "@/lib/notifications";
import { PageShell } from "@/shell/PageShell";

/* === Re-Exports === */

/* === Constants === */

const tones = {
  error: { icon: CircleAlert, label: "Error", className: "bg-destructive/10 text-destructive" },
  warning: { icon: TriangleAlert, label: "Warning", className: "bg-warning/10 text-warning" },
  info: { icon: Info, label: "Info", className: "bg-info/10 text-info" },
  success: { icon: CircleCheck, label: "Success", className: "bg-success/10 text-success" },
};

/* === Types === */

type Selection = { cursor?: NotificationCursor; direction: "older" | "newer" };

/* === Components === */

export function NotificationsSurface() {
  const summary = useNotificationSummary();
  const [selection, setSelection] = useState<Selection>({ direction: "older" });
  const [page, setPage] = useState<NotificationPage>({ records: [], older: false, newer: false });
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(true);
  const [now, setNow] = useState(Date.now);
  useEffect(() => {
    const timer = setInterval(() => setNow(Date.now()), 30_000);
    return () => clearInterval(timer);
  }, []);
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
      <div className="flex flex-wrap items-center justify-between gap-4">
        <span className="text-sm text-muted-foreground">{summary.count} Notifications</span>
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
          <NotificationHistoryEntry key={record.id} record={record} now={now} />
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

export function NotificationHistoryEntry({ record, now }: { record: NotificationRecord; now: number }) {
  const tone = tones[record.tone];
  const Icon = tone.icon;
  return (
    <li className="relative flex gap-3 px-4 py-4" data-tone={record.tone}>
      <span className={`flex size-8 shrink-0 items-center justify-center rounded-full ${tone.className}`}>
        <Icon className="size-4" aria-hidden="true" />
        <span className="sr-only">{tone.label}</span>
      </span>
      <div className="grid min-w-0 flex-1 gap-2">
        <div className="flex items-baseline justify-between gap-4">
          <button
            type="button"
            aria-haspopup="dialog"
            title={new Date(record.at).toLocaleString()}
            className="min-w-0 rounded text-left text-sm font-semibold text-foreground outline-none after:absolute after:inset-0 after:content-[''] focus-visible:ring-2 focus-visible:ring-ring [overflow-wrap:anywhere]"
            onClick={() => openNotificationDetails(record)}
          >
            {record.title}
          </button>
          <time
            className="shrink-0 whitespace-nowrap text-xs text-muted-foreground"
            dateTime={new Date(record.at).toISOString()}
            title={new Date(record.at).toLocaleString()}
          >
            {relativeTime(new Date(record.at).toISOString(), now)}
          </time>
        </div>
        {record.description?.trim() && (
          <p className="whitespace-pre-wrap text-sm leading-relaxed text-muted-foreground [overflow-wrap:anywhere]">{record.description}</p>
        )}
        {record.source.trim() && record.source.trim() !== "/" && (
          <span className="text-xs text-muted-foreground [overflow-wrap:anywhere]">{record.source}</span>
        )}
        <div className="relative z-10">
          <FailureConsequences items={record.consequences ?? []} />
        </div>
        {record.missingSecret ? (
          <Link
            className="relative z-10 text-sm underline"
            to="/settings/$section"
            params={{ section: "secrets" }}
            hash={`add-secret=${record.missingSecret}`}
          >
            Set Secret
          </Link>
        ) : record.pluginSettings ? (
          <Link className="relative z-10 text-sm underline" to="/settings/plugins/$pluginId" params={{ pluginId: record.pluginSettings }}>
            Open Plugin Settings
          </Link>
        ) : null}
      </div>
    </li>
  );
}

/* === Hooks === */

/* === Public Functions === */

/* === Private Functions === */
