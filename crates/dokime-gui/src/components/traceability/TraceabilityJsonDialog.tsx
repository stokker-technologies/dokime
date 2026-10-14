/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { Braces, Copy, Download } from "lucide-react";
import { useState } from "react";
import { toast } from "sonner";

/* === Local Imports === */

import { copyTraceabilityText } from "@/components/traceability/clipboard";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import {
  type TraceabilityExportName,
  traceabilityJson,
  traceabilityJsonFilename,
} from "@/lib/traceability";
import type { TraceabilitySnapshot } from "@/lib/types";

/* === Constants === */

/* === Types === */

type TraceabilityJsonDialogProps = {
  exportName?: TraceabilityExportName | null;
  traceability: TraceabilitySnapshot | null | undefined;
};

/* === Components === */

/**
 * The traceability card's header action: the whole traceability as JSON.
 *
 * Everything works from the snapshot already in memory, so the frozen report
 * copies and downloads exactly as the live view does, with no backend.
 */
export function TraceabilityJsonDialog({ exportName, traceability }: TraceabilityJsonDialogProps) {
  const [open, setOpen] = useState(false);
  const json = open ? traceabilityJson(traceability) : "";
  const filename = traceabilityJsonFilename(exportName);

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
      <DialogContent className="grid max-h-[min(48rem,calc(100dvh-4rem))] grid-rows-[auto_minmax(0,1fr)_auto] sm:max-w-3xl">
        <DialogHeader>
          <DialogTitle>Traceability JSON</DialogTitle>
          <DialogDescription className="truncate">{filename}</DialogDescription>
        </DialogHeader>
        <pre className="min-h-0 overflow-auto rounded-md border bg-muted/30 p-3 font-mono text-xs leading-5">
          {json}
        </pre>
        <DialogFooter className="gap-2">
          <Button
            onClick={() => void copyTraceabilityText(json)}
            size="sm"
            type="button"
            variant="outline"
          >
            <Copy data-icon="inline-start" />
            Copy
          </Button>
          <Button onClick={() => downloadJson(json, filename)} size="sm" type="button">
            <Download data-icon="inline-start" />
            Download
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

/* === Private Functions === */

/**
 * Hand the in-memory JSON to the browser as a file.
 *
 * The shell's native save accepts only HTML reports and diagnostics bundles,
 * so a JSON export goes through a Blob download in both the desktop and the
 * frozen report, neither of which needs a backend for it.
 */
function downloadJson(json: string, filename: string): void {
  try {
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
    window.setTimeout(() => URL.revokeObjectURL(url), 1_000);
  } catch {
    toast.error("Could not download traceability");
  }
}
