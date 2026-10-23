/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Sidebar Brand Block Layout
 */

/* === Imports === */

// Standard Library Imports
import assert from "node:assert/strict";

// Third Party Imports
import { type ComponentProps, createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { test } from "vitest";

// Local Imports
import { type Brand, BrandProvider } from "@/lib/brand";
import { SidebarBrand } from "@/shell/SidebarFrame";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Public Functions === */

/* === Private Functions === */

function render(brand: Brand): string {
  return renderToStaticMarkup(
    createElement(BrandProvider, { brand } as ComponentProps<typeof BrandProvider>, createElement(SidebarBrand, { collapsed: false })),
  );
}

/* === Entry Point === */

test("the open sidebar sizes its logo to the window and centres the wordmark beneath", () => {
  const html = render({
    label: "Dokime",
    logos: {
      transparent_on_light: { src: "light.svg", alt: "Dokime" },
      transparent_on_dark: { src: "dark.svg", alt: "Dokime" },
    },
  });
  assert.equal(html.match(/h-\[var\(--sidebar-logo-height\)\]/g)?.length, 2);
  assert.ok(html.indexOf("dark.svg") < html.indexOf(">Dokime</span>"));
  assert.match(html, /flex-col items-center/);
  assert.match(html, /text-center/);
});

test("without artwork the wordmark alone stands for the brand", () => {
  const html = render({ label: "Dokime", logos: {} });
  assert.equal(html.match(/>Dokime</g)?.length, 1);
  assert.doesNotMatch(html, /sidebar-logo-height/);
});
