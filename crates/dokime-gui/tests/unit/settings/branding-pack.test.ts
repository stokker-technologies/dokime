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
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { test } from "vitest";

// Local Imports
import { BrandingPackView } from "@/features/settings/sections/BrandingPackRow";
import { brandingPackPresentation } from "@/features/settings/sections/branding-pack";
import type { BrandingStatus } from "@/lib/desktop/branding";

/* === Re-Exports === */

/* === Constants === */

const NONE: BrandingStatus = { active: null, pending: null, removed: false, error: null };
const FORK = { label: "Fork", organization: "Fork Labs", ignored: ["slug", "bundle identifier"] };

/* === Types === */

/* === Classes === */

/* === Tests === */

test("the brand card names the pack, what happens next and the actions that apply", () => {
  const none = row(NONE);
  assert.match(none, /data-state="none"[^>]*>Compiled Brand</);
  assert.deepEqual(buttons(none), ["Choose File…"]);

  const pending = row({ ...NONE, pending: FORK });
  assert.match(pending, />Fork · Fork Labs</);
  assert.ok(pending.includes("Applies on next start."));
  assert.ok(pending.includes("Kept from this build: slug, bundle identifier."));
  assert.deepEqual(buttons(pending), ["Choose File…", "Restart", "Remove"]);

  const active = row({ ...NONE, active: FORK });
  assert.match(active, /data-state="active"/);
  assert.deepEqual(buttons(active), ["Choose File…", "Remove"]);

  const removed = row({ ...NONE, active: null, removed: true });
  assert.ok(removed.includes("Compiled brand applies on next start."));
  assert.deepEqual(buttons(removed), ["Choose File…", "Restart"]);
});

test("an invalid pack and a refused archive show as the field error", () => {
  const invalid = row({ ...NONE, error: "brand.toml does not parse" });
  assert.match(invalid, /data-state="invalid"/);
  assert.match(invalid, /role="alert"[^>]*>Not applied: brand.toml does not parse. The compiled brand applies./);
  assert.deepEqual(buttons(invalid), ["Choose File…", "Remove"]);

  const refused = row(NONE, "unsafe path in the branding pack: ../brand.toml");
  assert.match(refused, /role="alert"[^>]*>unsafe path in the branding pack: ..\/brand.toml</);
  assert.match(refused, /data-state="none"/, "a refused archive changes nothing");
});

/* === Private Functions === */

function row(status: BrandingStatus, choiceError: string | null = null): string {
  return renderToStaticMarkup(
    createElement(BrandingPackView, {
      presentation: brandingPackPresentation(status),
      choiceError,
      busy: false,
      choose: () => undefined,
      remove: () => undefined,
      restart: () => undefined,
    }),
  );
}

function buttons(html: string): string[] {
  return [...html.matchAll(/<button[^>]*>([^<]*)<\/button>/g)].map((match) => match[1] ?? "");
}
