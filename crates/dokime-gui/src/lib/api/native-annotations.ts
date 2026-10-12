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

import { create } from "@bufbuild/protobuf";

/* === Local Imports === */

import * as rpc from "@/lib/rpc/generated/client";
import {
  type AnnotationOwner,
  AnnotationOwnerSchema,
  type AnnotationSelection,
  AnnotationSelectionSchema,
  ExecutionNoteOwnerSchema,
  type HealthItem as NativeHealthItem,
  type OperatorNote as NativeOperatorNote,
  NotePublication,
  PlannedCaseNoteOwnerSchema,
} from "@/lib/rpc/generated/dokime/v1/notes_pb";
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

/* === Public API === */

export async function fetchNativeNotes(
  session: SessionIdentity,
  scope: RecordScope,
): Promise<OperatorNote[]> {
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

export async function fetchNativeHealth(
  session: SessionIdentity,
  scope: RecordScope,
): Promise<HealthItem[]> {
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

/* === Mapping === */

function mapNativeNote(note: NativeOperatorNote): OperatorNote {
  const timestamp = timestampIso(note.createdAt);
  if (!timestamp) throw new Error(`Native note ${note.noteId} has no valid creation timestamp.`);
  return {
    note_id: note.noteId,
    text: note.message,
    timestamp,
    scope: recordScopeFromNative(note.scope),
  };
}

function mapNativeHealth(item: NativeHealthItem): HealthItem {
  const createdAt = timestampIso(item.observedAt);
  if (!createdAt)
    throw new Error(`Native health item ${item.healthItemId} has no valid observation timestamp.`);
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
  switch (value) {
    case NotePublication.NO_EXISTING_DATASETS:
      return "no_existing_datasets";
    case NotePublication.CONFIRMED:
      return "confirmed";
    case NotePublication.UNCONFIRMED:
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
    return create(AnnotationOwnerSchema, {
      owner: {
        case: "execution",
        value: create(ExecutionNoteOwnerSchema, {
          cycleId,
          executionId: scope.test_execution_id,
        }),
      },
    });
  }
  const caseId = scope.test_case_id;
  if (caseId) {
    return create(AnnotationOwnerSchema, {
      owner: {
        case: "plannedCase",
        value: create(PlannedCaseNoteOwnerSchema, { cycleId, caseId }),
      },
    });
  }
  return create(AnnotationOwnerSchema, { owner: { case: "cycleId", value: cycleId } });
}

function annotationSelection(scope: RecordScope): AnnotationSelection {
  return create(AnnotationSelectionSchema, {
    owner: annotationOwner(scope),
    includeContext: false,
  });
}
