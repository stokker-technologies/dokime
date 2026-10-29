/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import assert from "node:assert/strict";

import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { test, vi } from "vitest";

import { OrganizationPackView } from "@/features/settings/sections/OrganizationPackRow";
import { organizationPackPresentation } from "@/features/settings/sections/organization-pack";
import type { OrganizationStatus } from "@/lib/desktop/organization";

const notice = vi.hoisted(() => vi.fn());
vi.mock("@/components/NotificationMessage", () => ({
  NotificationMessage: (props: unknown) => {
    notice(props);
    return null;
  },
}));

const NONE: OrganizationStatus = { active: null, pending: null, removed: false, error: null };
const CUSTOM = { label: "Stokker Technologies Test Hub", organization: "Stokker Technologies" };

test("the brand card names the pack, what happens next and the actions that apply", () => {
  const none = row(NONE);
  assert.match(none, /data-state="none"[^>]*>Bundled Organization</);
  assert.deepEqual(buttons(none), ["Choose File…"]);

  const pending = row({ ...NONE, pending: CUSTOM });
  assert.match(pending, />Stokker Technologies Test Hub · Stokker Technologies</);
  assert.equal(organizationPackPresentation({ ...NONE, pending: CUSTOM }).restart, true);
  assert.deepEqual(buttons(pending), ["Choose File…", "Restart", "Remove"]);

  const active = row({ ...NONE, active: CUSTOM });
  assert.match(active, /data-state="active"/);
  assert.deepEqual(buttons(active), ["Choose File…", "Remove"]);

  const removed = row({ ...NONE, active: null, removed: true });
  assert.equal(organizationPackPresentation({ ...NONE, removed: true }).state, "removed");
  assert.deepEqual(buttons(removed), ["Choose File…", "Restart"]);
});

test("Invalid Packs Notify", () => {
  const invalid = row({ ...NONE, error: "organization.toml does not parse" });
  assert.match(invalid, /data-state="invalid"/);
  assert.equal(notice.mock.calls.length, 1);
  assert.doesNotMatch(invalid, /role="alert"/);
  assert.deepEqual(buttons(invalid), ["Choose File…", "Remove"]);

  const refused = row(NONE, "unsafe path in the organization pack: ../organization.toml");
  assert.equal(notice.mock.calls.length, 1);
  assert.doesNotMatch(refused, /role="alert"/);
  assert.match(refused, /data-state="none"/, "a refused archive changes nothing");
});

test("Failed Activation Retains Pack", () => {
  const retained = row({ ...NONE, active: CUSTOM, error: "pending pack does not parse" });

  assert.match(retained, /data-state="active"/);
  assert.match(retained, />Stokker Technologies Test Hub · Stokker Technologies</);
  assert.equal(notice.mock.calls.length, 1);
  const presentation = organizationPackPresentation({ ...NONE, active: CUSTOM, error: "fixture" });
  assert.equal(presentation.state, "active");
  assert.equal(presentation.remove, true);
  assert.equal(presentation.restart, false);
});

function row(status: OrganizationStatus, choiceError: string | null = null): string {
  notice.mockClear();
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
