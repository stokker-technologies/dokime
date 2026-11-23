// Dokime — SPDX-License-Identifier: Apache-2.0

/* === Public Functions === */

export function formatHistoryDate(iso: string): string {
  const date = new Date(iso);
  return Number.isNaN(date.getTime()) ? iso : date.toLocaleDateString();
}
