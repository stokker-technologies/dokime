// @vitest-environment happy-dom
/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Render Native Report Responses
 */

/* === Imports === */

// Standard Library Imports
import assert from "node:assert/strict";

// Third Party Imports
import { fireEvent, render } from "@testing-library/svelte";
import { create, fromBinary, toBinary } from "@bufbuild/protobuf";
import { FileDescriptorSetSchema } from "@bufbuild/protobuf/wkt";
import { onTestFinished, test, vi } from "vitest";
import { createRawSnippet } from "svelte";

// Local Imports
import ReportShell from "@/report/ReportShell.svelte";
import ReportCycleWorkspace from "@/report/ReportCycleWorkspace.svelte";
import { NativeIntegerValue } from "@/lib/api/native-values";
import { defaultBrand } from "@/lib/brand";
import { materializeReport, readReportCapture, reportOutcome } from "@/report/native-report";
import { applyReportAppearance } from "@/report/report-appearance";
import {
  BackendAppearanceResponseSchema,
  CaseStatus,
  CyclePhase,
  CycleStatus,
  DocumentCardKind,
  DocumentsGetResponseSchema,
  ExecutionFileKind,
  ExecutionFileState,
  ExecutionFilesGetResponseSchema,
  PlotCartesianMode,
  PlotKind,
  ExecutionState,
  FileReferenceSchema,
  LogoVariant,
  LogsAppendRequestSchema,
  RecordsListResponseSchema,
  ReportCaptureSchema,
  ReportFileSchema,
  ResultStatus,
  StepStatus,
} from "@generated/host/model";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Public Functions === */

/* === Private Functions === */

async function nativeReport() {
  const types = create(FileDescriptorSetSchema);
  const digest = new Uint8Array(await crypto.subtle.digest("SHA-256", toBinary(FileDescriptorSetSchema, types)));
  const scope = { cycleId: "cycle", executionId: "execution" };
  const metadata = { identifier: "value", label: "Value" };
  const exact = 9_007_199_254_740_993n;
  const counts = { cardCount: 1n, layoutNodeCount: 1n, bindingCount: 1n };
  return create(ReportCaptureSchema, {
    report: { testCycleId: scope.cycleId, testExecutionId: scope.executionId, generatedAt: { seconds: 1n } },
    cycle: {
      cycle: {
        cycleId: scope.cycleId,
        phase: CyclePhase.ACTIVE,
        status: CycleStatus.COMPLETED,
        updatedAt: { seconds: 1n },
        result: { status: ResultStatus.PASSED },
      },
    },
    appearance: { appearance: { appName: "Fixture" } },
    overview: { overview: { sessionId: "session" } },
    cases: [{ detail: { value: { summary: { caseId: "case", templateId: "template", status: CaseStatus.PASSED } } }, parameters: {} }],
    executions: [
      {
        detail: {
          execution: {
            cycleId: scope.cycleId,
            summary: {
              executionId: scope.executionId,
              caseId: "case",
              state: ExecutionState.SEALED,
              outcome: { attribution: { case: "evaluated", value: { status: ResultStatus.PASSED } } },
            },
            schemas: { digest },
          },
        },
        inputs: { parameters: [{ name: "exact", value: { kind: { case: "unsignedInteger", value: exact } } }] },
        outputs: {},
        steps: { steps: [{ stepId: "step", metadata, status: StepStatus.PASSED }] },
        plots: {},
        schemas: [{ digest, types }],
        document: {
          frames: [
            {
              frame: { kind: { case: "header", value: { scope, documentId: "default", metadata, frontiers: { processEpoch: "epoch" } } } },
            },
            { frame: { kind: { case: "card", value: { metadata, body: { kind: { case: "summary", value: { groupCount: 1n } } } } } } },
            { frame: { kind: { case: "groupDefinition", value: { cardId: "value", ordinal: 0n, tileCount: 1n, metadata } } } },
            {
              frame: {
                kind: {
                  case: "tileDefinition",
                  value: {
                    cardId: "value",
                    groupOrdinal: 0n,
                    ordinal: 0n,
                    metadata,
                    value: { kind: { case: "parameter", value: "exact" } },
                  },
                },
              },
            },
            {
              frame: {
                kind: { case: "binding", value: { sourceId: "value", metadata, result: { kind: { case: "pending", value: true } } } },
              },
            },
            {
              frame: {
                kind: { case: "layoutNode", value: { ordinal: 0n, node: { kind: { case: "placement", value: { cardId: "value" } } } } },
              },
            },
            { frame: { kind: { case: "structureComplete", value: { ...counts, frameCount: 7n } } } },
            { frame: { kind: { case: "evidenceCut", value: { cut: { store: "store", epoch: "execution", position: 7n } } } } },
            {
              frame: {
                kind: {
                  case: "binding",
                  value: { sourceId: "value", metadata, result: { kind: { case: "summary", value: { groupCount: 1n } } } },
                },
              },
            },
            { frame: { kind: { case: "summaryGroup", value: { cardId: "value", ordinal: 0n, tileCount: 1n, metadata } } } },
            {
              frame: {
                kind: {
                  case: "summaryTile",
                  value: {
                    cardId: "value",
                    groupOrdinal: 0n,
                    ordinal: 0n,
                    metadata,
                    value: { kind: { case: "unsignedInteger", value: exact } },
                  },
                },
              },
            },
            { frame: { kind: { case: "cardComplete", value: "value" } } },
            { frame: { kind: { case: "completion", value: { ...counts, frameCount: 13n } } } },
          ],
        },
      },
    ],
  });
}

