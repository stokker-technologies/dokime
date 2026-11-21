// Dokime — SPDX-License-Identifier: Apache-2.0
import { ErrorCard } from "@dokime/plots/components/ErrorCard";
import { failureOf } from "@dokime/plots/lib/notification-details";
import { createRoot, type Root } from "react-dom/client";
import { DokimeToaster } from "@dokime/document/components/DokimeToaster";
import { defaultBrand } from "@dokime/document/lib/brand";
import "./desktop.css";

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
      <ErrorCard notificationContext="background" failure={failureOf(error, "Application Could Not Start")} />
    </>,
  );
});
