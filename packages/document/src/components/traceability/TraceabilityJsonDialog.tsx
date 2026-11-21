/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { failureOf } from "@dokime/plots/lib/notification-details";
import { Braces, Copy, Download } from "lucide-react";
import { createContext, useContext, useState } from "react";
import { toast } from "@dokime/plots/lib/notifications";

/* === Local Imports === */

import { copyTraceabilityText } from "./clipboard";
import { Button } from "@dokime/plots/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@dokime/plots/components/ui/dialog";
import { Tooltip, TooltipContent, TooltipTrigger } from "@dokime/plots/components/ui/tooltip";
import { type TraceabilityExportName, traceabilityJson, traceabilityJsonFilename } from "../../lib/traceability";
import type { TraceabilitySnapshot } from "@dokime/plots/lib/types";

/* === Constants === */

const GUI_DOWNLOAD_REVOKE_MS = 1_000;

/* === Types === */

export const TraceabilitySaveContext = createContext<((json: string, filename: string) => Promise<void>) | null>(null);

type TraceabilityJsonDialogProps = {
  exportName?: TraceabilityExportName | null;
  traceability: TraceabilitySnapshot | null | undefined;
};

/* === Components === */

/**
 * The traceability card's header action: the whole traceability as JSON.
 *
 * Everything works from the snapshot already in memory. Download goes through
 * the shell's native save on the desktop; the frozen report has no shell, so
 * there it is a browser download.
 */
export function TraceabilityJsonDialog({ exportName, traceability }: TraceabilityJsonDialogProps) {
  const [open, setOpen] = useState(false);
  const json = open ? traceabilityJson(traceability) : "";
  const filename = traceabilityJsonFilename(exportName);
  const save = useContext(TraceabilitySaveContext);
  const [saving, setSaving] = useState(false);

  const download = async () => {
    setSaving(true);
    try {
      if (save) await save(json, filename);
      else browserDownload(json, filename);
    } catch (error) {
      toast.error(failureOf(error, "Could Not Download Traceability"));
    } finally {
      setSaving(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <Tooltip>
        <TooltipTrigger asChild>
          <Button
            aria-label="View Traceability as JSON"
            disabled={!traceability?.groups.length}
            onClick={() => setOpen(true)}
            size="icon-xs"
            type="button"
            variant="ghost"
          >
            <Braces />
          </Button>
        </TooltipTrigger>
        <TooltipContent side="top">View as JSON</TooltipContent>
      </Tooltip>
      <DialogContent className="grid max-h-[min(--spacing(192),calc(100dvh_-_--spacing(16)))] grid-rows-[auto_minmax(0,1fr)_auto] sm:max-w-3xl">
        <DialogHeader>
          <DialogTitle>Traceability JSON</DialogTitle>
          <DialogDescription className="truncate">{filename}</DialogDescription>
        </DialogHeader>
        <pre className="min-h-0 overflow-auto rounded-lg border bg-muted/30 p-3 font-mono text-xs leading-5">{json}</pre>
        <DialogFooter className="gap-2">
          <Button onClick={() => void copyTraceabilityText(json)} size="sm" type="button" variant="outline">
            <Copy data-icon="inline-start" />
            Copy
          </Button>
          <Button disabled={saving} onClick={() => void download()} size="sm" type="button">
            <Download data-icon="inline-start" />
            Download
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

/* === Private Functions === */

/** Hand the in-memory JSON to the browser as a file. */
function browserDownload(json: string, filename: string): void {
  const url = URL.createObjectURL(new Blob([json], { type: "application/json" }));
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = filename;
  anchor.rel = "noopener";
  document.body.append(anchor);
  anchor.click();
  anchor.remove();
  // The click starts the download synchronously; the URL is released after
  // the browser has had a turn to read it.
  window.setTimeout(() => URL.revokeObjectURL(url), GUI_DOWNLOAD_REVOKE_MS);
}
