/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { StrictMode } from "react";
import { createRoot, type Root } from "react-dom/client";

/* === Local Imports === */

import { BrandProvider, defaultBrand } from "@/lib/brand";
import { ReportApp } from "./ReportApp";
import { applyReportAppearance } from "./report-appearance";
import { readReportPayload } from "./report-data";
import "../styles.css";

/* === Constants === */

const payload = readReportPayload();
const brand = applyReportAppearance(payload.appearance);

declare global {
  interface Window {
    __dokimeReportRoot__?: Root;
  }
}

function reportRoot(): Root {
  if (window.__dokimeReportRoot__) {
    return window.__dokimeReportRoot__;
  }

  const rootElement = document.getElementById("root");
  if (!rootElement) {
    throw new Error(`Missing ${defaultBrand.label} report root element.`);
  }

  window.__dokimeReportRoot__ = createRoot(rootElement);
  return window.__dokimeReportRoot__;
}

const root = reportRoot();

root.render(
  <StrictMode>
    <BrandProvider brand={brand}>
      <ReportApp payload={payload} />
    </BrandProvider>
  </StrictMode>,
);
