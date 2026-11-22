/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Typed native plot and telemetry adapters for the original routed UI. */

/* === Imports === */

import { PlotKind, PlotStreamKind, type PlotRequest, type SnapshotsWatchResponse } from "@generated/host/model";
import { type PlotEnvelope, mapNativePlotEnvelope, mapNativePlotPayload } from "@dokime/plots/lib/api/native-plot-projection";
import { plotsSnapshotsList, plotsSnapshotsWatch } from "@/lib/host/client";
import { cutKey, required, type Wire } from "@dokime/plots/lib/rpc/canonical";
import type { SessionIdentity } from "@/lib/rpc/transport";
import type { DocumentPlotType, PlotPayloadStreamMessage, PlotPayloadEnvelope } from "@dokime/plots/lib/types";
import { NATIVE_MAX_PAGES, NativeApiLimitError, safeNumber } from "@dokime/plots/lib/api/native-values";

/* === Constants === */

export const NATIVE_PLOT_MAX_REQUESTS = 32;

/* === Types === */

type PlotWatchPage = Wire<SnapshotsWatchResponse>;
type PlotStreamUpdate = PlotWatchPage["updates"][number];

type PlotTypes = ReadonlyMap<string, DocumentPlotType | null | undefined>;

/**
 * The watch's word that a plot is subscribed and still reading its first
 * frame: sent at once when the watch opens and as a keepalive while the watch
 * is quiet, even after the plot's data arrived. It carries no data and no
 * revision.
 */
export type NativePlotPendingMessage = {
  kind: "pending";
  plot_id: string;
};

/** One Plot Within An Ordered Watch Revision. */
export type NativePlotStreamUpdate = PlotPayloadStreamMessage | NativePlotPendingMessage;

/** One Complete Dirty Set, Or Payload-Free Keepalive Notices. */
export type NativePlotStreamMessage = { updates: NativePlotStreamUpdate[] };

export interface NativePlotStream extends AsyncIterable<NativePlotStreamMessage> {
  cancel(): Promise<void>;
}

export type NativePlotReadOptions = {
  signal?: AbortSignal;
};

export type NativePlotWatchOptions = NativePlotReadOptions;

/* === Public Functions === */

/** Read one finite batch without subscribing to subsequent runtime changes. */
export async function fetchNativePlotPayloads(
  session: SessionIdentity,
  requests: readonly PlotRequest[],
  options: NativePlotReadOptions = {},
): Promise<PlotPayloadEnvelope[]> {
  boundedRequestCount(requests.length, "Plots.Snapshots.List");
  ensureActive(session, options.signal);
  try {
    const envelopes: PlotPayloadEnvelope[] = [];
    const plotTypes = requestedPlotTypes(requests);
    let at: Wire<Awaited<ReturnType<typeof plotsSnapshotsList>>>["resolvedAt"];
    let token = "";
    const seen = new Set<string>();
    for (let page = 0; page < NATIVE_MAX_PAGES; page += 1) {
      const response = await plotsSnapshotsList(session, {
        requests: [...requests],
        at,
        page: { pageSize: NATIVE_PLOT_MAX_REQUESTS, pageToken: token },
      });
      ensureActive(session, options.signal);
      const resolved = required(response.resolvedAt, "Read Plot Snapshots: Missing Read Cut");
      if (at && cutKey(at) !== cutKey(resolved)) {
        throw new Error("Read Plot Snapshots: Read Cut Changed");
      }
      at = resolved;
      envelopes.push(...response.envelopes.map((envelope) => mapNativePlotEnvelope(envelope, plotTypes.get(envelope.plotId))));
      token = response.nextPageToken;
      if (!token) {
        return envelopes;
      }
      if (seen.has(token)) {
        throw new Error("Read Plot Snapshots: Repeated Continuation");
      }
      seen.add(token);
    }
    throw new NativeApiLimitError("Read Plot Snapshots", "page budget exhausted");
  } catch (error) {
    throw normalizeCancellation(error, session, options.signal);
  }
}

/**
 * Open the subscribed plot watch. Each request receives the server's
 * initial snapshots after subscription, so callers cannot miss a racing
 * evidence commit between a separate snapshot and watch setup.
 */
export async function openNativePlotStream(
  session: SessionIdentity,
  requests: readonly PlotRequest[],
  options: NativePlotWatchOptions = {},
): Promise<NativePlotStream> {
  boundedRequestCount(requests.length, "Plots.Watch");
  ensureActive(session, options.signal);
  const plotTypes = requestedPlotTypes(requests);

  let source: Awaited<ReturnType<typeof plotsSnapshotsWatch>> | null = null;
  try {
    source = await plotsSnapshotsWatch(session, { requests: [...requests] });
    ensureActive(session, options.signal);
  } catch (error) {
    await source?.cancel().catch(() => undefined);
    throw normalizeCancellation(error, session, options.signal);
  }
  if (!source) {
    throw new Error("Native plot watch returned no stream.");
  }

  let ended = false;
  const iterator = source[Symbol.asyncIterator]();
  let removeAbortListener: (() => void) | undefined;
  const cancel = async (): Promise<void> => {
    if (ended) {
      return;
    }
    ended = true;
    removeAbortListener?.();
    removeAbortListener = undefined;
    await source.cancel();
  };
  const onAbort = () => {
    void cancel().catch(() => undefined);
  };
  if (options.signal) {
    options.signal.addEventListener("abort", onAbort, { once: true });
    removeAbortListener = () => options.signal?.removeEventListener("abort", onAbort);
  }
  if (options.signal?.aborted || !session.isCurrent()) {
    await cancel().catch(() => undefined);
    throw abortError();
  }

  const mapped: AsyncIterator<NativePlotStreamMessage> = {
    async next() {
      if (ended) {
        return { done: true, value: undefined };
      }
      try {
        ensureActive(session, options.signal);
        const result = await iterator.next();
        if (result.done) {
          await cancel();
          return { done: true, value: undefined };
        }
        ensureActive(session, options.signal);
        return { done: false, value: mapNativePlotWatchPage(result.value, plotTypes) };
      } catch (error) {
        await cancel().catch(() => undefined);
        throw normalizeCancellation(error, session, options.signal);
      }
    },
    async return() {
      await cancel();
      return { done: true, value: undefined };
    },
  };

  return {
    cancel,
    [Symbol.asyncIterator]: () => mapped,
  };
}

