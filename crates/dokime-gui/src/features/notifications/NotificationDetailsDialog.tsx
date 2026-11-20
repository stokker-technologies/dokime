/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Inspect and Copy Complete Notification Context
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
import { useRef, useState } from "react";

// Local Imports
import { ErrorCard } from "@/components/ErrorCard";
import { relativeTime } from "@/components/document/document-overview-model";
import { FailureConsequences } from "@/components/FailureConsequences";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { failureOf, notificationDetailsText } from "@/lib/notification-details";
import { openNotificationDetails, toast, useNotificationDetails } from "@/lib/notifications";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/* === Components === */

export function NotificationDetailsDialog() {
  const record = useNotificationDetails();
  const [copyStatus, setCopyStatus] = useState("");
  const opener = useRef<HTMLElement | null>(null);
  const failure = record?.failure;
  const timestamp = record ? new Date(record.at).toISOString() : "";
  const relative = record ? relativeTime(timestamp) : "";
  return (
    <Dialog
      open={Boolean(record)}
      onOpenChange={(open) => {
        if (!open) openNotificationDetails(null);
      }}
    >
      <DialogContent
        className="z-[110] max-h-[85dvh] grid-rows-[auto_minmax(0,1fr)_auto] sm:max-w-2xl"
        onOpenAutoFocus={() => {
          opener.current = document.activeElement instanceof HTMLElement ? document.activeElement : null;
          setCopyStatus("");
        }}
        onCloseAutoFocus={(event) => {
          event.preventDefault();
          if (opener.current?.isConnected) opener.current.focus();
        }}
      >
        <DialogHeader className="pr-8">
          <DialogTitle>{record?.title}</DialogTitle>
          <DialogDescription>Notification Details</DialogDescription>
        </DialogHeader>
        {record && (
          <div className="grid min-h-0 gap-4 overflow-y-auto whitespace-pre-wrap [overflow-wrap:anywhere]">
            {!failure && record.description && <p className="select-text">{record.description}</p>}
            {failure && <ErrorCard failure={failure} />}
            <dl className="grid grid-cols-[auto_minmax(0,1fr)] gap-x-4 gap-y-2">
              <dt className="text-muted-foreground">Source</dt>
              <dd>{record.source}</dd>
              <dt className="text-muted-foreground">Time</dt>
              <dd>
                <time dateTime={timestamp}>
                  {new Date(record.at).toLocaleString()} ({relative})
                </time>
              </dd>
              <dt className="text-muted-foreground">Severity</dt>
              <dd className="capitalize">{record.tone}</dd>
            </dl>
            <FailureConsequences items={record.consequences ?? []} />
          </div>
        )}
        <DialogFooter showCloseButton>
          <span role="status" className="self-center text-xs text-muted-foreground">
            {copyStatus}
          </span>
          {!failure && (
            <Button
              variant="outline"
              onClick={async () => {
                if (!record) return;
                try {
                  await navigator.clipboard.writeText(notificationDetailsText(record, relative));
                  setCopyStatus("Copied");
                } catch (error: unknown) {
                  toast.error(failureOf(error, "Could Not Copy Notification"));
                  setCopyStatus("Could not copy. Try again.");
                }
              }}
            >
              Copy
            </Button>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

/* === Hooks === */

/* === Public Functions === */

/* === Private Functions === */
