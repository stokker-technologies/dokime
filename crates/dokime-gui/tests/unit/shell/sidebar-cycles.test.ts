/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Sidebar Active Cycle Listing.
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

import { expect, test } from "vitest";

// Local Imports

import { type SidebarCycleSource, sidebarCycles } from "@/shell/sidebar-cycles";

/* === Re-Exports === */

/* === Constants === */

const OLD = row("old", "Active", "2026-01-01T00:00:00.000Z");
const NEW = row("new", "Active", "2026-03-01T00:00:00.000Z");
const SEALING = row("sealing", "Sealing", "2026-02-01T00:00:00.000Z");
const SEALED = row("sealed", "Sealed", "2026-04-01T00:00:00.000Z");

/* === Types === */

/* === Classes === */

/* === Public Functions === */

test("lists unsealed cycles newest first on any route", () => {
  expect(sidebarCycles([OLD, SEALED, NEW, SEALING], null).map((cycle) => [cycle.cycleId, cycle.current])).toEqual([
    ["new", false],
    ["sealing", false],
    ["old", false],
  ]);
});

test("marks the routed cycle current", () => {
  expect(sidebarCycles([OLD, NEW], "old").map((cycle) => [cycle.cycleId, cycle.current])).toEqual([
    ["new", false],
    ["old", true],
  ]);
});

test("a cycle leaves the list once it seals", () => {
  const sealed = { ...NEW, phase: "Sealed" as const };
  expect(sidebarCycles([OLD, NEW], null).map((cycle) => cycle.cycleId)).toContain("new");
  expect(sidebarCycles([OLD, sealed], null).map((cycle) => cycle.cycleId)).not.toContain("new");
});

test("keeps the routed cycle even when sealed or not yet listed", () => {
  expect(sidebarCycles([OLD, SEALED], "sealed").map((cycle) => [cycle.cycleId, cycle.current])).toEqual([
    ["sealed", true],
    ["old", false],
  ]);
  expect(sidebarCycles([OLD], "fresh").map((cycle) => [cycle.cycleId, cycle.current])).toEqual([
    ["fresh", true],
    ["old", false],
  ]);
});

/* === Private Functions === */

function row(cycleId: string, phase: SidebarCycleSource["phase"], createdAtIso: string): SidebarCycleSource {
  return { createdAtIso, cycleId, label: cycleId, phase };
}

/* === Entry Point === */
