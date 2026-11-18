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
import { NotificationMessage } from "@/components/NotificationMessage";
import { failureOf } from "@/lib/notification-details";
import { StrictMode } from "react";
import { createRoot, type Root } from "react-dom/client";

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
  const [{ readReportCapture, materializeReport }, { ReportApp }] = await Promise.all([import("./native-report"), import("./ReportApp")]);
  const capture = readReportCapture();
  const { source, appearance } = await materializeReport(capture);
  const brand = applyReportAppearance(appearance);
  root.render(
    <StrictMode>
      <BrandProvider brand={brand}>
        <ReportApp source={source} />
      </BrandProvider>
    </StrictMode>,
  );
}

void start().catch((error: unknown) => {
  root.render(
    <>
      <DokimeToaster />
      <NotificationMessage context="background" source="Report" title="Report Could Not Open" error={error}>
        {failureOf(error, "Operation Failed").message}
      </NotificationMessage>
    </>,
  );
});
