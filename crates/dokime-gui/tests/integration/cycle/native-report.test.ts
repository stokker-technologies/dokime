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
import { create, fromBinary, toBinary } from "@bufbuild/protobuf";
import { FileDescriptorSetSchema } from "@bufbuild/protobuf/wkt";
import { test, vi } from "vitest";

// Local Imports
import { NativeIntegerValue } from "@/lib/api/native-values";
import { materializeReport, readReportCapture, reportOutcome } from "@/report/native-report";
import {
  AppearanceLogoSchema,
  CaseStatus,
  CyclePhase,
  CycleStatus,
  ExecutionState,
  FileReferenceSchema,
  LogoVariant,
  ReportCaptureSchema,
  ReportFileSchema,
  ResultStatus,
  StepStatus,
} from "@host/model";

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
  const logo = create(AppearanceLogoSchema, {
    variant: LogoVariant.TRANSPARENT_ON_LIGHT,
    asset: {
      reference: { pluginId: "provider", assetId: "logo", sha256: "" },
      sizeBytes: 3n,
      contentType: "image/png",
      file,
    },
  });
  const appearance = capture.appearance?.appearance;
  assert.ok(appearance);
  appearance.logos.push(logo);
  const report = await materializeReport(capture);
  assert.equal(report.appearance.logos[0].url, "data:image/png;base64,AQID");
  data[0] ^= 1;
  await assert.rejects(materializeReport(capture));
});

/* === Entry Point === */
