// Dokime — SPDX-License-Identifier: Apache-2.0
import { mount } from "svelte";
import { failureOf } from "../lib/notification-details";
import { materializeReport, readReportCapture } from "./native-report";
import { applyReportAppearance } from "./report-appearance";
import ReportApp from "./ReportApp.svelte";

import "./report.css";

async function start() {
  const target = document.getElementById("root");
  if (!target) {
    throw new Error("Open the report from the original HTML export.");
  }
  try {
    const capture = readReportCapture();
    const { source, appearance, executionId } = await materializeReport(capture);
    const brand = applyReportAppearance(appearance);
    mount(ReportApp, { target, props: { brand, source, executionId } });
  } catch (error: unknown) {
    mount(ReportApp, { target, props: { failure: failureOf(error, "Report Could Not Open") } });
  }
}

void start();
