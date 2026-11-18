/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Native runtime shell, diagnostics, and observation stream adapters. */

/* === Local Imports === */

import { mapRuntimeShell } from "./native-runtime-projection";

import * as rpc from "@/lib/host/client";
import { required, type Wire } from "@/lib/rpc/canonical";
import type { EvidenceScope } from "@host/model";
import { type NativeTraceabilitySnapshot, mapNativeTraceability } from "./native-traceability-projection";
export type RuntimeStateFrame = Wire<
  Awaited<ReturnType<typeof rpc.backendOverviewWatch>> extends NativeStream<infer Frame> ? Frame : never
>;

import type { NativeStream, SessionIdentity } from "@/lib/rpc/transport";
import type { HubLiveState, TraceabilitySnapshot } from "@/lib/types";
import { listNativeCycles } from "./native-planning";
import { NATIVE_MAX_PAGES, NATIVE_PAGE_SIZE, NativeApiLimitError } from "./native-values";

/* === Types === */

type RuntimeShellSnapshot = Wire<NonNullable<Awaited<ReturnType<typeof rpc.backendOverviewGet>>["overview"]>>;
interface ExecutionProgressFrame {
  progress: { testExecutionId: string; label: string; phase: string; completed: bigint; total: bigint } | null;
  finished: boolean;
}

type TraceabilityCut = Wire<NonNullable<Awaited<ReturnType<typeof rpc.evidenceTraceabilityGet>>["resolvedAt"]>>;

/* === Runtime shell === */

export async function fetchNativeRuntimeShell(session: SessionIdentity, cycleId: string): Promise<HubLiveState> {
  const cycleSnapshots = await fetchRuntimeShellCycles(session);
  return mapRuntimeShell(cycleSnapshots, cycleId);
}

async function fetchRuntimeShellCycles(session: SessionIdentity): Promise<RuntimeShellSnapshot[]> {
  const reply = await rpc.backendOverviewGet(session, {});
  const snapshot = required(reply.overview, "Read Backend Overview");
  const cycles = await listNativeCycles(session);
  return [{ ...snapshot, cycleSummaries: cycles }];
}

type TraceabilityResponse = Awaited<ReturnType<typeof rpc.evidenceTraceabilityGet>>;

/** Read ordered traceability snapshots at one generation-bound, scoped read cut. */
export async function fetchNativeTraceability(session: SessionIdentity, scope?: Wire<EvidenceScope> | null): Promise<TraceabilitySnapshot> {
  const operation = "Traceability.Get";
  const requestedScope = scope ?? undefined;
  const items: NativeTraceabilitySnapshot[] = [];
  const tokens = new Set<string>();
  let pageToken = "";
  let at: TraceabilityCut | undefined;

  for (let page = 0; page < NATIVE_MAX_PAGES; page++) {
    assertTraceabilitySession(session, operation);
    const response = await rpc.evidenceTraceabilityGet(session, {
      ...(requestedScope ? { scope: requestedScope } : {}),
      page: { pageSize: NATIVE_PAGE_SIZE, pageToken },
      ...(at ? { at } : {}),
    });
    assertTraceabilitySession(session, operation);

    validateTraceabilityResponse(response, at, requestedScope, items.at(-1), operation);
    at ??= response.resolvedAt;
    items.push(...response.snapshots);

    const nextPageToken = response.nextPageToken;
    if (!nextPageToken) return mapNativeTraceability(items, requestedScope, operation);
    if (!response.snapshots.length) {
      throw new NativeApiLimitError(operation, "incomplete page contained no snapshots");
    }
    if (tokens.has(nextPageToken)) {
      throw new NativeApiLimitError(operation, "traceability pagination did not advance");
    }
    tokens.add(nextPageToken);
    pageToken = nextPageToken;
  }

  throw new NativeApiLimitError(operation, `more than ${NATIVE_MAX_PAGES} traceability pages`);
}

function validateTraceabilityResponse(
  response: TraceabilityResponse,
  expectedCut: TraceabilityCut | undefined,
  requestedScope: Wire<EvidenceScope> | undefined,
  previous: NativeTraceabilitySnapshot | undefined,
  operation: string,
): void {
  if (!response || !Array.isArray(response.snapshots) || typeof response.nextPageToken !== "string") {
    throw new NativeApiLimitError(operation, "response omitted its page fields");
  }
  const cut = response.resolvedAt;
  if (!cut || !Array.isArray(cut.positions) || !Array.isArray(cut.revisions) || cut.revisions.length) {
    throw new NativeApiLimitError(operation, "response omitted a usable read cut");
  }
  const histories = new Set<string>();
  for (const position of cut.positions) {
    const history = position.history;
    if (!history?.store?.trim() || !history.epoch?.trim() || typeof position.sequence !== "bigint" || position.sequence < 0n) {
      throw new NativeApiLimitError(operation, "read cut omitted a usable history position");
    }
    if (histories.has(history.epoch)) {
      throw new NativeApiLimitError(operation, "read cut repeated an execution history");
    }
    histories.add(history.epoch);
  }
  if (requestedScope?.executionId && (cut.positions.length !== 1 || !histories.has(requestedScope.executionId))) {
    throw new NativeApiLimitError(operation, "read cut did not retain the requested execution");
  }
  if (expectedCut && !sameTraceabilityCut(expectedCut, cut)) {
    throw new NativeApiLimitError(operation, "response read cut changed during pagination");
  }
  for (const snapshot of response.snapshots) {
    const position = snapshot.resolvedAt;
    const bound = cut.positions.find((item) => item.history?.epoch === position?.history?.epoch);
    if (
      !position ||
      !bound ||
      position.history?.store !== bound.history?.store ||
      typeof position.sequence !== "bigint" ||
      position.sequence < 1n ||
      position.sequence > bound.sequence ||
      !snapshot.revision?.length
    ) {
      throw new NativeApiLimitError(operation, "snapshot omitted its revision or fell outside the read cut");
    }
    if (!snapshot.scope || !matchesTraceabilityScope(requestedScope, snapshot.scope)) {
      throw new NativeApiLimitError(operation, "snapshot scope did not match the requested scope");
    }
    if (snapshot.scope.executionId !== position.history?.epoch) {
      throw new NativeApiLimitError(operation, "snapshot scope did not match its history");
    }
    if (previous?.resolvedAt) {
      const earlier = previous.resolvedAt;
      const epoch = required(position.history, "Read Traceability History").epoch;
      const earlierEpoch = required(earlier.history, "Read Previous Traceability History").epoch;
      if (epoch < earlierEpoch || (epoch === earlierEpoch && position.sequence <= earlier.sequence)) {
        throw new NativeApiLimitError(operation, "snapshots were not in strict history order");
      }
    }
    previous = snapshot;
  }
}

