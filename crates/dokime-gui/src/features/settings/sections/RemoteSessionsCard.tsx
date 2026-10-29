/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Recent Remote Sessions Of This Host
 */

import { useQuery } from "@tanstack/react-query";

import { RefreshCw } from "lucide-react";
import { useState } from "react";

import type { ConnectedSession } from "@/app/native-session/session";
import { DataTable } from "@/components/data-table/DataTable";
import { type DataTableColumns, dataTableColumns } from "@/components/data-table/data-table-model";
import { NotificationMessage } from "@/components/NotificationMessage";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { SettingsCard } from "@/features/settings/SettingsCard";
import {
  currentCursor,
  firstPage,
  newerPage,
  olderPage,
  type RemoteSessionRow,
  SESSION_PAGE_SIZE,
  sessionRow,
} from "@/features/settings/sections/remote-sessions";
import * as rpc from "@/lib/rpc/generated/client";

const column = dataTableColumns<RemoteSessionRow>();
const COLUMNS: DataTableColumns<RemoteSessionRow> = column.columns([
  column.accessor("peer", {
    cell: ({ getValue }) => <span className="font-mono text-xs">{getValue()}</span>,
    header: "Peer",
    meta: { label: "Peer", truncate: true },
  }),
  column.accessor("principal", { header: "Principal", meta: { label: "Principal", width: "w-28" } }),
  column.accessor("connected", { header: "Connected", meta: { label: "Connected", width: "w-44" } }),
  column.accessor("lastSeen", { header: "Last Seen", meta: { label: "Last Seen", width: "w-44" } }),
  column.accessor("duration", {
    cell: ({ row }) => (row.original.open ? `${row.original.duration}, open` : row.original.duration),
    header: "Duration",
    meta: { label: "Duration", width: "w-32" },
  }),
  column.accessor("calls", { header: "Calls", meta: { align: "end", label: "Calls", width: "w-32" } }),
  column.accessor("outcome", {
    cell: ({ row }) => (
      <span className="flex min-w-0 items-center gap-2" title={row.original.rejection}>
        <Badge variant={row.original.outcome === "Admitted" ? "success" : row.original.outcome === "Rejected" ? "destructive" : "outline"}>
          {row.original.outcome}
        </Badge>
        {row.original.rejection ? <span className="truncate text-xs text-muted-foreground">{row.original.rejection}</span> : null}
      </span>
    ),
    header: "Outcome",
    meta: { label: "Outcome", width: "w-56" },
  }),
]);

/** Who reached this host remotely, newest first, with mount, focus and manual refresh. */
export function RemoteSessionsCard({ session }: { session: ConnectedSession }) {
  const [pager, setPager] = useState(firstPage);
  const cursor = currentCursor(pager);
  const query = useQuery({
    queryKey: ["settings", "remote-sessions", session.identity.generation, cursor ?? null],
    queryFn: () => rpc.remoteListSessions(session.identity, { cursor, limit: SESSION_PAGE_SIZE }),
    refetchOnWindowFocus: true,
    retry: false,
  });
  const rows = (query.data?.sessions ?? []).map((entry) => sessionRow(entry));
  const next = query.data?.nextCursor;
  const page = pager.cursors.length;
  return (
    <SettingsCard>
      <DataTable
        aria-label="Recent Remote Sessions"
        actions={
          <div className="flex flex-wrap items-center justify-end gap-2">
            <span className="mr-auto text-xs text-muted-foreground">Page {page}</span>
            <Button disabled={query.isFetching} onClick={() => void query.refetch()} size="sm" variant="outline">
              <RefreshCw data-icon="inline-start" />
              Refresh
            </Button>
            <Button disabled={page === 1} onClick={() => setPager(newerPage)} size="sm" variant="outline">
              Newer
            </Button>
            <Button disabled={!next} onClick={() => next && setPager(olderPage(pager, next))} size="sm" variant="outline">
              Older
            </Button>
          </div>
        }
        columns={COLUMNS}
        data={rows}
        description="Admitted and refused remote clients, newest first. A session ends after a minute without calls; the host keeps the last 500."
        emptyText={query.isPending ? "Loading Remote Sessions…" : "No Remote Sessions Yet"}
        getRowId={rowKey}
        title="Recent Remote Sessions"
      />
      {query.error ? (
        <NotificationMessage>{query.error instanceof Error ? query.error.message : "Remote sessions are unavailable."}</NotificationMessage>
      ) : null}
    </SettingsCard>
  );
}

function rowKey(row: RemoteSessionRow): string {
  return row.key;
}
