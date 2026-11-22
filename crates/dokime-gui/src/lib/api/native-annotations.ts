/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Native notes and health adapters for the original routed UI. */

/* === External Imports === */

/* === Local Imports === */

import { cutKey, required } from "@/lib/rpc/canonical";
import type { EvidenceScope } from "@generated/host/model";
import { type NativeHealthItem, type NativeOperatorNote, mapNativeHealth, mapNativeNote } from "@/lib/api/native-annotation-projection";

import * as rpc from "@/lib/host/client";
import type { Wire } from "@/lib/rpc/canonical";
import type { SessionIdentity } from "@/lib/rpc/transport";
import type { CommandReceipt, HealthItem, OperatorNote } from "@/lib/types";
import { NATIVE_MAX_PAGES, NATIVE_SMALL_PAGE_SIZE, NativeApiLimitError, nowIso, timestampIso } from "@/lib/api/native-values";

/* === Types === */

type AnnotationOwner = Wire<NonNullable<Parameters<typeof rpc.evidenceNotesCreate>[1]["owner"]>>;
type AnnotationSelection = Wire<NonNullable<Parameters<typeof rpc.evidenceNotesList>[1]["selection"]>>;

/* === Public API === */

export async function fetchNativeNotes(session: SessionIdentity, scope: Wire<EvidenceScope>): Promise<OperatorNote[]> {
  const selection = annotationSelection(scope);
  const items: NativeOperatorNote[] = [];
  let pageToken = "";
  let at: Parameters<typeof rpc.evidenceNotesList>[1]["at"];
  const tokens = new Set<string>();
  for (let index = 0; index < NATIVE_MAX_PAGES; index++) {
    const reply = await rpc.evidenceNotesList(session, { selection, at, page: { pageSize: NATIVE_SMALL_PAGE_SIZE, pageToken } });
    const cut = required(reply.resolvedAt, "Read Annotation Cut");
    if (at && cutKey(at) !== cutKey(cut)) throw new Error("Read Annotations: Retained Cut Changed");
    at = cut;
    items.push(...reply.notes);
    if (!reply.nextPageToken) return items.map(mapNativeNote);
    if (tokens.has(reply.nextPageToken)) throw new NativeApiLimitError("Read Annotations", "Repeated Page Token");
    pageToken = reply.nextPageToken;
    tokens.add(pageToken);
  }
  throw new NativeApiLimitError("Read Annotations", "Page Budget Reached");
}

export async function fetchNativeHealth(session: SessionIdentity, scope: Wire<EvidenceScope>): Promise<HealthItem[]> {
  const selection = annotationSelection(scope);
  const items: NativeHealthItem[] = [];
  let pageToken = "";
  let at: Parameters<typeof rpc.evidenceHealthEventsList>[1]["at"];
  const tokens = new Set<string>();
  for (let index = 0; index < NATIVE_MAX_PAGES; index++) {
    const reply = await rpc.evidenceHealthEventsList(session, { selection, at, page: { pageSize: NATIVE_SMALL_PAGE_SIZE, pageToken } });
    const cut = required(reply.resolvedAt, "Read Annotation Cut");
    if (at && cutKey(at) !== cutKey(cut)) throw new Error("Read Annotations: Retained Cut Changed");
    at = cut;
    items.push(...reply.events);
    if (!reply.nextPageToken) return items.map(mapNativeHealth);
    if (tokens.has(reply.nextPageToken)) throw new NativeApiLimitError("Read Annotations", "Repeated Page Token");
    pageToken = reply.nextPageToken;
    tokens.add(pageToken);
  }
  throw new NativeApiLimitError("Read Annotations", "Page Budget Reached");
}

export async function createNativeNote(
  session: SessionIdentity,
  text: string,
  scope?: Wire<EvidenceScope> | null,
): Promise<CommandReceipt> {
  const message = text.trim();
  if (!message) throw new Error("A note cannot be empty.");
  const mutationId = crypto.randomUUID();
  const owner = annotationOwner(scope);
  const receipt = await rpc.evidenceNotesCreate(session, {
    requestId: mutationId,
    owner,
    value: { message },
  });
  const acceptedAt = timestampIso(receipt.note?.createdAt) ?? nowIso();
  return {
    command_id: mutationId,
    kind: "note.add",
    target_id: receipt.note?.noteId ?? scope?.cycleId ?? "run",
    status: "applied",
    accepted_at: acceptedAt,
    updated_at: acceptedAt,
    result: { accepted: receipt.receipt?.completed },
  };
}

export async function editNativeNote(session: SessionIdentity, noteId: string, text: string): Promise<OperatorNote> {
  const message = text.trim();
  if (!message) throw new Error("A note cannot be empty.");
  const current = await rpc.evidenceNotesGet(session, { name: noteId });
  await rpc.evidenceNotesSet(session, {
    name: noteId,
    value: { message },
    expectedRevision: current.revision,
    requestId: crypto.randomUUID(),
  });
  return mapNativeNote(required((await rpc.evidenceNotesGet(session, { name: noteId })).note, "Read Edited Note"));
}

/* === Mapping === */

export function annotationOwner(scope?: Wire<EvidenceScope> | null): AnnotationOwner {
  const cycleId = scope?.cycleId;
  if (!cycleId) {
    throw new Error("Native notes require a cycle-scoped owner.");
  }
  if (scope.executionId) {
    return { owner: { kind: { case: "execution", value: { cycleId, executionId: scope.executionId } } } };
  }
  if (scope.caseId) {
    return { owner: { kind: { case: "plannedCase", value: { cycleId, caseId: scope.caseId } } } };
  }
  return { owner: { kind: { case: "cycleId", value: cycleId } } };
}

function annotationSelection(scope: Wire<EvidenceScope>): AnnotationSelection {
  return { owner: annotationOwner(scope), includeContext: false };
}
