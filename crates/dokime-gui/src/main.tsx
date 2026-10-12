// Dokime — SPDX-License-Identifier: Apache-2.0
import { createRoot, type Root } from "react-dom/client";
import { AppRoot } from "@/app/AppRoot";
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
root.render(<AppRoot />);
