/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Shared Native Response Projection
 */

/* === Imports === */

import type * as hostModel from "@host/model";

import type { Wire } from "@dokime/plots/lib/rpc/canonical";

import type { HealthItem, OperatorNote } from "@dokime/plots/lib/types";

import { Severity } from "@host/model";

import { timestampIso } from "@dokime/plots/lib/api/native-values";

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

export type NativeOperatorNote = Wire<hostModel.NotesListResponse["notes"][number]>;

export type NativeHealthItem = Wire<hostModel.HealthEventsListResponse["events"][number]>;

/* === Classes === */

/* === Public Functions === */

export function mapNativeNote(note: NativeOperatorNote): OperatorNote {
  const timestamp = timestampIso(note.createdAt);
  if (!timestamp) throw new Error(`Native note ${note.noteId} has no valid creation timestamp.`);
  return {
    note_id: note.noteId,
    author: note.author ?? null,
    text: note.message,
    timestamp,
    scope: note.scope ?? null,
  };
}

export function mapNativeHealth(item: NativeHealthItem): HealthItem {
  const createdAt = timestampIso(item.observedAt);
  if (!createdAt) throw new Error(`Native health item ${item.healthItemId} has no valid observation timestamp.`);
  return {
    health_id: item.healthItemId,
    level:
      item.severity === Severity.FAILURE
        ? "error"
        : item.severity === Severity.WARNING
          ? "warning"
          : item.severity === Severity.DEBUG
            ? "debug"
            : "info",
    title: item.message,
    message: item.detail ?? null,
    details: {},
    created_at: createdAt,
    scope: item.scope ?? null,
  };
}

/* === Private Functions === */

/* === Entry Point === */