function sameTraceabilityCut(left: TraceabilityCut, right: TraceabilityCut): boolean {
  return (
    left.positions.length === right.positions.length &&
    left.positions.every((position, index) => {
      const other = right.positions[index];
      return (
        position.history?.store === other.history?.store &&
        position.history?.epoch === other.history?.epoch &&
        position.sequence === other.sequence
      );
    })
  );
}

function matchesTraceabilityScope(wanted: Wire<EvidenceScope> | undefined, actual: Wire<EvidenceScope>): boolean {
  if (!wanted) return true;
  // The Host Validates the Runtime Epoch; Snapshot Scopes Carry Durable Attribution.
  const { origin, sessionId: _sessionId, ...fields } = wanted;
  const matchesFields = Object.entries(fields).every(
    ([key, value]) => key.startsWith("$") || value == null || actual[key as keyof Wire<EvidenceScope>] === value,
  );
  return matchesFields && (!origin || JSON.stringify(origin.kind?.kind) === JSON.stringify(actual.origin?.kind?.kind));
}

function assertTraceabilitySession(session: SessionIdentity, operation: string): void {
  if (!session.isCurrent()) throw new Error(`${operation} lost its owning connection.`);
}

export interface NativeObservationSession extends SessionIdentity {
  own(dispose: () => void): () => void;
}

export async function openNativeRuntimeStream(
  session: NativeObservationSession,
  at?: Parameters<typeof rpc.backendOverviewWatch>[1]["at"],
): Promise<NativeStream<RuntimeStateFrame>> {
  const reader = await rpc.backendOverviewWatch(session, { at });
  let settled = false;
  let cancelPromise: Promise<void> | undefined;
  const cancelReader = (): Promise<void> => {
    cancelPromise ??= reader.cancel();
    return cancelPromise;
  };
  const release = session.own(() => {
    if (!settled) void cancelReader().catch(() => undefined);
  });
  let released = false;
  const releaseOnce = () => {
    if (!released) {
      released = true;
      release();
    }
  };
  const forward = (async function* () {
    let exhausted = false;
    try {
      for await (const frame of reader) {
        if (frame.resyncRequired) throw new Error(`Watch Backend Overview: ${frame.resyncRequired.message}; Refresh the Overview`);
        yield frame;
      }
      exhausted = true;
    } finally {
      if (!exhausted) await cancelReader().catch(() => undefined);
      settled = true;
      releaseOnce();
    }
  })();
  return {
    cancel: async () => {
      if (settled) return;
      await cancelReader();
      settled = true;
      releaseOnce();
    },
    [Symbol.asyncIterator]: () => forward,
  };
}

export async function openNativeExecutionProgressStream(
  session: NativeObservationSession,
  testExecutionId: string,
): Promise<NativeStream<ExecutionProgressFrame>> {
  const reader = await rpc.executionsProgressWatch(session, { name: testExecutionId });
  let settled = false;
  let cancelPromise: Promise<void> | undefined;
  const cancelReader = (): Promise<void> => {
    cancelPromise ??= reader.cancel();
    return cancelPromise;
  };
  const release = session.own(() => {
    if (!settled) void cancelReader().catch(() => undefined);
  });
  let released = false;
  const releaseOnce = () => {
    if (!released) {
      released = true;
      release();
    }
  };
  const forward = (async function* () {
    let exhausted = false;
    try {
      for await (const frame of reader)
        yield {
          progress:
            frame.value && !frame.finished
              ? {
                  testExecutionId,
                  label: frame.value.label,
                  phase: frame.phase,
                  completed: frame.value.completed,
                  total: frame.value.total,
                }
              : null,
          finished: frame.finished,
        };
      exhausted = true;
    } finally {
      if (!exhausted) await cancelReader().catch(() => undefined);
      settled = true;
      releaseOnce();
    }
  })();
  return {
    cancel: async () => {
      if (settled) return;
      await cancelReader();
      settled = true;
      releaseOnce();
    },
    [Symbol.asyncIterator]: () => forward,
  };
}

/* === Private mapping === */