/* === Entry Point === */

test("Embedded Protobuf Uses the Live Document and Value Decoders", async () => {
  const capture = await nativeReport();
  const encoded = Buffer.from(toBinary(ReportCaptureSchema, capture)).toString("base64");
  vi.stubGlobal("document", { getElementById: () => ({ textContent: encoded }) });
  try {
    const { source } = await materializeReport(readReportCapture());
    const value = source.document("execution")?.cards[0].groups[0].metrics[0].value;
    assert.ok(value instanceof NativeIntegerValue);
    assert.equal(value.decimal, "9007199254740993");
    assert.equal(source.executions[0].steps[0].test_execution_id, "execution");
    assert.equal(source.executions[0].steps[0].status, "passed");
    assert.deepEqual(source.executions[0].parameters?.exact, value);
    assert.equal(reportOutcome(source).variant, "success");
  } finally {
    vi.unstubAllGlobals();
  }
});

test("Execution Reports Keep Records Without a Case Tag", async () => {
  const capture = await nativeReport();
  capture.records.push(
    create(RecordsListResponseSchema, {
      records: [
        {
          recordId: "captured-record",
          recordType: "dokime.log",
          timestamp: { seconds: 1n },
          scope: { cycleId: "cycle", executionId: "execution", stepId: "step" },
          payload: toBinary(LogsAppendRequestSchema, create(LogsAppendRequestSchema, { message: "captured-value" })),
        },
      ],
    }),
  );
  const { source } = await materializeReport(capture);
  assert.equal(source.records.length, 1);
  const { container, unmount } = render(ReportCycleWorkspace, { source, selection: { executionId: "execution" } });
  onTestFinished(unmount);
  const records = container.querySelectorAll('[data-testid="record-rows"] [data-window-row]');
  assert.equal(records.length, 1);
});

