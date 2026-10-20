/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { FileUp, Loader2 } from "lucide-react";
import { useState } from "react";

/* === Local Imports === */

import { Button } from "@/components/ui/button";
import { formatStagePluginBytes, type StagePluginProgress } from "@/lib/api/native-plugins";
import { useBrand } from "@/lib/brand";
import { cn } from "@/lib/utils";

/* === Types === */

type PluginDropZoneProps = {
  disabled: boolean;
  error: string | null;
  filename: string | null;
  onFile: (file: File) => void;
  onPick: () => void;
  progress: StagePluginProgress | null;
};

/* === Components === */

export function PluginDropZone({ disabled, error, filename, onFile, onPick, progress }: PluginDropZoneProps) {
  const brand = useBrand();
  const [dragging, setDragging] = useState(false);

  return (
    <div className="grid gap-2">
      <fieldset
        aria-busy={disabled}
        aria-label="Plugin file drop zone"
        className={cn(
          "grid min-h-40 place-items-center rounded-lg border border-dashed bg-muted/20 p-6 text-center transition-colors",
          dragging && "border-primary bg-primary/5",
          disabled && "opacity-70",
        )}
        disabled={disabled}
        onDragEnter={(event) => {
          event.preventDefault();
          if (!disabled) {
            setDragging(true);
          }
        }}
        onDragLeave={(event) => {
          if (!event.currentTarget.contains(event.relatedTarget as Node | null)) {
            setDragging(false);
          }
        }}
        onDragOver={(event) => {
          event.preventDefault();
          event.dataTransfer.dropEffect = "copy";
        }}
        onDrop={(event) => {
          event.preventDefault();
          setDragging(false);
          const file = event.dataTransfer.files.item(0);
          if (file && !disabled) {
            onFile(file);
          }
        }}
      >
        <div className="grid max-w-md justify-items-center gap-3">
          {disabled ? (
            <Loader2 className="size-8 animate-spin text-muted-foreground" />
          ) : (
            <FileUp className="size-8 text-muted-foreground" />
          )}
          <div className="grid gap-1">
            <p className="font-medium">{disabled ? stagingLabel(progress, filename) : "Drop a plugin bundle here"}</p>
            <p className="text-sm text-muted-foreground">{brand.label} stages and inspects its frozen catalog without running the file.</p>
          </div>
          <Button disabled={disabled} onClick={onPick} size="sm" type="button" variant="outline">
            Choose File
          </Button>
        </div>
      </fieldset>
      {error ? (
        <p className="text-sm text-destructive" role="alert">
          {error}
        </p>
      ) : null}
    </div>
  );
}

/* === Private Functions === */

/** Render the busy drop zone's status line for the mutation's current phase. */
function stagingLabel(progress: StagePluginProgress | null, filename: string | null): string {
  switch (progress?.phase) {
    case undefined:
      return `Inspecting ${filename ?? "plugin"}…`;
    case "reading":
      return "Reading file…";
    case "uploading":
      return `Uploading ${formatStagePluginBytes(progress.sent)} of ${formatStagePluginBytes(progress.total)}`;
    case "inspecting":
      return "Inspecting on the host…";
  }
}
