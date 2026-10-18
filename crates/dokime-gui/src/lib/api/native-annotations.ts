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

import type { Wire } from "@/lib/rpc/canonical";
import * as rpc from "@/lib/rpc/generated/client";
import type { SessionIdentity } from "@/lib/rpc/transport";
import type { CommandReceipt, HealthItem, OperatorNote, RecordScope } from "@/lib/types";
import {
  mapNativeSeverity,
  NATIVE_MAX_PAGES,
  NATIVE_PAGE_SIZE,
  NativeApiLimitError,
  nowIso,
  recordScopeFromNative,
  timestampIso,
} from "./native-values";

/* === Types === */

type AnnotationOwner = Wire<NonNullable<Parameters<typeof rpc.notesCreateNote>[1]["owner"]>>;
type AnnotationSelection = Wire<NonNullable<Parameters<typeof rpc.notesListNotes>[1]["selection"]>>;
type NativeOperatorNote = Wire<Awaited<ReturnType<typeof rpc.notesListNotes>>["notes"][number]>;
type NativeHealthItem = Wire<Awaited<ReturnType<typeof rpc.healthListHealth>>["items"][number]>;
type NotePublication = Wire<Awaited<ReturnType<typeof rpc.notesCreateNote>>>["publication"];

/* === Public API === */

export async function fetchNativeNotes(session: SessionIdentity, scope: RecordScope): Promise<OperatorNote[]> {
  const selection = annotationSelection(scope);
  const notes: NativeOperatorNote[] = [];
  let afterId = "";
  let cursor: Parameters<typeof rpc.notesListNotes>[1]["cursor"];
  for (let page = 0; page < NATIVE_MAX_PAGES; page++) {
    const reply = await rpc.notesListNotes(session, {
      selection,
      limit: NATIVE_PAGE_SIZE,
      afterId,
      cursor,
    });
    notes.push(...reply.notes);
    const next = reply.nextAfterId;
    if (!next) return notes.map(mapNativeNote);
    if (next === afterId) {
      throw new NativeApiLimitError("Notes.ListNotes", `cursor did not advance at ${next}`);
    }
    afterId = next;
    cursor = reply.cursor;
  }
  throw new NativeApiLimitError("Notes.ListNotes", `more than ${NATIVE_MAX_PAGES} pages`);
}

export async function fetchNativeHealth(session: SessionIdentity, scope: RecordScope): Promise<HealthItem[]> {
  const selection = annotationSelection(scope);
  const items: NativeHealthItem[] = [];
  let afterId = "";
  let cursor: Parameters<typeof rpc.healthListHealth>[1]["cursor"];
  for (let page = 0; page < NATIVE_MAX_PAGES; page++) {
    const reply = await rpc.healthListHealth(session, {
      selection,
      limit: NATIVE_PAGE_SIZE,
      afterId,
      cursor,
    });
    items.push(...reply.items);
    const next = reply.nextAfterId;
    if (!next) return items.map(mapNativeHealth);
    if (next === afterId) {
      throw new NativeApiLimitError("Health.ListHealth", `cursor did not advance at ${next}`);
    }
    afterId = next;
    cursor = reply.cursor;
  }
  throw new NativeApiLimitError("Health.ListHealth", `more than ${NATIVE_MAX_PAGES} pages`);
}

export async function createNativeNote(
  session: SessionIdentity,
  epoch: string,
  text: string,
  scope?: RecordScope | null,
): Promise<CommandReceipt> {
  const message = text.trim();
  if (!message) throw new Error("A note cannot be empty.");
  const mutationId = crypto.randomUUID();
  const owner = annotationOwner(scope);
  const receipt = await rpc.notesCreateNote(session, {
    epoch,
    mutationId,
    owner,
    message,
  });
  const acceptedAt = timestampIso(receipt.note?.createdAt) ?? nowIso();
  return {
    command_id: mutationId,
    kind: "note.add",
    target_id: receipt.note?.noteId ?? scope?.test_cycle_id ?? "run",
    status: "applied",
    accepted_at: acceptedAt,
    updated_at: acceptedAt,
    result: {
      publication: notePublicationName(receipt.publication),
      checkpoint_targets: receipt.checkpointTargets,
      unconfirmed_targets: receipt.unconfirmedTargets,
    },
  };
}

export async function editNativeNote(session: SessionIdentity, epoch: string, noteId: string, text: string): Promise<OperatorNote> {
  const message = text.trim();
  if (!message) throw new Error("A note cannot be empty.");
  const reply = await rpc.notesEditNote(session, { epoch, noteId, message });
  if (!reply.note) throw new Error(`Native note ${noteId} returned no edited note.`);
  return mapNativeNote(reply.note);
}

/* === Mapping === */

function mapNativeNote(note: NativeOperatorNote): OperatorNote {
  const timestamp = timestampIso(note.createdAt);
  if (!timestamp) throw new Error(`Native note ${note.noteId} has no valid creation timestamp.`);
  return {
    note_id: note.noteId,
    author: note.author ?? null,
    text: note.message,
    timestamp,
    scope: recordScopeFromNative(note.scope),
  };
}

function mapNativeHealth(item: NativeHealthItem): HealthItem {
  const createdAt = timestampIso(item.observedAt);
  if (!createdAt) throw new Error(`Native health item ${item.healthItemId} has no valid observation timestamp.`);
  return {
    health_id: item.healthItemId,
    level: mapNativeSeverity(item.severity),
    title: item.message,
    message: item.detail ?? null,
    details: {},
    created_at: createdAt,
    scope: recordScopeFromNative(item.scope),
  };
}

function notePublicationName(value: NotePublication): string {
  switch (value?.HarmosVariant.case) {
    case "NoExistingDatasets":
      return "no_existing_datasets";
    case "Confirmed":
      return "confirmed";
    case "Unconfirmed":
      return "unconfirmed";
    default:
      return "unspecified";
  }
}

export function annotationOwner(scope?: RecordScope | null): AnnotationOwner {
  const cycleId = scope?.test_cycle_id;
  if (!cycleId) {
    throw new Error("Native notes require a cycle-scoped owner.");
  }
  if (scope.test_execution_id) {
    return {
      owner: {
        HarmosVariant: {
          case: "Execution",
          value: { field0: { cycleId, executionId: scope.test_execution_id } },
        },
      },
    };
  }
  const caseId = scope.test_case_id;
  if (caseId) {
    return {
      owner: { HarmosVariant: { case: "PlannedCase", value: { field0: { cycleId, caseId } } } },
    };
  }
  return { owner: { HarmosVariant: { case: "CycleId", value: { field0: cycleId } } } };
}

function annotationSelection(scope: RecordScope): AnnotationSelection {
  return { owner: annotationOwner(scope), includeContext: false };
}