test("Captured Plot Bindings Preserve Categories and Sample Instants", async () => {
  for (const plotType of [PlotKind.BAR, PlotKind.PIE, PlotKind.TRAJECTORY, PlotKind.HEAT_STRIP, PlotKind.XY]) {
    const capture = await nativeReport();
    const execution = capture.executions[0];
    const header = execution.document?.frames[0];
    assert.ok(header);
    const metadata = { identifier: "captured-plot", label: "Captured Plot" };
    const counts = { cardCount: 1n, layoutNodeCount: 1n, bindingCount: 1n };
    execution.document = create(DocumentsGetResponseSchema, {
      frames: [
        header,
        {
          frame: {
            kind: {
              case: "card",
              value: { metadata, body: { kind: { case: "owner", value: { kind: DocumentCardKind.PLOT, plotId: metadata.identifier } } } },
            },
          },
        },
        {
          frame: {
            kind: {
              case: "binding",
              value: { sourceId: metadata.identifier, metadata, result: { kind: { case: "pending", value: true } } },
            },
          },
        },
        {
          frame: { kind: { case: "layoutNode", value: { node: { kind: { case: "placement", value: { cardId: metadata.identifier } } } } } },
        },
        { frame: { kind: { case: "structureComplete", value: { ...counts, frameCount: 5n } } } },
        { frame: { kind: { case: "evidenceCut", value: { cut: { store: "store", epoch: "execution", position: 7n } } } } },
        {
          frame: {
            kind: {
              case: "binding",
              value: {
                sourceId: metadata.identifier,
                metadata,
                result: {
                  kind: {
                    case: "plot",
                    value: {
                      metadata,
                      request: {
                        plotId: metadata.identifier,
                        plotType,
                        stream: "samples",
                        ...(plotType === PlotKind.HEAT_STRIP
                          ? {
                              cellExpression: { kind: { case: "stream", value: { identifier: "samples", field: "cell" } } },
                              facetExpression: { kind: { case: "stream", value: { identifier: "samples", field: "facet" } } },
                            }
                          : {}),
                      },
                    },
                  },
                },
              },
            },
          },
        },
        { frame: { kind: { case: "cardComplete", value: metadata.identifier } } },
        { frame: { kind: { case: "completion", value: { ...counts, frameCount: 9n } } } },
      ],
    });
    const xy = plotType === PlotKind.XY;
    const coordinates = xy || plotType === PlotKind.TRAJECTORY;
    const timed = coordinates || plotType === PlotKind.HEAT_STRIP;
    const playback = xy || plotType === PlotKind.HEAT_STRIP;
    execution.plots = create(ReportCaptureSchema, {
      executions: [
        {
          plots: {
            envelopes: [
              {
                plotId: metadata.identifier,
                epoch: "epoch",
                selection: "selection",
                diagnostics: {},
                payload: {
                  stream: "samples",
                  series: [{ key: "value", rawPoints: 1n }],
                  frame: {
                    mode: PlotCartesianMode.FACETED,
                    pointCount: 1n,
                    series: [
                      {
                        x: [{ kind: coordinates ? { case: "number", value: 2 } : { case: "text", value: "category" } }],
                        y: [{ value: 3 }],
                        totals: [{ kind: { case: "unsignedInteger", value: 3n } }],
                        instants: timed ? ["1970-01-01T00:00:01Z"] : [],
                      },
                    ],
                  },
                },
              },
            ],
          },
        },
      ],
    }).executions[0].plots;
    if (playback) {
      const frame = execution.plots?.envelopes[0].payload?.frame;
      assert.ok(frame);
      frame.pointCount = 2n;
      frame.series[0].x.push(frame.series[0].x[0]);
      frame.series[0].y.push({ ...frame.series[0].y[0], value: 9 });
      frame.series[0].instants.push("1970-01-01T00:00:02Z");
      frame.series[0].totals.push({ ...frame.series[0].totals[0], kind: { case: "unsignedInteger", value: 9n } });
    }
    const { source } = await materializeReport(capture);
    if (plotType === PlotKind.HEAT_STRIP) {
      const declaration = source.document("execution")?.cards[0].plots[0];
      assert.equal(declaration?.plot_type, "heat_strip");
      assert.equal(declaration?.cell_expression?.kind.case, "stream");
      assert.equal(declaration?.facet_expression?.kind.case, "stream");
    }
    const payload = source.plotPayloads("execution").plots[metadata.identifier];
    if (xy) {
      assert.deepEqual(payload.cartesian?.series[0], { x: [2, 2], y: [3, 9], instants: ["1970-01-01T00:00:01Z", "1970-01-01T00:00:02Z"] });
    } else {
      assert.equal(payload.cartesian, null);
      const coordinate = payload.series[0].points[0].x;
      assert.equal(coordinates ? Number(coordinate) : coordinate, coordinates ? 2 : "category");
      assert.equal(payload.series[0].points[0].y, 3);
      assert.equal(payload.series[0].points[0].instant, timed ? "1970-01-01T00:00:01Z" : null);
    }
    if (playback) {
      const { container: host, unmount } = render(ReportCycleWorkspace, { source, selection: { executionId: "execution" } });
      onTestFinished(unmount);
      assert.ok(xy || host.querySelector<HTMLElement>("[data-cell]")?.title.includes(" · 9 · "));
      const slider = host.querySelector<HTMLElement>('[role="slider"]');
      assert.ok(slider);
      assert.equal(slider.getAttribute("aria-valuenow"), "1");
      await fireEvent.keyDown(slider, { key: "ArrowLeft" });
      assert.equal(host.querySelector("[data-playback-instant]")?.getAttribute("data-playback-instant"), "1970-01-01T00:00:01Z");
      if (!xy) {
        assert.ok(host.querySelector<HTMLElement>("[data-cell]")?.title.includes(" · 3 · "));
        assert.equal(source.plotPayloads("execution").plots[metadata.identifier].series[0].points[1].y, 9);
      }
    }
  }
});

test("Incomplete Embedded Documents Are Refused by the Live Assembler", async () => {
  const capture = await nativeReport();
  capture.executions[0].document?.frames.pop();
  await assert.rejects(materializeReport(capture));
});

