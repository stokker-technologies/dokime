/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { useQueryClient } from "@tanstack/react-query";
import { FileUp, Image as ImageIcon, Square, Video } from "lucide-react";
import { type MutableRefObject, useEffect, useId, useRef, useState } from "react";

/* === Local Imports === */

import { useNativeSession } from "@/app/native-session/context";
import { useOperatorArtifactUploadMutation } from "@/components/artifact-upload-mutation";
import { Button } from "@/components/ui/button";
import { CardContent } from "@/components/ui/card";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Label } from "@/components/ui/label";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { invalidateWorkflowDocuments, invalidateWorkflowResourceQueries } from "@/features/cycle/server-state";
import { uploadScope } from "@/lib/api/native-content";
import { type NativeRecordingSink, openNativeRecording } from "@/lib/rpc/artifact-upload";
import type { RecordScope } from "@/lib/types";

/* === Types === */

type ArtifactUploaderProps = {
  executionScope: RecordScope | null;
  mode?: "combined" | "upload" | "record";
  variant?: "card" | "embedded";
  cycleScope: RecordScope | null;
  existingArtifactPaths?: string[];
  onArtifactsChanged: () => Promise<void>;
};

/* === Constants === */

/** Bounds peak webview heap to one fragment of the recording, not the whole thing. */
const RECORDING_TIMESLICE_MS = 2_000;

/* === Components === */