/**
 * Map one protobuf storage-query snapshot into the renderer.
 *
 * An error frame reports one plot's failure while the watch stays open, so it
 * may carry no payload or diagnostics; it maps to an empty payload
 * with its warnings rather than ending the whole stream on a mapping throw.
 */
export function mapNativePlotWatchPage(frame: PlotWatchPage, plotTypes?: PlotTypes): NativePlotStreamMessage {
  if (frame.resyncRequired) {
    throw new Error("Watch Plot Snapshots: Resynchronization Required");
  }
  if (frame.updates.length === 0) {
    throw new Error("Native plot revision has no updates.");
  }
  return { updates: frame.updates.map((update) => mapNativePlotStreamUpdate(update, plotTypes?.get(update.envelope?.plotId ?? ""))) };
}

function mapNativePlotStreamUpdate(frame: PlotStreamUpdate, plotType?: DocumentPlotType | null): NativePlotStreamUpdate {
  if (!frame.envelope) {
    throw new Error("Native plot stream frame has no envelope.");
  }
  // An unspecified frame that names a plot and carries nothing is the watch's pending notice.
  if (frame.kind === PlotStreamKind.UNSPECIFIED && !frame.envelope.payload) {
    return { kind: "pending", plot_id: frame.envelope.plotId };
  }
  const kind = plotStreamKind(frame.kind);
  if (kind === "error") {
    return {
      ...mapNativePlotErrorEnvelope(frame.envelope, plotType),
      kind,
    };
  }
  const envelope = mapNativePlotEnvelope(frame.envelope, plotType);
  return {
    ...envelope,
    kind,
  };
}

function mapNativePlotErrorEnvelope(envelope: PlotEnvelope, plotType?: DocumentPlotType | null): PlotPayloadEnvelope {
  if (envelope.payload && envelope.diagnostics) {
    return mapNativePlotEnvelope(envelope, plotType);
  }
  return {
    plot_id: envelope.plotId,
    revision: safeNumber(envelope.revision, "Plots envelope revision"),
    epoch: envelope.epoch,
    selection: envelope.selection,
    payload: envelope.payload
      ? mapNativePlotPayload(envelope.payload, plotType)
      : { stream: "", x_key: "", max_points: 0, generated_at: "", series: [] },
    ...(envelope.diagnostics ? { diagnostics: envelope.diagnostics } : {}),
    warnings: envelope.warnings.length ? [...envelope.warnings] : ["Plot stream returned an error."],
  };
}

/* === Private Functions === */

function requestedPlotTypes(requests: readonly PlotRequest[]): PlotTypes {
  const kinds: Partial<Record<PlotKind, DocumentPlotType>> = {
    [PlotKind.TRAJECTORY]: "trajectory",
    [PlotKind.BAR]: "bar",
    [PlotKind.PIE]: "pie",
  };
  return new Map(requests.map((request) => [request.plotId ?? request.stream, kinds[request.plotType]]));
}

function plotStreamKind(value: PlotStreamKind): PlotPayloadStreamMessage["kind"] {
  switch (value) {
    case PlotStreamKind.SNAPSHOT:
      return "snapshot";
    case PlotStreamKind.REPLACE:
      return "replace";
    case PlotStreamKind.DELTA:
      return "delta";
    case PlotStreamKind.STATISTICS:
      return "statistics";
    case PlotStreamKind.ERROR:
      return "error";
    default:
      throw new Error("Native plot stream returned an unspecified kind.");
  }
}

function boundedRequestCount(count: number, operation: string): void {
  if (count > NATIVE_PLOT_MAX_REQUESTS) {
    throw new NativeApiLimitError(operation, `more than ${NATIVE_PLOT_MAX_REQUESTS} plots`);
  }
}

function ensureActive(session: SessionIdentity, signal?: AbortSignal): void {
  if (signal?.aborted || !session.isCurrent()) {
    throw abortError();
  }
}

function normalizeCancellation(error: unknown, session: SessionIdentity, signal?: AbortSignal): unknown {
  if (signal?.aborted || !session.isCurrent()) {
    return abortError();
  }
  return error;
}

function abortError(): DOMException {
  return new DOMException("Native plot request cancelled.", "AbortError");
}
