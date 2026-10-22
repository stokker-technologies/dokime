/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Secret Settings Preserve Host Attribution And Never Fetch Stored Values.
 */

/* === Imports === */

// Standard Library Imports
import assert from "node:assert/strict";

// Third Party Imports
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { test, vi } from "vitest";

// Local Imports
import { SecretsSection } from "@/features/settings/sections/SecretsSection";
import { putSecret, readSecrets, removeSecret } from "@/lib/api/native-secrets";

/* === Re-Exports === */

/* === Constants === */

const fixture = vi.hoisted(() => ({
  identity: { generation: "remote-42", isCurrent: () => true },
  list: vi.fn(),
  put: vi.fn(),
  remove: vi.fn(),
}));

/* === Types === */

/* === Classes === */

/* === Public Functions === */

/* === Private Functions === */

/* === Entry Point === */

vi.mock("@/app/native-session/context", () => ({ useNativeSessionOptional: () => ({ identity: fixture.identity }) }));
vi.mock("@/lib/rpc/generated/client", () => ({ secretsList: fixture.list, secretsPut: fixture.put, secretsRemove: fixture.remove }));

test("Secret Settings Keep Host Metadata And Values Separate", async () => {
  const metadata = { name: "bench.token", label: "Bench Token", description: "Production bench" };
  const reply = {
    hostId: "host-remote-42",
    hostLabel: "Remote Bench",
    generation: 9n,
    cleanupPending: false,
    entries: [{ metadata, knownUses: ["obc.token"], value: "must-never-render" }],
  };
  fixture.list.mockResolvedValue(reply);
  fixture.put.mockResolvedValue(reply);
  fixture.remove.mockResolvedValue({ ...reply, entries: [], generation: 10n });
  const snapshot = await readSecrets(fixture.identity);
  const client = new QueryClient();
  client.setQueryData(["settings", "secrets", fixture.identity.generation], snapshot);
  const html = renderToStaticMarkup(createElement(QueryClientProvider, { client }, createElement(SecretsSection)));
  assert.ok(html.includes("Remote Bench"));
  assert.ok(html.includes("host-remote-42"));
  assert.ok(html.includes("bench.token"));
  assert.ok(html.includes("obc.token"));
  assert.ok(html.includes("••••••••"));
  assert.ok(!html.includes("must-never-render"));
  assert.deepEqual(fixture.list.mock.calls, [[fixture.identity, {}]]);

  await putSecret(fixture.identity, metadata, undefined, 9n);
  assert.deepEqual(fixture.put.mock.calls[0], [fixture.identity, { metadata, value: undefined, generation: 9n }]);
  await putSecret(fixture.identity, metadata, "replacement", 9n);
  assert.equal(fixture.put.mock.calls[1][1].value, "replacement");
  const removed = await removeSecret(fixture.identity, metadata.name, 9n);
  assert.deepEqual(removed.entries, []);
  assert.deepEqual(fixture.remove.mock.calls[0], [fixture.identity, { names: ["bench.token"], generation: 9n }]);
  client.clear();
});
