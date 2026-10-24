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
import { OrganizationPackView } from "@/features/settings/sections/OrganizationPackRow";
import { organizationPackPresentation } from "@/features/settings/sections/organization-pack";
import type { OrganizationStatus } from "@/lib/desktop/organization";

/* === Re-Exports === */

/* === Constants === */

const NONE: OrganizationStatus = { active: null, pending: null, removed: false, error: null };
const FORK = { label: "Fork", organization: "Fork Labs" };

/* === Types === */

/* === Classes === */

/* === Tests === */

test("the brand card names the pack, what happens next and the actions that apply", () => {
  const none = row(NONE);
  assert.match(none, /data-state="none"[^>]*>Bundled Organization</);
  assert.deepEqual(buttons(none), ["Choose File…"]);

  const pending = row({ ...NONE, pending: FORK });
  assert.match(pending, />Fork · Fork Labs</);
  assert.ok(pending.includes("Applies on next start."));
  assert.deepEqual(buttons(pending), ["Choose File…", "Restart", "Remove"]);

  const active = row({ ...NONE, active: FORK });
  assert.match(active, /data-state="active"/);
  assert.deepEqual(buttons(active), ["Choose File…", "Remove"]);

  const removed = row({ ...NONE, active: null, removed: true });
  assert.ok(removed.includes("Bundled organization applies on next start."));
  assert.deepEqual(buttons(removed), ["Choose File…", "Restart"]);
});

test("an invalid pack and a refused archive show as the field error", () => {
  const invalid = row({ ...NONE, error: "organization.toml does not parse" });
  assert.match(invalid, /data-state="invalid"/);
  assert.match(invalid, /role="alert"[^>]*>Not applied: organization.toml does not parse. The bundled organization applies./);
  assert.deepEqual(buttons(invalid), ["Choose File…", "Remove"]);

  const refused = row(NONE, "unsafe path in the organization pack: ../organization.toml");
  assert.match(refused, /role="alert"[^>]*>unsafe path in the organization pack: ..\/organization.toml</);
  assert.match(refused, /data-state="none"/, "a refused archive changes nothing");
});

test("a refused pending pack reports that the last valid pack remains active", () => {
  const retained = row({ ...NONE, active: FORK, error: "pending pack does not parse" });

  assert.match(retained, /data-state="active"/);
  assert.match(retained, />Fork · Fork Labs</);
  assert.match(retained, /The last valid pack remains active/);
  assert.doesNotMatch(retained, /The bundled organization applies/);
});

/* === Private Functions === */

function row(status: OrganizationStatus, choiceError: string | null = null): string {
  return renderToStaticMarkup(
    createElement(OrganizationPackView, {
      presentation: organizationPackPresentation(status),
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
