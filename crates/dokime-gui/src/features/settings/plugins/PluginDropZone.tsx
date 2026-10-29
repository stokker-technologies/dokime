/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { getCurrentWebview } from "@tauri-apps/api/webview";
import { FileUp, Loader2 } from "lucide-react";
import { type RefObject, useEffect, useRef, useState } from "react";

/* === Local Imports === */

import { Button } from "@/components/ui/button";
import { formatInstallPluginBytes, type InstallPluginProgress } from "@/lib/api/native-plugins";
import { useBrand } from "@/lib/brand";
import { isTauriDesktop } from "@/lib/desktop-open";
import { cn } from "@/lib/utils";

/* === Types === */

type PluginDropZoneProps = {
  disabled: boolean;
  error: string | null;
  filename: string | null;
  onDrop: (filename: string) => void;
  onPick: () => void;
  progress: InstallPluginProgress | null;
};

/* === Components === */

export function PluginDropZone({ disabled, error, filename, onDrop, onPick, progress }: PluginDropZoneProps) {
  const brand = useBrand();
  const zone = useRef<HTMLFieldSetElement>(null);
  const dragging = useNativeFileDrop(zone, disabled, onDrop);

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
        ref={zone}
      >
        <div className="grid max-w-md justify-items-center gap-3">
          {disabled ? (
            <Loader2 className="size-8 animate-spin text-muted-foreground" />
          ) : (
            <FileUp className="size-8 text-muted-foreground" />
          )}
          <div className="grid gap-1">
            <p className="font-medium">{disabled ? installationLabel(progress, filename) : "Drop a plugin bundle here"}</p>
            <p className="text-sm text-muted-foreground">{brand.label} verifies and installs the bundle. It activates after restart.</p>
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

/**
 * Follow the shell's native file drag over the zone. The webview never sees a
 * dropped file's bytes or path; the shell records the drop and installs it.
 */
function useNativeFileDrop(zone: RefObject<HTMLElement | null>, disabled: boolean, onDrop: (filename: string) => void): boolean {
  const [over, setOver] = useState(false);
  const latest = useRef({ disabled, onDrop });
  latest.current = { disabled, onDrop };

  useEffect(() => {
    if (!isTauriDesktop()) return;
    const inside = (position: { x: number; y: number }) => {
      const bounds = zone.current?.getBoundingClientRect();
      const x = position.x / window.devicePixelRatio;
      const y = position.y / window.devicePixelRatio;
      return Boolean(bounds && x >= bounds.left && x <= bounds.right && y >= bounds.top && y <= bounds.bottom);
    };
    let disposed = false;
    let unlisten: (() => void) | undefined;
    void getCurrentWebview()
      .onDragDropEvent(({ payload }) => {
        if (payload.type === "leave") return setOver(false);
        const hovering = inside(payload.position) && !latest.current.disabled;
        setOver(payload.type !== "drop" && hovering);
        if (payload.type === "drop" && hovering) {
          latest.current.onDrop(payload.paths[0]?.split(/[\\/]/).at(-1) ?? "plugin");
        }
      })
      .then((stop) => {
        if (disposed) stop();
        else unlisten = stop;
      });
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [zone]);

  return over;
}

/** Render the busy drop zone's status line for the mutation's current phase. */
function installationLabel(progress: InstallPluginProgress | null, filename: string | null): string {
  switch (progress?.phase) {
    case undefined:
      return `Installing ${filename ?? "plugin"}…`;
    case "reading":
      return "Reading file…";
    case "uploading":
      return `Uploading ${formatInstallPluginBytes(progress.sent)} of ${formatInstallPluginBytes(progress.total)}`;
    case "installing":
      return "Installing on the host…";
  }
}
