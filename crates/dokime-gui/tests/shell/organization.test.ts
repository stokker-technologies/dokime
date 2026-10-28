/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Brand Surface Selection And Resource Ownership
 */

import assert from "node:assert/strict";
import { createHash } from "node:crypto";

import { test, vi } from "vitest";

import { brandFromLogos, selectLogo } from "@/lib/brand";
import { loadAppearance } from "@/lib/rpc/appearance";

const calls = vi.hoisted(() => ({ appearance: vi.fn(), download: vi.fn() }));
vi.mock("@/lib/rpc/generated/client.js", () => ({
  appearanceGetAppearance: calls.appearance,
  contentDownloadAppearanceAsset: calls.download,
}));

test("Brand Asset Lifecycle", async () => {
  const categories = ["TransparentOnLight", "TransparentOnDark", "ContainedOnLight", "ContainedOnDark"];
  const bytes = new TextEncoder().encode("<svg/>");
  const sha256 = createHash("sha256").update(bytes).digest("hex");
  const logos = categories.map((category) => ({
    variant: { HarmosVariant: { case: category } },
    asset: { reference: { pluginId: "brand", assetId: category, sha256 }, contentType: "image/svg+xml", sizeBytes: BigInt(bytes.length) },
    alt: "Stokker Technologies",
  }));
  calls.appearance.mockResolvedValue({ logos });
  let failAsset: string | null = null;
  let current = true;
  let supersedeAsset: string | null = null;
  const cancelled: string[] = [];
  calls.download.mockImplementation(async (_session, { reference }) => {
    const metadata = logos.find((logo) => logo.asset.reference.assetId === reference.assetId)?.asset;
    assert.ok(metadata);
    return {
      async *[Symbol.asyncIterator]() {
        yield { payload: { HarmosVariant: { case: "Metadata", value: { field0: metadata } } } };
        if (reference.assetId === failAsset) throw new Error("transfer failed");
        yield { payload: { HarmosVariant: { case: "Chunk", value: { field0: { offset: 0n, data: bytes } } } } };
        yield { payload: { HarmosVariant: { case: "Complete", value: { field0: { sizeBytes: BigInt(bytes.length), sha256 } } } } };
        if (reference.assetId === supersedeAsset) current = false;
      },
      async cancel() {
        cancelled.push(reference.assetId);
      },
    };
  });
  const revoked = vi.spyOn(URL, "revokeObjectURL");
  try {
    const session = { generation: "1", isCurrent: () => current };
    const resource = await loadAppearance(session);
    assert.equal(resource.logos.length, 4);
    for (const logo of resource.logos) assert.equal(await (await fetch(logo.url)).text(), "<svg/>");
    const brand = brandFromLogos("Stokker Technologies", resource.logos);
    assert.equal(selectLogo(brand, "transparent", "light")?.src, resource.logos[0].url);
    assert.equal(selectLogo(brand, "transparent", "dark")?.src, resource.logos[1].url);
    assert.equal(selectLogo(brand, "contained", "dark")?.src, resource.logos[3].url);
    const partial = brandFromLogos("Stokker Technologies", [resource.logos[2]]);
    assert.equal(selectLogo(partial, "transparent", "light")?.src, resource.logos[2].url);
    assert.equal(selectLogo(partial, "transparent", "dark"), undefined);
    resource.dispose();
    resource.dispose();
    assert.equal(revoked.mock.calls.length, 4);
    for (const logo of resource.logos) await assert.rejects(fetch(logo.url));

    failAsset = "TransparentOnDark";
    await assert.rejects(loadAppearance(session), /transfer failed/);
    assert.equal(revoked.mock.calls.length, 5);
    assert.deepEqual(cancelled.slice(-2), ["TransparentOnLight", "TransparentOnDark"]);

    failAsset = null;
    supersedeAsset = "TransparentOnDark";
    await assert.rejects(loadAppearance(session), (error: { kind: string }) => error.kind === "session");
    assert.equal(revoked.mock.calls.length, 6);
  } finally {
    revoked.mockRestore();
  }
});
