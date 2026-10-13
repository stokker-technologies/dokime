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

// Standard Library Imports

// Third Party Imports
import { useQueryClient } from "@tanstack/react-query";
import { Camera, Mic, Monitor, Square } from "lucide-react";
import { useEffect, useRef, useState } from "react";

// Local Imports
import { useNativeSessionOptional } from "@/app/native-session/NativeSessionProvider";
import { Button } from "@/components/ui/button";
import {
  invalidateWorkflowDocuments,
  invalidateWorkflowResourceQueries,
} from "@/features/cycle/server-state";
import {
  captureNativePhoto,
  captureNativeScreenshot,
  type NativeAudioRecording,
  startNativeAudioRecording,
} from "@/lib/api/native-media";
import type { SessionIdentity } from "@/lib/rpc/transport";
import type { RecordScope } from "@/lib/types";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

type MediaCaptureControlsProps = {
  /** A cycle and execution scope; captures are execution artifacts. */
  scope: RecordScope;
};

type Activity = "idle" | "screenshot" | "photo" | "recording" | "stopping";

/* === Components === */

export function MediaCaptureControls({ scope }: MediaCaptureControlsProps) {
  const session = useNativeSessionOptional();
  /* Host capture needs a connected backend; a browser build has none. */
  return session ? <CaptureControls identity={session.identity} scope={scope} /> : null;
}

function CaptureControls({
  identity,
  scope,
}: MediaCaptureControlsProps & { identity: SessionIdentity }) {
  const queryClient = useQueryClient();
  const recording = useRef<NativeAudioRecording | null>(null);
  const stopRequested = useRef(false);
  const [activity, setActivity] = useState<Activity>("idle");
  const [elapsedMs, setElapsedMs] = useState(0);
  const [status, setStatus] = useState<string | null>(null);
  const busy = activity !== "idle";

  /* Leaving the card stops the recording; the backend seals what it heard. */
  useEffect(
    () => () => {
      stopRequested.current = true;
      void recording.current?.stop().catch(() => undefined);
    },
    [],
  );

  async function refresh() {
    await Promise.all([
      invalidateWorkflowResourceQueries(queryClient),
      invalidateWorkflowDocuments(queryClient),
    ]);
  }

  async function capture(kind: "screenshot" | "photo") {
    setActivity(kind);
    setStatus(null);
    try {
      const name =
        kind === "screenshot"
          ? await captureNativeScreenshot(identity, scope)
          : await captureNativePhoto(identity, scope);
      setStatus(`Saved ${name}`);
      await refresh();
    } catch (error) {
      setStatus(errorMessage(error));
    } finally {
      setActivity("idle");
    }
  }

  async function startRecording() {
    setActivity("recording");
    setStatus(null);
    setElapsedMs(0);
    stopRequested.current = false;
    try {
      const started = await startNativeAudioRecording(identity, scope, setElapsedMs);
      recording.current = started;
      /* A stop pressed while the device was still opening applies now. */
      if (stopRequested.current) void started.stop().catch(() => undefined);
      const name = await started.done;
      setStatus(name ? `Saved ${name}` : "Recording saved");
    } catch (error) {
      setStatus(errorMessage(error));
    } finally {
      recording.current = null;
      setActivity("idle");
      await refresh();
    }
  }

  async function stopRecording() {
    setActivity("stopping");
    stopRequested.current = true;
    await recording.current?.stop().catch((error: unknown) => setStatus(errorMessage(error)));
  }

  return (
    <div className="flex flex-wrap items-center gap-2 border-b pb-3">
      <Button
        disabled={busy}
        size="sm"
        type="button"
        variant="outline"
        onClick={() => void capture("screenshot")}
      >
        <Monitor data-icon="inline-start" />
        {activity === "screenshot" ? "Capturing" : "Screenshot"}
      </Button>
      <Button
        disabled={busy}
        size="sm"
        type="button"
        variant="outline"
        onClick={() => void capture("photo")}
      >
        <Camera data-icon="inline-start" />
        {activity === "photo" ? "Capturing" : "Photo"}
      </Button>
      {activity === "recording" || activity === "stopping" ? (
        <Button
          aria-label={`Stop audio recording (${formatElapsed(elapsedMs)})`}
          disabled={activity === "stopping"}
          size="sm"
          type="button"
          variant="danger"
          onClick={() => void stopRecording()}
        >
          <Square data-icon="inline-start" />
          {activity === "stopping" ? "Saving" : "Stop"}
          <span className="font-mono text-xs tabular-nums" aria-live="polite">
            {formatElapsed(elapsedMs)}
          </span>
        </Button>
      ) : (
        <Button
          disabled={busy}
          size="sm"
          type="button"
          variant="outline"
          onClick={() => void startRecording()}
        >
          <Mic data-icon="inline-start" />
          Record Audio
        </Button>
      )}
      {status ? <span className="text-xs text-muted-foreground">{status}</span> : null}
    </div>
  );
}

/* === Hooks === */

/* === Public Functions === */

/* === Private Functions === */

function formatElapsed(ms: number): string {
  const totalSeconds = Math.max(0, Math.floor(ms / 1000));
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;
  return `${String(minutes).padStart(2, "0")}:${String(seconds).padStart(2, "0")}`;
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
