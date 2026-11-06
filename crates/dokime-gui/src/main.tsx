import { failureOf } from "@/lib/notification-details";
// Dokime — SPDX-License-Identifier: Apache-2.0

import { createRoot, type Root } from "react-dom/client";
import { DokimeToaster } from "@/components/DokimeToaster";
import { NotificationMessage } from "@/components/NotificationMessage";
import { defaultBrand } from "@/lib/brand";
import "@/styles.css";

declare global {
  interface Window {
    __dokimeLiveRoot__?: Root;
  }
}
const element = document.getElementById("root");
if (!element) throw new Error(`Missing ${defaultBrand.label} root element.`);
const root = window.__dokimeLiveRoot__ ?? createRoot(element);
window.__dokimeLiveRoot__ = root;
void (async () => {
  const { AppRoot } = await import("@/app/AppRoot");
  root.render(<AppRoot />);
})().catch((error: unknown) => {
  root.render(
    <>
      <DokimeToaster />
      <NotificationMessage context="background" source="Startup" title="Application Could Not Start" error={error}>
        {failureOf(error, "Operation Failed").message}
      </NotificationMessage>
    </>,
  );
});
