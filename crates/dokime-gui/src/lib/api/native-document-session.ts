/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Native Document Operations
 */

/* === Imports === */

import * as rpc from "@/lib/host/client";

import { cutKey, required } from "@dokime/plots/lib/rpc/canonical";

import type { SessionIdentity } from "@/lib/rpc/transport";

import type { DocumentDownsampleTechnique } from "@dokime/plots/lib/types";

import { Downsample } from "@host/model";

import { assembleDocument, sameFrontiers, type CompletedDocument } from "@dokime/document/lib/api/native-documents";

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Re-Exports === */

/* === Constants === */

const DOCUMENT_FRAME_PAGE_SIZE = 32;

/* === Types === */

export interface NativeDocumentRequest {
  at?: Parameters<typeof rpc.documentsGet>[1]["at"];
  cycleId: string;
  executionId: string;
  documentId: string;
  epoch: string;
  structureOnly?: boolean;
  cardIds?: string[];
  page?: string;
}

export interface NativeDocumentSession extends SessionIdentity {
  /** Keep the stream attached to the owning generation until it settles. */
  own(dispose: () => void): () => void;
}

/** One operator edit of a plot's display settings on a running cycle's frozen document. */
export interface PlotSettingsUpdate {
  cycleId: string;
  executionId: string;
  documentId: string;
  plotId: string;
  /** `null` restores the host default; `0` keeps every sample. */
  displayPoints: number | null;
  downsample: DocumentDownsampleTechnique;
}

/* === Classes === */

/* === Public Functions === */

/**
 * Persist a plot's display settings into the execution's frozen document, so
 * the live view and the sealed report render the same reduction.
 *
 */
export async function updatePlotSettings(session: SessionIdentity, update: PlotSettingsUpdate): Promise<void> {
  if (!session.isCurrent()) throw new Error("The backend connection was replaced.");
  if (!update.cycleId || !update.executionId || !update.documentId || !update.plotId) {
    throw new Error("An execution document and a plot are required.");
  }
  const variants = {
    auto: Downsample.AUTO,
    raw: Downsample.RAW,
    min_max: Downsample.MIN_MAX,
    lttb: Downsample.LTTB,
  } as const;
  const identity = {
    scope: { cycleId: update.cycleId, executionId: update.executionId },
    documentId: update.documentId,
    plotId: update.plotId,
  };
  const current = await rpc.plotsSettingsGet(session, identity);
  await rpc.plotsSettingsSet(session, {
    ...identity,
    expectedRevision: current.revision,
    requestId: crypto.randomUUID(),
    value: { displayPoints: update.displayPoints ?? undefined, downsample: variants[update.downsample] },
  });
  if (!session.isCurrent()) throw new Error("The backend connection was replaced.");
}

/** Open one generation-owned document stream and assemble it after clean EOF. */
export async function fetchNativeDocument(
  session: NativeDocumentSession,
  request: NativeDocumentRequest,
  signal?: AbortSignal,
  previous?: CompletedDocument | null,
  publish?: (document: CompletedDocument) => void,
): Promise<CompletedDocument> {
  const scope = createDocumentScope(request);
  const documentId = request.executionId && request.documentId === "default" ? "" : request.documentId;
  let cancelled = false;
  const dispose = session.own(() => {
    cancelled = true;
  });
  const active = () => !cancelled && !signal?.aborted && session.isCurrent();
  const frames = (async function* () {
    let pageToken = request.page ?? "";
    let at: Parameters<typeof rpc.documentsGet>[1]["at"] = request.at;
    const tokens = new Set<string>();
    for (let index = 0; index < 4096; index++) {
      if (!active()) throw new Error("Read Document: Selection Changed or Transfer Cancelled");
      const reply = await rpc.documentsGet(session, {
        scope,
        documentId,
        structureOnly: request.structureOnly ?? false,
        cardIds: request.cardIds ?? [],
        pageToken,
        maxFrames: DOCUMENT_FRAME_PAGE_SIZE,
        at,
      });
      const cut = required(reply.resolvedAt, "Read Document Cut");
      if (at && cutKey(at) !== cutKey(cut)) invalid("retained document cut changed");
      at = cut;
      for (const frame of reply.frames) yield frame;
      if (!reply.nextPageToken) return;
      if (tokens.has(reply.nextPageToken)) invalid("document continuation did not advance");
      pageToken = reply.nextPageToken;
      tokens.add(pageToken);
    }
    invalid("document page budget exceeded");
  })();
  try {
    return await assembleDocument(
      frames,
      { scope, documentId, epoch: request.epoch, structureOnly: request.structureOnly, cardIds: request.cardIds },
      active,
      publish,
      async (header) => {
        if (
          previous &&
          previous.complete !== false &&
          ![...previous.bindings].some(
            ([id, binding]) => (!request.cardIds?.length || request.cardIds.includes(id)) && binding.result?.kind.case === "failure",
          ) &&
          sameFrontiers(header.frontiers, previous.header.frontiers)
        )
          return previous;
        return undefined;
      },
    );
  } finally {
    dispose();
  }
}

function createDocumentScope(request: NativeDocumentRequest) {
  return {
    cycleId: request.cycleId,
    executionId: request.executionId,
  };
}

/* === Private Functions === */

function invalid(message: string): never {
  throw new Error(`Document transfer rejected: ${message}`);
}

/* === Entry Point === */
