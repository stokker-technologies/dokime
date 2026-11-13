/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * The operator's host capture controls in the media card: a screenshot, a
 * camera photo and an audio recording, each stored as an execution artifact.
 */

/* === Imports === */

import { Camera, Mic, Monitor } from "lucide-react";
import { useNativeSessionOptional } from "@/app/native-session/context";
import { Button } from "@/components/ui/button";
import type { RecordScope } from "@/lib/types";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

type MediaCaptureControlsProps = { scope: RecordScope };

/* === Components === */

export function MediaCaptureControls(_props: MediaCaptureControlsProps) {
  const session = useNativeSessionOptional();
  if (!session) return null;
  return (
    <div className="flex flex-wrap items-center gap-2 border-b pb-3">
      <Button disabled size="sm" type="button" variant="outline">
        <Monitor data-icon="inline-start" />
        Screenshot
      </Button>
      <Button disabled size="sm" type="button" variant="outline">
        <Camera data-icon="inline-start" />
        Photo
      </Button>
      <Button disabled size="sm" type="button" variant="outline">
        <Mic data-icon="inline-start" />
        Record Audio
      </Button>
      <span className="text-xs text-muted-foreground" role="status">
        Host media capture is unavailable until the host supplies its capture operations.
      </span>
    </div>
  );
}

/* === Hooks === */

/* === Public Functions === */

/* === Private Functions === */
