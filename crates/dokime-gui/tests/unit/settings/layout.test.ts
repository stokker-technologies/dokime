/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

// Standard Library Imports
import assert from "node:assert/strict";

// Third Party Imports
import { type ComponentProps, createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { test } from "vitest";

// Local Imports
import { SettingsCard } from "@/features/settings/SettingsCard";
import { SettingsGrid } from "@/features/settings/SettingsGrid";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

// Children Travel As createElement Arguments, Not Props.
type CardProps = ComponentProps<typeof SettingsCard>;

/* === Classes === */

/* === Tests === */

test("settings cards share one two-column grid and tables span both columns", () => {
  const html = renderToStaticMarkup(
    createElement(
      SettingsGrid,
      null,
      createElement(SettingsCard, { title: "Listener" } as CardProps, "rows"),
      createElement(SettingsCard, { title: "Secrets" } as CardProps, "rows"),
      createElement(SettingsCard, { span: "full", title: "Recent Remote Sessions" } as CardProps, "table"),
    ),
  );
  assert.match(html, /^<div class="[^"]*\bwide:grid-cols-2\b/);
  const cards = html.match(/<section class="[^"]*"/g) ?? [];
  assert.equal(cards.length, 3);
  assert.equal(cards.filter((card) => card.includes("wide:col-span-2")).length, 1);
  assert.ok(cards[2]?.includes("wide:col-span-2"));
});
