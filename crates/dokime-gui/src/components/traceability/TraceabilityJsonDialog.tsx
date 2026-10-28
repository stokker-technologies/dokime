/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import { Braces, Copy, Download } from "lucide-react";
import { useState } from "react";
import { toast } from "sonner";

import { useNativeSessionOptional } from "@/app/native-session/context";
import { copyTraceabilityText } from "@/components/traceability/clipboard";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { saveNativeDocument } from "@/lib/api/native-delivery";
import type { SessionIdentity } from "@/lib/rpc/transport";
import { type TraceabilityExportName, traceabilityJson, traceabilityJsonFilename } from "@/lib/traceability";
import type { TraceabilitySnapshot } from "@/lib/types";
import { isReportBuild } from "@/lib/ui-mode";

const GUI_DOWNLOAD_REVOKE_MS = 1_000;

type TraceabilityJsonDialogProps = {
  exportName?: TraceabilityExportName | null;
  traceability: TraceabilitySnapshot | null | undefined;
};

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
  const session = useNativeSessionOptional();
  const [saving, setSaving] = useState(false);

  const download = async () => {
    setSaving(true);
    try {
      await saveTraceabilityJson(json, filename, session?.identity ?? null);
    } catch (error) {
      toast.error("Could not download traceability", {
        description: error instanceof Error ? error.message : String(error),
      });
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

/**
 * Save the JSON where this surface can: the native picker on the desktop, a
 * browser download in the frozen report. Resolves quietly when the operator
 * cancels the picker; throws when neither route is available.
 */
async function saveTraceabilityJson(json: string, filename: string, session: SessionIdentity | null): Promise<void> {
  if (isReportBuild()) {
    browserDownload(json, filename);
    return;
  }
  if (!session) {
    throw new Error("Saving needs a connected desktop session.");
  }
  await saveNativeDocument(session, {
    kind: "json_document",
    filename,
    bytes: new TextEncoder().encode(json),
  });
}

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
