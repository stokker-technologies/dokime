/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Pure helpers a document view composes: parameter scopes, the telemetry
 * context its plots read, and de-duplicated artifacts. Evidence values come
 * from the backend; nothing here derives them. */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

import type { DocumentViewCard, HubLiveState, TelemetryStream } from "@/lib/types";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Public Functions === */

export function mergeParameterContexts(...contexts: Array<Record<string, unknown> | null | undefined>): Record<string, unknown> | null {
  const merged: Record<string, unknown> = {};
  for (const context of contexts) {
    if (!context) {
      continue;
    }
    Object.assign(merged, context);
  }
  return Object.keys(merged).length ? merged : null;
}

export function plotContextTelemetrySnapshot(
  snapshot: HubLiveState | null,
  documentPlots: DocumentViewCard["plots"],
): Record<string, TelemetryStream> | null {
  if (!snapshot) {
    return null;
  }
  const entries = plotContextStreams(documentPlots)
    .map((stream) => [stream, snapshot.telemetry[stream]] as const)
    .filter((entry): entry is readonly [string, TelemetryStream] => Boolean(entry[1]));
  return entries.length ? Object.fromEntries(entries) : null;
}

export function uniqueArtifacts<T extends { artifact_id: string }>(artifacts: T[]): T[] {
  const seen = new Set<string>();
  return artifacts.filter((artifact) => {
    if (seen.has(artifact.artifact_id)) {
      return false;
    }
    seen.add(artifact.artifact_id);
    return true;
  });
}

/* === Private Functions === */

function plotContextStreams(documentPlots: DocumentViewCard["plots"]): string[] {
  const streams = new Set<string>();
  documentPlots.forEach((plot) => {
    const stream = plot.stream;
    if (!stream) {
      return;
    }
    streams.add(stream);
    if (stream === "host.wifi.scan") {
      streams.add("host.wifi.link");
    }
  });
  return [...streams].sort();
}

/* === Entry Point === */