export function ArtifactUploader({
  executionScope,
  mode = "combined",
  variant = "card",
  cycleScope,
  existingArtifactPaths = [],
  onArtifactsChanged,
}: ArtifactUploaderProps) {
  const idPrefix = useId();
  const fileInput = useRef<HTMLInputElement | null>(null);
  const recorder = useRef<MediaRecorder | null>(null);
  const recordingStream = useRef<MediaStream | null>(null);
  const displayStream = useRef<MediaStream | null>(null);
  const microphoneStream = useRef<MediaStream | null>(null);
  const audioMixContext = useRef<AudioContext | null>(null);
  const recordingSink = useRef<NativeRecordingSink | null>(null);
  const recordingWriteQueue = useRef<Promise<void>>(Promise.resolve());
  const recordingWroteAnyBytes = useRef(false);
  /** Set by the first abandonment reason (a failed timeslice write); read by
   * `finishRecording` once the recorder that abandonment stops eventually
   * fires `onstop`. */
  const recordingAbandoned = useRef<string | null>(null);
  const recordingMediaType = useRef<string>("");
  const recordingStartedAtRef = useRef<number | null>(null);
  const recordingFinalizedAt = useRef<number | null>(null);
  const cameraStream = useRef<MediaStream | null>(null);
  const cameraVideo = useRef<HTMLVideoElement | null>(null);
  const cameraCanvas = useRef<HTMLCanvasElement | null>(null);
  const [artifactStatus, setArtifactStatus] = useState<string | null>(null);
  const [uploading, setUploading] = useState(false);
  const [cameraOpen, setCameraOpen] = useState(false);
  const [cameraUploading, setCameraUploading] = useState(false);
  const [recordingState, setRecordingState] = useState<"idle" | "starting" | "recording" | "stopping" | "uploading">("idle");
  const [recordingElapsedMs, setRecordingElapsedMs] = useState(0);
  const [scopeKind, setScopeKind] = useState<"cycle" | "execution">(executionScope ? "execution" : "cycle");
  const uploadArtifactMutation = useOperatorArtifactUploadMutation();
  const session = useNativeSession();
  const queryClient = useQueryClient();
  const showUpload = mode !== "record";
  const showRecording = mode !== "upload";
  const artifactScopeId = `${idPrefix}-artifact-scope`;
  const artifactFileId = `${idPrefix}-artifact-file`;
  const scopeOptionCount = executionScope ? 2 : 1;

  const resolvedScope = scopeKind === "execution" && executionScope ? executionScope : cycleScope;

  useEffect(() => {
    setScopeKind(executionScope ? "execution" : "cycle");
  }, [executionScope?.test_execution_id, executionScope]);

  useEffect(() => {
    if (!executionScope && scopeKind === "execution") {
      setScopeKind("cycle");
    }
    if (executionScope && !cycleScope) {
      setScopeKind("execution");
    }
  }, [executionScope, scopeKind, cycleScope]);

  useEffect(
    () => () => {
      // A recording sink open at unmount time (navigating away mid-recording)
      // never gets a `finish` call; cancel it so its temp file doesn't linger.
      void recordingSink.current?.cancel();
    },
    [],
  );

  useEffect(() => {
    if (recordingState !== "recording") {
      return;
    }
    const tick = () => {
      const start = recordingStartedAtRef.current;
      if (start === null) return;
      setRecordingElapsedMs(Date.now() - start);
    };
    tick();
    const handle = window.setInterval(tick, 250);
    return () => window.clearInterval(handle);
  }, [recordingState]);

  useEffect(() => {
    if (!cameraOpen || !cameraVideo.current || !cameraStream.current) {
      return;
    }
    cameraVideo.current.srcObject = cameraStream.current;
    void cameraVideo.current.play().catch(() => undefined);
  }, [cameraOpen]);

  async function handleFileUpload(file: File | undefined) {
    if (!file) {
      return;
    }
    setUploading(true);
    setArtifactStatus(null);
    try {
      await uploadArtifactMutation.mutateAsync({
        file,
        filename: file.name,
        label: file.name,
        mediaType: file.type || "application/octet-stream",
        scope: resolvedScope,
      });
      setArtifactStatus("Uploaded");
      if (fileInput.current) {
        fileInput.current.value = "";
      }
      await onArtifactsChanged();
    } catch (exc) {
      setArtifactStatus(errorMessage(exc));
    } finally {
      setUploading(false);
    }
  }

  async function startRecording() {
    if (!navigator.mediaDevices?.getDisplayMedia || typeof MediaRecorder === "undefined") {
      setArtifactStatus("Screen recording is not available in this browser.");
      return;
    }
    if (!resolvedScope) {
      setArtifactStatus("Select a cycle or execution scope before recording.");
      return;
    }
    setRecordingState("starting");
    setArtifactStatus(null);
    try {
      // Step 1: screen + tab/system audio.
      const display = await navigator.mediaDevices.getDisplayMedia({
        video: true,
        audio: true,
      });
      displayStream.current = display;

      // Step 2: microphone. ``getDisplayMedia({audio: true})`` only
      // captures the page/system audio, never the mic, so we ask
      // separately and mix the streams. If the operator declines the
      // mic permission, we record without it instead of failing.
      let microphone: MediaStream | null = null;
      let microphoneError: string | null = null;
      try {
        microphone = await navigator.mediaDevices.getUserMedia({
          audio: {
            echoCancellation: true,
            noiseSuppression: true,
            autoGainControl: true,
          },
          video: false,
        });
      } catch (exc) {
        microphoneError = errorMessage(exc);
      }
      microphoneStream.current = microphone;

      const compositeStream = composeRecordingStream(display, microphone, audioMixContext);

      recordingStream.current = compositeStream;

      const recordingFormat = preferredRecordingFormat();
      recordingMediaType.current = recordingFormat.mediaType;
      recordingFinalizedAt.current = null;
      recordingWroteAnyBytes.current = false;
      recordingAbandoned.current = null;

      // Opened before the recorder starts: a sink that fails to open aborts
      // the capture instead of recording content with nowhere to land.
      const filename = nextGeneratedFilename(existingArtifactPaths, "recording", recordingFormat.extension);
      const sink = await openNativeRecording(session.identity, {
        scope: uploadScope(resolvedScope),
        filename,
        contentType: recordingFormat.mediaType,
      });
      recordingSink.current = sink;
      recordingWriteQueue.current = Promise.resolve();

      const instance = new MediaRecorder(compositeStream, {
        mimeType: recordingFormat.mediaType,
      });
      recorder.current = instance;
      instance.ondataavailable = (event) => {
        if (event.data.size === 0) return;
        recordingWroteAnyBytes.current = true;
        const chunk = event.data;
        // Serialized: at most one native write in flight, so timeslices
        // land in the recorder's own arrival order.
        recordingWriteQueue.current = recordingWriteQueue.current
          .then(async () => {
            const bytes = new Uint8Array(await chunk.arrayBuffer());
            await recordingSink.current?.write(bytes);
          })
          .catch((exc) => {
            // A write failure abandons the recording: further timeslices
            // would just be silently dropped otherwise. Cancel the sink
            // right away and let the recorder's own `stop` (below) settle
            // through the usual `onstop` -> `finishRecording` path, where
            // `recordingAbandoned` is read.
            if (recordingAbandoned.current) return;
            recordingAbandoned.current = errorMessage(exc);
            void recordingSink.current?.cancel();
            stopRecording();
          });
      };
      instance.onerror = (event: ErrorEvent) => {
        // Unlike a failed write, the recorder itself is not guaranteed to
        // fire `stop` after `error`, so this path cleans up directly rather
        // than relying on `finishRecording`.
        const recorderError = event.error as DOMException | undefined;
        setArtifactStatus(`Recording failed: ${recorderError?.message ?? "unknown error"}`);
        void recordingSink.current?.cancel();
        cleanupRecording();
        setRecordingState("idle");
        recordingStartedAtRef.current = null;
        setRecordingElapsedMs(0);
      };
      instance.onstop = () => {
        recordingFinalizedAt.current = Date.now();
        void finishRecording();
      };
      // The display tracks fire ``ended`` when the operator hits the
      // browser's "stop sharing" pill; bind the stop hook there. The
      // mic tracks ending shouldn't terminate the recording so they
      // are not bound here.
      display.getTracks().forEach((track) => {
        track.addEventListener("ended", () => {
          stopRecording();
        });
      });
      // A 2 s timeslice bounds peak webview heap to one fragment instead of
      // the whole (up to 2 GiB) recording: each fragment streams straight to
      // the native sink as it arrives, instead of buffering in
      // `recordingChunks`.
      instance.start(RECORDING_TIMESLICE_MS);
      recordingStartedAtRef.current = Date.now();
      setRecordingElapsedMs(0);
      setRecordingState("recording");
      setArtifactStatus(
        microphoneError === null
          ? "Recording screen with system audio and microphone"
          : `Recording screen with system audio (no microphone: ${microphoneError})`,
      );
    } catch (exc) {
      cleanupRecording();
      setRecordingState("idle");
      recordingStartedAtRef.current = null;
      setRecordingElapsedMs(0);
      setArtifactStatus(errorMessage(exc));
    }
  }

  function stopRecording() {
    const activeRecorder = recorder.current;
    if (!activeRecorder || activeRecorder.state === "inactive") {
      return;
    }
    setRecordingState("stopping");
    activeRecorder.stop();
  }

  async function finishRecording() {
    const startedAt = recordingStartedAtRef.current;
    const finalizedAt = recordingFinalizedAt.current ?? Date.now();
    const durationMs = startedAt !== null ? Math.max(0, finalizedAt - startedAt) : 0;
    const mediaType = recordingMediaType.current;
    setRecordingState("uploading");
    setArtifactStatus("Saving recording");
    try {
      // Every queued timeslice write must land before `finish` reads the
      // file. A write failure already cancelled the sink from within the
      // queue itself and recorded its reason here, rather than rejecting
      // this await (further timeslices must still no-op instead of piling
      // up rejected promises).
      await recordingWriteQueue.current;
      if (recordingAbandoned.current) {
        throw new Error(recordingAbandoned.current);
      }
      if (!recordingWroteAnyBytes.current) {
        throw new Error("Recording produced no data.");
      }
      const { commands } = await import("@/lib/desktop/commands");
      await commands.desktopRecordingFinish(session.identity.generation, durationMs, mediaType);
      setArtifactStatus("Recording saved");
      await Promise.all([invalidateWorkflowResourceQueries(queryClient), invalidateWorkflowDocuments(queryClient)]);
      await onArtifactsChanged();
    } catch (exc) {
      setArtifactStatus(errorMessage(exc));
      // Covers the failed-`finish` path; a failed write already cancelled
      // the sink above, and `cancel` is safe to call again.
      void recordingSink.current?.cancel();
    } finally {
      cleanupRecording();
      setRecordingState("idle");
      recordingStartedAtRef.current = null;
      setRecordingElapsedMs(0);
    }
  }

  async function openCameraCapture() {
    if (!navigator.mediaDevices?.getUserMedia) {
      setArtifactStatus("Camera capture is not available in this browser.");
      return;
    }
    setArtifactStatus(null);
    try {
      closeCameraCapture();
      cameraStream.current = await navigator.mediaDevices.getUserMedia({
        video: {
          facingMode: "user",
        },
        audio: false,
      });
      setCameraOpen(true);
    } catch (exc) {
      closeCameraCapture();
      setArtifactStatus(errorMessage(exc));
    }
  }

  async function capturePhoto() {
    const video = cameraVideo.current;
    const canvas = cameraCanvas.current;
    if (!video || !canvas || !resolvedScope) {
      return;
    }
    const width = video.videoWidth || 1280;
    const height = video.videoHeight || 720;
    canvas.width = width;
    canvas.height = height;
    const context = canvas.getContext("2d");
    if (!context) {
      setArtifactStatus("Camera frame capture is not available.");
      return;
    }
    context.drawImage(video, 0, 0, width, height);
    setCameraUploading(true);
    try {
      const blob = await canvasToBlob(canvas, "image/jpeg", 0.92);
      const filename = nextGeneratedFilename(existingArtifactPaths, "photo", "jpg");
      await uploadArtifactMutation.mutateAsync({
        file: blob,
        filename,
        label: filename,
        mediaType: "image/jpeg",
        scope: resolvedScope,
        groupId: "operator-photos",
        groupLabel: "Operator Photos",
      });
      setArtifactStatus("Photo saved");
      closeCameraCapture();
      await onArtifactsChanged();
    } catch (exc) {
      setArtifactStatus(errorMessage(exc));
    } finally {
      setCameraUploading(false);
    }
  }

  function closeCameraCapture() {
    cameraStream.current?.getTracks().forEach((track) => {
      track.stop();
    });
    cameraStream.current = null;
    setCameraOpen(false);
  }

  function cleanupRecording() {
    recordingStream.current?.getTracks().forEach((track) => {
      track.stop();
    });
    displayStream.current?.getTracks().forEach((track) => {
      track.stop();
    });
    microphoneStream.current?.getTracks().forEach((track) => {
      track.stop();
    });
    if (audioMixContext.current) {
      void audioMixContext.current.close().catch(() => undefined);
    }
    recordingStream.current = null;
    displayStream.current = null;
    microphoneStream.current = null;
    audioMixContext.current = null;
    recorder.current = null;
    recordingSink.current = null;
    recordingWroteAnyBytes.current = false;
    recordingAbandoned.current = null;
    recordingFinalizedAt.current = null;
  }

  const stopBusy = recordingState === "stopping" || recordingState === "uploading";
  const recordingActive = recordingState !== "idle";
  const captureBusy = uploading || recordingActive || cameraUploading;
  const actionButtonClassName = "min-h-11 min-w-[10.5rem] flex-1 justify-center whitespace-normal text-center text-sm font-semibold";

  const content = (
    <>
      <div className="grid gap-2">
        <Label id={artifactScopeId}>Scope</Label>
        <ToggleGroup
          aria-labelledby={artifactScopeId}
          className={`grid w-full ${scopeOptionCount === 2 ? "grid-cols-2" : "grid-cols-1"}`}
          size="default"
          spacing={0}
          type="single"
          value={scopeKind}
          variant="outline"
          onValueChange={(value) => {
            if (value === "cycle" || value === "execution") {
              setScopeKind(value);
            }
          }}
        >
          <ToggleGroupItem className="w-full justify-center" value="cycle">
            Cycle
          </ToggleGroupItem>
          {executionScope ? (
            <ToggleGroupItem className="w-full justify-center" value="execution">
              Execution
            </ToggleGroupItem>
          ) : null}
        </ToggleGroup>
      </div>
      {showUpload || showRecording ? (
        <div className="flex flex-wrap gap-2">
          {showUpload ? (
            <>
              <input
                className="sr-only"
                id={artifactFileId}
                ref={fileInput}
                disabled={captureBusy}
                type="file"
                onChange={(event) => {
                  void handleFileUpload(event.target.files?.[0]);
                }}
              />
              <Button
                className={actionButtonClassName}
                disabled={captureBusy}
                type="button"
                variant="outline"
                onClick={() => {
                  fileInput.current?.click();
                }}
              >
                <FileUp data-icon="inline-start" />
                Choose File
              </Button>
            </>
          ) : null}
          {showRecording ? (
            recordingState === "idle" ? (
              <Button
                className={actionButtonClassName}
                disabled={uploading || cameraUploading}
                type="button"
                variant="outline"
                onClick={() => {
                  void startRecording();
                }}
              >
                <Video data-icon="inline-start" />
                Start Recording
              </Button>
            ) : (
              <Button
                aria-label={`Stop recording (${formatElapsed(recordingElapsedMs)})`}
                className={actionButtonClassName}
                disabled={stopBusy}
                size="lg"
                type="button"
                variant="danger"
                onClick={stopRecording}
              >
                <span className="relative flex items-center gap-2">
                  <span aria-hidden className="inline-block h-2 w-2 rounded-full bg-current animate-pulse" />
                  <Square size={14} aria-hidden />
                  <span>{stopBusy ? "Saving" : "Stop"}</span>
                  <span className="font-mono tabular-nums text-xs opacity-90" aria-live="polite">
                    {formatElapsed(recordingElapsedMs)}
                  </span>
                </span>
              </Button>
            )
          ) : null}
          <Button
            className={actionButtonClassName}
            disabled={captureBusy}
            type="button"
            variant="outline"
            onClick={() => {
              void openCameraCapture();
            }}
          >
            <ImageIcon data-icon="inline-start" />
            Take Picture
          </Button>
        </div>
      ) : null}
      <div className="flex min-h-5 items-center gap-2 text-xs text-muted-foreground">
        {uploading || recordingState === "uploading" ? <FileUp size={14} /> : null}
        {artifactStatus ? <span>{artifactStatus}</span> : null}
      </div>
      <Dialog
        open={cameraOpen}
        onOpenChange={(open) => {
          if (!open) {
            closeCameraCapture();
          }
        }}
      >
        <DialogContent className="max-w-3xl">
          <DialogHeader>
            <DialogTitle>Take Picture</DialogTitle>
            <DialogDescription>Capture a webcam image and attach it to the selected scope.</DialogDescription>
          </DialogHeader>
          <div className="overflow-hidden rounded-xl border bg-black">
            <video className="max-h-[60vh] w-full object-contain" muted playsInline ref={cameraVideo} />
            <canvas className="hidden" ref={cameraCanvas} />
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={closeCameraCapture}>
              Cancel
            </Button>
            <Button
              disabled={cameraUploading}
              onClick={() => {
                void capturePhoto();
              }}
            >
              <ImageIcon data-icon="inline-start" />
              {cameraUploading ? "Saving" : "Capture"}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  );

  if (variant === "embedded") {
    return <div className="grid gap-4">{content}</div>;
  }

  return <CardContent className="grid gap-3 border-b p-4">{content}</CardContent>;
}

