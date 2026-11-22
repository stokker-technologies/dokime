/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { DokimeToaster } from "@/components/DokimeToaster";
import { failureOf } from "@/lib/notification-details";
import { StrictMode } from "react";
import { createRoot, type Root } from "react-dom/client";

/* === Local Imports === */

import { ErrorCard } from "@/components/ErrorCard";

import { BrandProvider, defaultBrand } from "@/lib/brand";
import { applyReportAppearance } from "./report-appearance";

import "./report.css";

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

async function start(): Promise<void> {
  const root = reportRoot();
  const text = document.getElementById("hub-report-data")?.textContent?.trim();
  if (!text) throw new Error("Report data is unavailable.");
  const [{ readReportCapture, materializeReport }, { ReportApp }] = await Promise.all([import("./native-report"), import("./ReportApp")]);
  const capture = readReportCapture();
  const { source, appearance, executionId } = await materializeReport(capture);
  const brand = applyReportAppearance(appearance);
  root.render(
    <StrictMode>
      <BrandProvider brand={brand}>
        <ReportApp source={source} executionId={executionId} />
      </BrandProvider>
    </StrictMode>,
  );
}

void start().catch((error: unknown) => {
  const root = reportRoot();
  root.render(
    <>
      <DokimeToaster />
      <ErrorCard failure={failureOf(error, "Report Could Not Open")} />
    </>,
  );
});
