/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Operator media captured on the backend host through the `Media` operations;
 * each capture is stored as an execution artifact the media gallery shows.
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
import * as rpc from "@/lib/rpc/generated/client";
import type { SessionIdentity } from "@/lib/rpc/transport";
import type { RecordScope } from "@/lib/types";
import { uploadScope } from "./native-content";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

export type NativeAudioRecording = {
  /** Cancel the stream; the backend seals what it recorded so far. */
  stop(): Promise<void>;
  /** Settles when the stream ends, with the sealed name when the backend announced it. */
  done: Promise<string | null>;
};

/* === Classes === */

/* === Public Functions === */

/** Capture the primary display as `screenshots/<timestamp>.png`; returns the artifact name. */
export async function captureNativeScreenshot(session: SessionIdentity, scope: RecordScope): Promise<string> {
  const artifact = await rpc.mediaScreenshot(session, { scope: uploadScope(scope) });
  return artifact.filename;
}

/** Capture one frame of the first camera as `photos/<timestamp>.jpg`; returns the artifact name. */
export async function captureNativePhoto(session: SessionIdentity, scope: RecordScope): Promise<string> {
  const artifact = await rpc.mediaPhoto(session, { scope: uploadScope(scope) });
  return artifact.filename;
}

/** Record the default audio input as `audio/<timestamp>.wav` until stopped. */
export async function startNativeAudioRecording(
  session: SessionIdentity,
  scope: RecordScope,
  onProgress: (elapsedMs: number) => void,
): Promise<NativeAudioRecording> {
  const stream = await rpc.mediaAudioRecording(session, { scope: uploadScope(scope) });
  let stopped = false;
  const done = (async () => {
    try {
      for await (const frame of stream) {
        const variant = frame.payload?.HarmosVariant;
        if (variant?.case === "Progress") {
          onProgress(Number(variant.value.field0?.elapsedMs ?? 0n));
        }
        if (variant?.case === "Completed") {
          return variant.value.field0?.filename ?? null;
        }
      }
      return null;
    } catch (error) {
      if (stopped) return null;
      throw error;
    }
  })();
  return {
    stop: async () => {
      stopped = true;
      await stream.cancel();
    },
    done,
  };
}

/* === Private Functions === */

/* === Entry Point === */