function formatElapsed(ms: number): string {
  const totalSeconds = Math.max(0, Math.floor(ms / 1000));
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;
  const pad = (value: number) => value.toString().padStart(2, "0");
  if (hours > 0) {
    return `${hours}:${pad(minutes)}:${pad(seconds)}`;
  }
  return `${pad(minutes)}:${pad(seconds)}`;
}

/* === Private Functions === */

function composeRecordingStream(
  display: MediaStream,
  microphone: MediaStream | null,
  audioMixContextRef: MutableRefObject<AudioContext | null>,
): MediaStream {
  const videoTracks = display.getVideoTracks();
  const displayAudioTracks = display.getAudioTracks();
  const micAudioTracks = microphone?.getAudioTracks() ?? [];

  // Fast path: only one audio source (or none). No mixing needed.
  if (displayAudioTracks.length === 0 && micAudioTracks.length === 0) {
    return new MediaStream(videoTracks);
  }
  if (displayAudioTracks.length === 0) {
    return new MediaStream([...videoTracks, ...micAudioTracks]);
  }
  if (micAudioTracks.length === 0) {
    return new MediaStream([...videoTracks, ...displayAudioTracks]);
  }

  // Mix system audio + microphone via AudioContext. Each source becomes
  // a node, both connect to a single MediaStreamDestination which emits
  // one combined audio track for the recorder.
  const ctx = new AudioContext();
  audioMixContextRef.current = ctx;
  const dest = ctx.createMediaStreamDestination();
  ctx.createMediaStreamSource(new MediaStream(displayAudioTracks)).connect(dest);
  ctx.createMediaStreamSource(new MediaStream(micAudioTracks)).connect(dest);
  return new MediaStream([...videoTracks, ...dest.stream.getAudioTracks()]);
}