test("Cycle Reports Retain Their Own Document and Execution Documents", async () => {
  const capture = await nativeReport();
  assert.ok(capture.report);
  capture.report.testExecutionId = undefined;
  capture.document = fromBinary(ReportCaptureSchema, toBinary(ReportCaptureSchema, capture)).executions[0].document;
  const first = capture.document?.frames[0].frame?.kind;
  assert.ok(first);
  assert.equal(first.case, "header");
  if (first.case === "header") {
    assert.ok(first.value.scope);
    first.value.scope.executionId = "";
  }
  const { source } = await materializeReport(capture);
  assert.ok(source.document(null));
  assert.ok(source.document("execution"));
  assert.equal(source.traceability(null)?.scope?.cycleId, "cycle");
});

test("Embedded Descriptor Bytes Must Match the Frozen Digest", async () => {
  const capture = await nativeReport();
  const schemas = capture.executions[0].detail?.execution?.schemas;
  assert.ok(schemas);
  schemas.digest[0] ^= 1;
  await assert.rejects(materializeReport(capture));
});

test("Embedded File Bytes Retain Their Resource Media Type and Digest", async () => {
  const capture = await nativeReport();
  const data = new Uint8Array([1, 2, 3]);
  const sha256 = new Uint8Array(await crypto.subtle.digest("SHA-256", data));
  const file = create(FileReferenceSchema, { length: 3n, sha256 });
  const captured = create(ReportFileSchema, {
    file,
    reads: [{ data, totalLength: 3n, sha256 }],
  });
  capture.files.push(captured);
  const metadata = {
    relativePath: "frame.png",
    contentType: "image/png",
    kind: ExecutionFileKind.MEDIA,
    state: ExecutionFileState.FINALIZED,
    sizeBytes: 3n,
  };
  capture.executions[0].files.push(create(ExecutionFilesGetResponseSchema, { metadata, file }));
  const report = await materializeReport(capture);
  assert.equal(report.source.executionFiles("execution")[0].href, "data:image/png;base64,AQID");
  data[0] ^= 1;
  await assert.rejects(materializeReport(capture));
});

test("Embedded Organization Appearance Styles the Report in Both Schemes", async () => {
  const capture = await nativeReport();
  const css = '@font-face{font-family:"Example Sans";src:url("data:font/woff2;base64,AAEC")}';
  capture.appearance = create(BackendAppearanceResponseSchema, {
    appearance: {
      appName: "Example Lab",
      logos: [{ variant: LogoVariant.TRANSPARENT_ON_LIGHT, dataUri: "data:image/svg+xml;base64,PHN2Zy8+" }],
      stylesheet: css,
    },
  });
  const { appearance } = await materializeReport(capture);
  const originalTitle = document.title;
  onTestFinished(() => {
    document.title = originalTitle;
    vi.unstubAllGlobals();
  });
  for (const dark of [false, true]) {
    vi.stubGlobal("matchMedia", () => ({ matches: dark }));
    const brand = applyReportAppearance(appearance);
    assert.equal(brand.label, "Example Lab");
    assert.equal(brand.logos.transparent_on_light?.src, "data:image/svg+xml;base64,PHN2Zy8+");
    assert.equal(document.getElementById("dokime-host-theme")?.textContent, css);
    assert.equal(document.documentElement.classList.contains("dark"), dark);
  }
  assert.equal(document.title, "Example Lab Report");
});

test("Report Shell Navigation Follows the Subject Even with One Execution", async () => {
  const capture = await nativeReport();
  for (const executionSubject of [true, false]) {
    assert.ok(capture.report);
    capture.report.testExecutionId = executionSubject ? "execution" : undefined;
    const { source, executionId } = await materializeReport(capture);
    assert.equal(source.executions.length, 1);
    assert.equal(executionId, executionSubject ? "execution" : null);
    const children = createRawSnippet(() => ({ render: () => '<output data-execution="execution"></output>' }));
    const { container, unmount } = render(ReportShell, {
      brand: defaultBrand,
      source,
      selection: { executionId: "execution" },
      onSelect: executionId === null ? () => undefined : undefined,
      children,
    });
    onTestFinished(unmount);
    assert.equal(Boolean(container.querySelector("nav")), !executionSubject);
    assert.ok(container.querySelector('[data-execution="execution"]'));
    assert.equal(Boolean(container.querySelector('[data-slot="sidebar-brand"]')), !executionSubject);
  }
});
