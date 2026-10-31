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
import { DokimeToaster } from "@/components/DokimeToaster";
import { NotificationMessage } from "@/components/NotificationMessage";

/* === Local Imports === */

import { BrandProvider, defaultBrand } from "@/lib/brand";
import { applyReportAppearance } from "./report-appearance";

import "../styles.css";

/* === Constants === */

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

async function start(): Promise<void> {
  const text = document.getElementById("hub-report-data")?.textContent?.trim();
  if (!text) throw new Error("Report data is unavailable.");
  const [{ readReportPayload }, { ReportApp }] = await Promise.all([import("./report-data"), import("./ReportApp")]);
  const payload = readReportPayload();
  const brand = applyReportAppearance(payload.appearance);
  root.render(
    <StrictMode>
      <BrandProvider brand={brand}>
        <ReportApp payload={payload} />
      </BrandProvider>
    </StrictMode>,
  );
}

void start().catch((error: unknown) => {
  root.render(
    <>
      <DokimeToaster />
      <NotificationMessage source="Report" title="Report Could Not Open" error={error}>
        {error instanceof Error ? error.message : String(error)}
      </NotificationMessage>
    </>,
  );
});