function preferredRecordingFormat(): { extension: "mp4" | "webm"; mediaType: string } {
  const candidates = [
    { extension: "mp4", mediaType: "video/mp4;codecs=avc1.42E01E,mp4a.40.2" },
    { extension: "mp4", mediaType: "video/mp4;codecs=h264,aac" },
    { extension: "mp4", mediaType: "video/mp4" },
    { extension: "webm", mediaType: "video/webm;codecs=vp9,opus" },
    { extension: "webm", mediaType: "video/webm;codecs=vp8,opus" },
    { extension: "webm", mediaType: "video/webm" },
  ] as const;
  return (
    candidates.find((candidate) => MediaRecorder.isTypeSupported(candidate.mediaType)) ?? {
      extension: "webm",
      mediaType: "video/webm",
    }
  );
}

function errorMessage(exc: unknown): string {
  return exc instanceof Error ? exc.message : String(exc);
}

async function canvasToBlob(canvas: HTMLCanvasElement, type: string, quality?: number): Promise<Blob> {
  return new Promise((resolve, reject) => {
    canvas.toBlob(
      (blob) => {
        if (!blob) {
          reject(new Error("Camera capture produced no image data."));
          return;
        }
        resolve(blob);
      },
      type,
      quality,
    );
  });
}

function nextGeneratedFilename(existingArtifactPaths: string[], prefix: "photo" | "recording", extension: string): string {
  const nextIndex =
    existingArtifactPaths
      .map(pathBasename)
      .map((filename) => {
        const match = new RegExp(`^${prefix}-(\\d+)\\.[^.]+$`, "i").exec(filename);
        return match ? Number.parseInt(match[1] ?? "0", 10) : 0;
      })
      .reduce((max, value) => Math.max(max, Number.isFinite(value) ? value : 0), 0) + 1;
  return `${prefix}-${String(nextIndex).padStart(3, "0")}.${extension}`;
}

function pathBasename(path: string): string {
  const normalized = path.replace(/\\/g, "/");
  return normalized.split("/").at(-1) ?? normalized;
}
