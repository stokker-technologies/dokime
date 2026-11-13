/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

// Standard Library Imports
import assert from "node:assert/strict";

// Third Party Imports
import { QueryClient, QueryObserver } from "@tanstack/react-query";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { beforeEach, test, vi } from "vitest";
import { ExecutionMode, PromptKind, ExecutionState, ResultStatus, LiveExecutionState } from "@host/dokime/v1/client_types_pb.js";
import { create, createFileRegistry, fromBinary, toBinary } from "@bufbuild/protobuf";
import { PromptFieldKind, StreamMatchOperator } from "@host/dokime/v1/host_pb.js";
import { FileDescriptorSetSchema, FieldDescriptorProto_Type } from "@bufbuild/protobuf/wkt";
import { fetchNativeArtifactBytes } from "@/lib/api/native-content";
import { fetchNativeDocument } from "@/lib/api/native-documents";
import { sha256Hex } from "@/lib/api/native-values";
import { ArtifactState } from "@host/dokime/v1/client_types_pb.js";
import { RecordContractsSchema } from "@host/dokime/v1/catalog_pb.js";
import { fetchNativeRecordTail, openNativeRecordStream } from "@/lib/api/native-evidence";
import { Severity } from "@host/dokime/v1/execution_pb.js";
import { previewNativeTestCycle, projectExecutionStatus } from "@/lib/api/native-planning";
import { fetchNativePendingPrompts, respondNativePrompt } from "@/lib/api/native-prompts";
import { openNativeRuntimeStream, openNativeExecutionProgressStream } from "@/lib/api/native-observation";
import { jsonToNativeValue, nativeValueToJson } from "@/lib/host/values";
import { NativeIntegerValue, NativeNumberValue } from "@/lib/api/native-values";

// Local Imports
import { ControlCard } from "@/features/cycle/ControlCard";
import { cycleControlState, executionControlState } from "@/features/cycle/control-model";
import { invalidateRuntimeDocuments, invalidateRuntimeProjectionQueries, workflowQueryKeys } from "@/features/cycle/server-state";
import {
  appendNativeCaseAttempt,
  fetchNativeTestCycleSteps,
  sealNativeTestCycle,
  startNativeTestExecution,
  stopNativeTestExecution,
} from "@/lib/api/native-execution";

/* === Re-Exports === */

/* === Constants === */

const fixture = vi.hoisted(() => ({
  unary: vi.fn(),
  stream: vi.fn(),
  extra: vi.fn(),
  append: vi.fn(),
  start: vi.fn(),
  stop: vi.fn(),
  seal: vi.fn(),
  detail: vi.fn(),
  parameters: vi.fn(),
  steps: vi.fn(),
}));

const session = { generation: "lifecycle", isCurrent: () => true };

/* === Types === */

/* === Classes === */

/* === Public Functions === */

/* === Private Functions === */

function accepted() {
  return { outcome: { kind: { case: "accepted", value: { commandId: "execution", jobId: null } } } };
}

/* === Entry Point === */

vi.mock("@/lib/rpc/transport", () => ({ unary: fixture.unary, openStream: fixture.stream }));

beforeEach(() => {
  vi.clearAllMocks();
  fixture.extra.mockReset();
  fixture.stream.mockReset();
  fixture.unary.mockImplementation(async (path, input, output, identity, request) => {
    const decoded = fromBinary(input, toBinary(input, create(input, request)));
    const operations: Record<string, typeof fixture.append> = {
      "/dokime.v1.Execution/AppendExecutions": fixture.append,
      "/dokime.v1.Execution/StartExecution": fixture.start,
      "/dokime.v1.Execution/StopExecution": fixture.stop,
      "/dokime.v1.Execution/SealCycle": fixture.seal,
      "/dokime.v1.Execution/GetExecution": fixture.detail,
      "/dokime.v1.Execution/ListExecutionParameters": fixture.parameters,
      "/dokime.v1.Execution/ListExecutionSteps": fixture.steps,
    };
    const operation = operations[path];
    const reply = operation ? await operation(identity, request) : await fixture.extra(path, decoded);
    return fromBinary(output, toBinary(output, create(output, reply)));
  });
});

test("Execution Detail Uses Embedded Criteria", async () => {
  fixture.detail.mockResolvedValue({ cycleId: "cycle", summary: { executionId: "execution", caseId: "case" } });
  fixture.parameters.mockResolvedValue({ parameters: [] });
  const steps = Array.from({ length: 20 }, (_, ordinal) => ({
    stepId: `step-${ordinal}`,
    ordinal: BigInt(ordinal),
    measurementCount: 0n,
    criterionCount: 1n,
    criteria: [{ criterionId: `criterion-${ordinal}`, ordinal: 0n }],
  }));
  fixture.steps.mockResolvedValue({ steps });

  const records = await fetchNativeTestCycleSteps(session, { cycleId: "cycle", executionId: "execution" });
  assert.equal(records.length, 40);
  assert.equal(records.filter((record) => record.record_type === "criterion").length, 20);
  assert.equal(records[39].step_id, "criterion-19");
  assert.equal(fixture.detail.mock.calls.length, 1);
  assert.equal(fixture.parameters.mock.calls.length, 2);
  assert.equal(fixture.steps.mock.calls.length, 1);
});

test("Append Without Start", async () => {
  fixture.append.mockResolvedValue({});
  const first = await appendNativeCaseAttempt(session, "cycle", "case");
  const second = await appendNativeCaseAttempt(session, "cycle", "case");
  assert.notEqual(first, second);
  assert.match(first, /^[0-9a-f-]{36}$/);
  assert.deepEqual(fixture.append.mock.calls, [
    [session, { cycleId: "cycle", executions: [{ executionId: first, caseId: "case" }] }],
    [session, { cycleId: "cycle", executions: [{ executionId: second, caseId: "case" }] }],
  ]);
  assert.equal(fixture.start.mock.calls.length, 0);
});

test("Start Reuses Identity", async () => {
  fixture.start.mockResolvedValue(accepted());
  await startNativeTestExecution(session, "execution");
  await startNativeTestExecution(session, "execution");
  const request = { executionId: "execution", mode: ExecutionMode.AUTOMATIC };
  assert.deepEqual(fixture.start.mock.calls, [
    [session, request],
    [session, request],
  ]);
});

test("Start Admission Refusal", async () => {
  fixture.start.mockResolvedValue({
    outcome: {
      kind: {
        case: "rejected",
        value: {
          code: "cycle_sealed",
          message: "Cycle Is Sealed",
          issues: [{ path: "execution_id", message: "Admission Closed" }],
        },
      },
    },
  });
  await assert.rejects(startNativeTestExecution(session, "execution"), /cycle_sealed: Cycle Is Sealed.*execution_id: Admission Closed/);
});

test("Durable Stop Seal", async () => {
  let finishStop!: () => void;
  fixture.stop.mockImplementation(
    () =>
      new Promise<void>((resolve) => {
        finishStop = resolve;
      }),
  );
  let stopped = false;
  const stop = stopNativeTestExecution(session, "execution").then(() => {
    stopped = true;
  });
  await Promise.resolve();
  assert.equal(stopped, false);
  finishStop();
  await stop;
  assert.equal(stopped, true);
  assert.deepEqual(fixture.stop.mock.calls, [[session, { executionId: "execution" }]]);

  let failSeal!: (error: Error) => void;
  fixture.seal.mockImplementation(
    () =>
      new Promise<void>((_resolve, reject) => {
        failSeal = reject;
      }),
  );
  const seal = sealNativeTestCycle(session, "cycle");
  const rejected = assert.rejects(seal, /Persistence Failed/);
  failSeal(new Error("Persistence Failed"));
  await rejected;
  assert.deepEqual(fixture.seal.mock.calls, [[session, { cycleId: "cycle" }]]);
});

test("Container Seal Controls", () => {
  // An Active Cycle Offers Seal Cycle Alone, Whatever Its Members Did
  for (const running of [null, { ordinal: 2, step: "Energize Charger" }]) {
    const state = cycleControlState({ phase: "active", running });
    assert.deepEqual(state.actions, ["Seal Cycle"]);
    const html = renderToStaticMarkup(
      createElement(ControlCard, {
        actions: createElement("button", { type: "button" }, state.actions[0]),
        label: "Cycle Controls",
        state: state.line,
      }),
    );
    assert.match(html, /Seal Cycle/);
    assert.equal((html.match(/<button/g) ?? []).length, 1);
    assert.doesNotMatch(html, /Start Cycle|Stop Cycle|Run Again|Pause Cycle|Resume Cycle/);
  }
  assert.equal(cycleControlState({ phase: "active", running: null }).line, "Idle · no execution running");
  assert.equal(
    cycleControlState({ phase: "active", running: { ordinal: 2, step: "Energize Charger" } }).line,
    "Execution 2 · Energize Charger",
  );

  // Sealing and Sealed Offer Nothing
  assert.deepEqual(cycleControlState({ phase: "sealing", running: null }), { actions: [], line: "Sealing · saving evidence" });
  assert.deepEqual(cycleControlState({ phase: "sealed", running: null }), { actions: [], line: "Sealed · evidence saved" });

  // A Failed Seal Offers Retry and Archive, Never a Running Control
  assert.deepEqual(cycleControlState({ phase: "seal_failed", running: null }).actions, ["Retry Seal", "Archive"]);
});

test("Finished Execution Without Its Plugin Has No New Attempt Button", () => {
  const state = executionControlState({
    controls: null,
    pluginAvailable: false,
    progress: null,
    promptWaiting: false,
    status: "passed",
    stopping: false,
  });
  const html = renderToStaticMarkup(
    createElement(ControlCard, {
      actions: state.actions.map((action) => createElement("button", { key: action, type: "button" }, action)),
      detail: state.detail,
      label: "Execution Controls",
      state: state.line,
    }),
  );
  assert.equal(state.phase, "ended");
  assert.equal((html.match(/<button/g) ?? []).length, 0);
  assert.equal((html.match(/data-control-state/g) ?? []).length, 1);
  assert.ok(state.detail);
});

test("Stream Refreshes Documents And Preserves Sealed Detail", async () => {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false, staleTime: Infinity } } });
  const calls = { document: 0, sealed: 0, active: 0 };
  const keys = {
    document: workflowQueryKeys.executionDocument({ cycleId: "cycle", executionId: "active" }),
    sealed: workflowQueryKeys.executionDetail("sealed"),
    active: workflowQueryKeys.executionDetail("active"),
  };
  const observers = Object.entries(keys).map(([name, queryKey]) => {
    const kind = name as keyof typeof calls;
    return new QueryObserver(client, {
      queryKey,
      queryFn: async () => ({ revision: ++calls[kind], sealed: kind === "sealed" }),
    });
  });
  const release = observers.map((observer) => observer.subscribe(() => {}));
  try {
    await Promise.all(observers.map((observer) => observer.refetch()));
    assert.deepEqual(calls, { document: 1, sealed: 1, active: 1 });

    await invalidateRuntimeDocuments(client);
    assert.deepEqual(calls, { document: 2, sealed: 1, active: 1 });
    await invalidateRuntimeProjectionQueries(client);
    assert.deepEqual(calls, { document: 2, sealed: 1, active: 2 });
  } finally {
    for (const unsubscribe of release) unsubscribe();
    client.clear();
  }
});

test("Planning Preserves Selection Order and Exact Recursive Parameters", async () => {
  const requests: Record<string, unknown>[] = [];
  fixture.extra.mockImplementation(async (path, request) => {
    if (path === "/dokime.v1.Catalog/ListSuites") return { suites: [{ suiteId: "fixture", source: "fixture" }] };
    if (path === "/dokime.v1.Catalog/GetSuite")
      return {
        metadata: { identifier: "fixture", label: "Fixture" },
        templates: [{ metadata: { identifier: "fixture.run", label: "Run" } }],
      };
    assert.equal(path, "/dokime.v1.Planning/PreviewSelection");
    requests.push(request);
    return {
      cases: [
        {
          caseId: "one",
          suiteId: "fixture",
          templateId: "fixture.run",
          ordinal: 0n,
          origin: { kind: { case: "pluginRun", value: { testCase: { parameters: { count: jsonToNativeValue(18446744073709551615n) } } } } },
        },
        { caseId: "two", suiteId: "fixture", templateId: "fixture.run", ordinal: 1n },
      ],
    };
  });
  const selection = {
    suite_id: "fixture",
    template_id: "fixture.run",
    parameters: { count: [18446744073709551615n], samples: [[1, new NativeNumberValue(-0)]] },
    files: { calibration: null },
  };
  const preview = await previewNativeTestCycle(session, { cycle_id: "cycle", parameters: {}, selections: [selection, selection] });
  assert.deepEqual(
    preview.cases.map((item) => item.case_id),
    ["one", "two"],
  );
  assert.equal(String(preview.cases[0].parameters.count), "18446744073709551615");
  const request = requests[0] as {
    plan: {
      cycleId: string;
      selections: {
        files: { value: { kind: { kind: { case: string } } } }[];
        parameters: { key: string; value: { values: Parameters<typeof nativeValueToJson>[0][] } }[];
      }[];
    };
  };
  assert.equal(request.plan.cycleId, "cycle");
  assert.equal(request.plan.selections.length, 2);
  assert.deepEqual(request.plan.selections[0], request.plan.selections[1]);
  assert.equal(request.plan.selections[0].files[0].value.kind.kind.case, "clear");
  const sampleParameter = request.plan.selections[0].parameters.find((item) => item.key === "samples");
  assert.ok(sampleParameter);
  const samples = nativeValueToJson(sampleParameter.value.values[0]) as unknown[];
  assert.equal(samples[0], 1);
  assert.ok(Object.is(Number(samples[1]), -0));
});

test("Prompt Read and Answer Keep Canonical Fields and Unsigned Values", async () => {
  const replies: unknown[] = [];
  fixture.extra.mockImplementation(async (path, request) => {
    if (path === "/dokime.v1.Prompts/ListPending") {
      assert.equal(request.cycleId, "cycle");
      return { prompts: [{ promptId: "prompt" }] };
    }
    if (path === "/dokime.v1.Prompts/GetPrompt")
      return {
        prompt: {
          promptId: "prompt",
          kind: PromptKind.FORM,
          createdAt: { seconds: 7n },
          scope: { cycleId: "cycle", executionId: "execution" },
          definition: {
            metadata: { identifier: "fixture.answer", label: "Answer" },
            level: Severity.WARNING,
            template: "fixture.run",
            fields: [
              {
                metadata: { identifier: "count", label: "Count" },
                kind: PromptFieldKind.INTEGER,
                defaultValue: jsonToNativeValue(18446744073709551615n),
              },
            ],
            actions: [{ identifier: "confirm", label: "Confirm", requiresCondition: true }],
            condition: {
              kind: {
                case: "all",
                value: {
                  conditions: [
                    { kind: { case: "stream", value: { stream: "temperature", field: "value", operator: StreamMatchOperator.EXISTS } } },
                  ],
                },
              },
            },
          },
        },
      };
    assert.equal(path, "/dokime.v1.Prompts/Respond");
    replies.push(request);
    assert.equal(request.values.values.count.kind.case, "unsignedInteger");
    assert.equal(request.values.values.count.kind.value, 18446744073709551615n);
    assert.equal(request.action, "confirm");
    assert.equal(request.accepted, true);
    return { commandId: "answer", promptId: "prompt", committedRevision: 9007199254740993n, acceptedAt: { seconds: 8n } };
  });
  const [prompt] = await fetchNativePendingPrompts(session, { cycleId: "cycle" });
  assert.equal(prompt.kind, "form");
  assert.equal(prompt.level, "warning");
  assert.equal(prompt.fields[0].kind, "int");
  assert.equal(prompt.actions[0].id, "confirm");
  assert.equal(prompt.scope?.test_execution_id, "execution");
  assert.equal(prompt.verification?.condition.kind, "host");
  const receipt = await respondNativePrompt(session, prompt.correlation_id, "confirm", { count: prompt.fields[0].default });
  assert.equal(receipt.result?.committed_revision, "9007199254740993");
  assert.equal(replies.length, 1);
});

test("Watch Resynchronization and Cancellation Stay with Their Generation", async () => {
  const canceled = vi.fn(async () => {});
  const owned: (() => void)[] = [];
  const release = vi.fn();
  const observation = {
    ...session,
    own: (dispose: () => void) => {
      owned.push(dispose);
      return release;
    },
  };
  fixture.stream.mockImplementation(async (path, input, output, identity, request, envelope) => {
    assert.equal(identity, observation);
    if (path.endsWith("WatchRuntime")) assert.ok(envelope.typeName.endsWith("Output"));
    else assert.equal(envelope, undefined);
    const decoded = fromBinary(input, toBinary(input, create(input, request)));
    const frame = path.endsWith("WatchRuntime")
      ? { resynchronized: true, cursor: { epoch: "host", stream: "runtime", position: 9n } }
      : { finished: true };
    if (path === "/dokime.v1.executions.Progress/Watch") {
      assert.ok("name" in decoded);
      assert.equal(decoded.name, "execution");
    }
    return {
      cancel: canceled,
      async *[Symbol.asyncIterator]() {
        yield fromBinary(output, toBinary(output, create(output, frame)));
      },
    };
  });
  const runtime = await openNativeRuntimeStream(observation);
  for await (const frame of runtime) {
    assert.equal(frame.resynchronized, true);
    assert.equal(frame.cursor?.position, 9n);
    break;
  }
  assert.equal(canceled.mock.calls.length, 1);
  assert.equal(release.mock.calls.length, 1);
  const progress = await openNativeExecutionProgressStream(observation, "execution");
  const frames = [];
  for await (const frame of progress) frames.push(frame);
  assert.equal(frames[0].progress, null);
  assert.equal(frames[0].finished, true);
  assert.equal(release.mock.calls.length, 2);
  await progress.cancel();
  for (const dispose of owned) dispose();
  assert.equal(canceled.mock.calls.length, 1);
});

test("Imported and Unevaluated Outcomes Never Gain Run Capabilities", () => {
  assert.equal(
    projectExecutionStatus(ExecutionState.CLOSED, {
      attribution: { case: "declared", value: { status: ResultStatus.PASSED, reason: "", importer: "fixture" } },
    }),
    "passed",
  );
  assert.equal(projectExecutionStatus(ExecutionState.OPEN, undefined, undefined, LiveExecutionState.CLEANING_UP), "finalizing");
  const state = executionControlState({ status: "planned", controls: null, progress: null, promptWaiting: false, stopping: false });
  assert.deepEqual(state.actions, []);
  assert.deepEqual(
    executionControlState({ status: "passed", controls: null, progress: null, promptWaiting: false, stopping: false }).actions,
    [],
  );
  assert.throws(() => projectExecutionStatus(ExecutionState.SEALED), /Outcome Attribution/);
  const values: unknown[] = [];
  values.push(values);
  assert.throws(() => jsonToNativeValue(values), /Cycles/);
  const exact = new NativeIntegerValue(18446744073709551615n, true);
  assert.equal(String(nativeValueToJson(jsonToNativeValue(JSON.parse(JSON.stringify(exact))))), exact.decimal);
});

test("Live and Saved Records Decode with Each Execution's Frozen Descriptor", async () => {
  const unsigned = create(FileDescriptorSetSchema, {
    file: [
      {
        name: "reading.proto",
        package: "fixture",
        syntax: "proto3",
        messageType: [
          {
            name: "Reading",
            field: [
              { name: "count", number: 1, type: FieldDescriptorProto_Type.UINT64 },
              { name: "voltage", number: 2, type: FieldDescriptorProto_Type.DOUBLE },
            ],
          },
        ],
      },
    ],
  });
  const text = create(FileDescriptorSetSchema, {
    file: [
      {
        name: "reading.proto",
        package: "fixture",
        syntax: "proto3",
        messageType: [{ name: "Reading", field: [{ name: "count", number: 1, type: FieldDescriptorProto_Type.STRING }] }],
      },
    ],
  });
  const unsignedType = createFileRegistry(unsigned).getMessage("fixture.Reading");
  const textType = createFileRegistry(text).getMessage("fixture.Reading");
  assert.ok(unsignedType);
  assert.ok(textType);
  const contracts = toBinary(
    RecordContractsSchema,
    create(RecordContractsSchema, { records: [{ metadata: { identifier: "fixture.reading" }, message: "fixture.Reading" }] }),
  );
  const cursor = { epoch: "host", stream: "records", position: 3n };
  const records = [
    {
      recordId: "unsigned:1",
      sequence: 1n,
      timestamp: { seconds: 1n },
      recordType: "fixture.reading",
      severity: Severity.INFO,
      scope: { executionId: "unsigned", cycleId: "cycle", origin: { kind: { kind: { case: "stepAttemptId", value: "attempt" } } } },
      payload: toBinary(unsignedType, create(unsignedType, { count: 18446744073709551615n, voltage: -0 })),
    },
    {
      recordId: "text:2",
      sequence: 2n,
      timestamp: { seconds: 2n },
      recordType: "fixture.reading",
      severity: Severity.INFO,
      scope: { executionId: "text", cycleId: "cycle" },
      payload: toBinary(textType, create(textType, { count: "changed-schema" })),
    },
  ];
  fixture.detail.mockImplementation(async (_identity, request) => ({
    schemas: {
      catalog: {
        types: request.executionId === "unsigned" ? unsigned : text,
        sections: [{ type: "dokime.v1.RecordContracts", payload: contracts }],
      },
    },
  }));
  fixture.extra.mockImplementation(async (path) => {
    assert.equal(path, "/dokime.v1.Evidence/ListRecords");
    return { records, scanned: cursor, through: cursor, replayComplete: true };
  });
  const tail = await fetchNativeRecordTail(session, { test_cycle_id: "cycle" });
  assert.equal(String(tail.records[0].details.count), "18446744073709551615");
  assert.ok(Object.is(Number(tail.records[0].details.voltage), -0));
  assert.equal(tail.records[0].scope?.origin?.kind, "step_attempt");
  assert.equal(tail.records[1].details.count, "changed-schema");
  const cancel = vi.fn(async () => {});
  fixture.stream.mockImplementation(async (path, input, output, _identity, request) => {
    assert.equal(path, "/dokime.v1.Evidence/WatchRecords");
    const decoded = fromBinary(input, toBinary(input, create(input, request)));
    assert.ok("after" in decoded);
    assert.deepEqual(decoded.after, { $typeName: "dokime.v1.Cursor", ...cursor });
    return {
      cancel,
      async *[Symbol.asyncIterator]() {
        yield create(output, {
          records: [records[0]],
          scanned: { ...cursor, position: 4n },
          through: { ...cursor, position: 4n },
          replayComplete: true,
        });
      },
    };
  });
  const release = vi.fn();
  const stream = await openNativeRecordStream({ ...session, own: () => release }, { test_cycle_id: "cycle" }, tail.cursor);
  for await (const page of stream) {
    assert.equal(page.cursor?.position, 4n);
    assert.equal(String(page.records[0].details.count), "18446744073709551615");
    break;
  }
  assert.equal(cancel.mock.calls.length, 1);
  assert.equal(release.mock.calls.length, 1);
});

test("Artifact Bytes Require Canonical Metadata Contiguous Chunks and Verified Completion", async () => {
  const data = new TextEncoder().encode("verified-content");
  const digest = await sha256Hex(data);
  let gap = false;
  const canceled = vi.fn(async () => {});
  fixture.stream.mockImplementation(async (path, input, output, _identity, request) => {
    assert.equal(path, "/dokime.v1.Content/DownloadArtifact");
    const decoded = fromBinary(input, toBinary(input, create(input, request)));
    assert.ok("artifactId" in decoded);
    assert.equal(decoded.artifactId, "attachment");
    return {
      cancel: canceled,
      async *[Symbol.asyncIterator]() {
        yield create(output, {
          payload: {
            kind: {
              case: "metadata",
              value: {
                artifactId: "attachment",
                filename: "reading.txt",
                sizeBytes: BigInt(data.length),
                sha256: digest,
                state: ArtifactState.COMPLETE,
              },
            },
          },
        });
        yield create(output, { payload: { kind: { case: "chunk", value: { offset: gap ? 1n : 0n, data } } } });
        yield create(output, { payload: { kind: { case: "complete", value: { sizeBytes: BigInt(data.length), sha256: digest } } } });
      },
    };
  });
  const release = vi.fn();
  const owned = { ...session, own: () => release };
  assert.deepEqual(await fetchNativeArtifactBytes(owned, "attachment"), data);
  gap = true;
  await assert.rejects(fetchNativeArtifactBytes(owned, "attachment"));
  assert.equal(canceled.mock.calls.length, 2);
  assert.equal(release.mock.calls.length, 2);
});

test("Default Execution Documents Keep the Host's Selected Identity and Epoch", async () => {
  const canceled = vi.fn(async () => {});
  fixture.stream.mockImplementation(async (path, input, output, _identity, request) => {
    assert.equal(path, "/dokime.v1.Documents/CaptureCurrentDocument");
    const decoded = fromBinary(input, toBinary(input, create(input, request)));
    assert.ok("documentId" in decoded);
    assert.equal(decoded.documentId, "");
    return {
      cancel: canceled,
      async *[Symbol.asyncIterator]() {
        yield create(output, {
          frame: {
            kind: {
              case: "header",
              value: {
                documentId: "execution",
                scope: { executionId: "execution", cycleId: "cycle" },
                metadata: { identifier: "execution", label: "Execution" },
                frontiers: { processEpoch: "product-epoch", stateRevision: 3n },
              },
            },
          },
        });
        yield create(output, { frame: { kind: { case: "structureComplete", value: { frameCount: 2n } } } });
        yield create(output, {
          frame: {
            kind: {
              case: "evidenceCut",
              value: { cut: { store: "host", epoch: "execution", position: 0n }, telemetry: true, cacheable: false },
            },
          },
        });
        yield create(output, { frame: { kind: { case: "completion", value: { frameCount: 4n } } } });
      },
    };
  });
  const release = vi.fn();
  const document = await fetchNativeDocument(
    { ...session, own: () => release },
    { cycleId: "cycle", executionId: "execution", documentId: "default", epoch: "product-epoch" },
  );
  assert.equal(document.header.documentId, "execution");
  assert.equal(document.complete, true);
  assert.equal(release.mock.calls.length, 1);
});

test("History Projects Attributed Outcomes Without Inventing Criterion Counts", async () => {
  const { fetchNativeExecutionPage } = await import("@/lib/api/native-checkpoints");
  const { ExecutionAvailability, ArchiveScope } = await import("@host/dokime/v1/client_types_pb.js");
  fixture.extra.mockImplementation(async (path, request) => {
    assert.equal(path, "/dokime.v1.History/BrowseExecutions");
    assert.equal(request.filter.archive, ArchiveScope.ALL);
    return {
      discoveryComplete: true,
      entries: [
        {
          metadata: {
            executionId: "imported",
            caseId: "case",
            title: "Imported Result",
            status: ExecutionState.SEALED,
            outcome: { attribution: { case: "declared", value: { status: ResultStatus.PASSED } } },
          },
          availability: ExecutionAvailability.STORED,
        },
      ],
    };
  });
  const page = await fetchNativeExecutionPage(session, null, undefined, {
    filter: { text: "", archive: ArchiveScope.ALL, statuses: [], templateIds: [], cycleIds: [] },
  });
  assert.equal(page.entries[0].metadata.status, "passed");
  assert.equal(page.entries[0].metadata.result, null);
});

test("Checkpoint Records Use Frozen Evidence and Preserve the Captured Log", async () => {
  const { fetchNativeCheckpointRecords, nativeRecordText } = await import("@/lib/api/native-checkpoints");
  const { LogsAppendRequestSchema } = await import("@host/dokime/context/v1/evidence/logs_pb.js");
  const position = { history: { store: "stored", epoch: "captured" }, sequence: 7n };
  const checkpoint = { executionIdentifier: "archived", position, evidenceThrough: position, version: new Uint8Array([1]) };
  fixture.extra.mockImplementation(async (path) => {
    if (path === "/dokime.v1.History/ListCheckpointRecords")
      return {
        checkpoint,
        records: [
          {
            epoch: "captured",
            ordinal: 1n,
            record: {
              timestamp: { seconds: 1n, nanos: 0 },
              recordId: "log",
              recordType: "dokime.log",
              source: "thermal",
              severity: Severity.INFO,
              payload: toBinary(
                LogsAppendRequestSchema,
                create(LogsAppendRequestSchema, { severity: Severity.INFO, message: "Captured Measurement" }),
              ),
            },
          },
        ],
      };
    assert.equal(path, "/dokime.v1.History/GetCheckpointDocument");
    return { checkpoint, document: { execution: { executionId: "archived", status: ExecutionState.SEALED } } };
  });
  const page = await fetchNativeCheckpointRecords(session, checkpoint);
  assert.equal(page.records[0].decoded?.details.message, "Captured Measurement");
  assert.ok(page.records[0].record);
  assert.equal(nativeRecordText(page.records[0].record, page.records[0].decoded).severity, "info");
  assert.equal(fixture.extra.mock.calls.length, 2);
});

test("Notes Publish Through Canonical Owner and Keep Their Mutation Identity", async () => {
  const { createNativeNote } = await import("@/lib/api/native-annotations");
  const { NotePublication } = await import("@host/dokime/v1/client_types_pb.js");
  fixture.extra.mockImplementation(async (path, request) => {
    assert.equal(path, "/dokime.v1.Notes/CreateNote");
    assert.equal(request.epoch, "execution-epoch");
    assert.deepEqual(request.owner.owner.kind, {
      case: "execution",
      value: create((await import("@host/dokime/v1/client_types_pb.js")).ExecutionNoteOwnerSchema, {
        cycleId: "cycle",
        executionId: "run",
      }),
    });
    assert.ok(request.mutationId);
    return { note: { noteId: "note", createdAt: { seconds: 1n, nanos: 0 } }, publication: NotePublication.CONFIRMED };
  });
  const receipt = await createNativeNote(session, "execution-epoch", "Measured", { test_cycle_id: "cycle", test_execution_id: "run" });
  assert.equal(receipt.result?.publication, "confirmed");
});

test("Settings Read Canonical Cards and Save Typed Plugin Values", async () => {
  const { fetchNativeSettingsFrame, getNativePluginSettings, updateNativePluginSettings } = await import("@/lib/api/native-settings");
  const { SettingsCardState } = await import("@host/dokime/v1/client_types_pb.js");
  const settings = {
    settings: {
      settings: [{ metadata: { identifier: "power", label: "Power" }, kind: { case: "boolean", value: { defaultValue: false } } }],
    },
    values: { values: { power: jsonToNativeValue(true) } },
  };
  fixture.extra.mockImplementation(async (path, request) => {
    if (path.endsWith("/GetSettings"))
      return {
        sections: [{ sectionId: "entry", owner: { kind: { case: "installedPluginId", value: "entry" } } }],
        cards: [
          { cardId: "state", state: SettingsCardState.READY, body: { blocks: [{ kind: { kind: { case: "text", value: "Ready" } } }] } },
        ],
      };
    if (path.endsWith("/UpdatePluginSettings")) assert.equal(nativeValueToJson(request.values.values.power), false);
    else assert.ok(path.endsWith("/GetPluginSettings"));
    return settings;
  });
  const frame = await fetchNativeSettingsFrame(session);
  assert.equal(frame.sections[0].owner.kind, "installed_plugin");
  assert.equal(frame.cards[0].state, "ready");
  assert.equal(frame.cards[0].body.blocks?.[0].type, "text");
  const projected = await getNativePluginSettings(session, "entry");
  assert.equal(projected.values.power, true);
  assert.equal(projected.settings.contributions?.[0].kind, "boolean");
  assert.equal(projected.restart, undefined);
  await updateNativePluginSettings(session, "entry", { power: false });
});

test("Diagnostics Read Stable Ranges and Verify the Whole Archive", async () => {
  const { collectDiagnosticsBundle } = await import("@/lib/api/native-diagnostics");
  const bytes = new Uint8Array(262_145).fill(13);
  const digest = new Uint8Array(await crypto.subtle.digest("SHA-256", bytes));
  const name = Array.from(digest, (byte) => byte.toString(16).padStart(2, "0")).join("");
  let corrupt = false;
  fixture.extra.mockImplementation(async (path, request) => {
    assert.equal(path, "/harmos.v1.status.Diagnostics/Get");
    assert.equal(request.name, request.offset === 0n ? "" : name);
    const data = bytes.slice(Number(request.offset), Number(request.offset + request.maxBytes));
    if (corrupt) data[0] ^= 1;
    return { name, totalLength: BigInt(bytes.length), sha256: digest, data };
  });
  const bundle = await collectDiagnosticsBundle(session);
  assert.deepEqual(bundle.bytes, bytes);
  assert.ok(bundle.filename.endsWith(".tar"));
  corrupt = true;
  await assert.rejects(collectDiagnosticsBundle(session), /Digest/);
});

test("Trace Snapshot Hands Its Exact Position to Watch and Filters Scope", async () => {
  const { openTraceStream, NativeTraceHistoryExpired } = await import("@/lib/api/native-logs");
  const at = { history: { store: "trace", epoch: "retained" }, sequence: 2n };
  fixture.extra.mockImplementation(async (path) => {
    assert.equal(path, "/harmos.v1.status.TraceEvents/List");
    return {
      resolvedAt: at,
      events: [
        { sequence: 1n, level: "INFO", fields: { execution_id: "run", cycle_id: "cycle" } },
        { sequence: 2n, level: "ERROR", fields: { execution_id: "other", cycle_id: "cycle" } },
      ],
    };
  });
  const cancelled = vi.fn(async () => {});
  fixture.stream.mockImplementation(async (path, _input, output, _identity, request) => {
    assert.equal(path, "/harmos.v1.status.TraceEvents/Watch");
    assert.deepEqual(request.at, create((await import("@host/harmos/v1/types_pb.js")).PositionSchema, at));
    return {
      cancel: cancelled,
      async *[Symbol.asyncIterator]() {
        yield create(output, {
          resolvedAt: { ...at, sequence: 3n },
          events: [{ sequence: 3n, level: "WARN", fields: { execution_id: "run", cycle_id: "cycle" } }],
        });
        throw new Error("history gap: resynchronize with List");
      },
    };
  });
  const stream = await openTraceStream(session, { cycleId: "cycle", executionId: "run", minLevel: "info", afterSequence: 0n });
  const pages = stream[Symbol.asyncIterator]();
  const first = await pages.next();
  assert.equal(first.value.records.length, 1);
  const live = await pages.next();
  assert.equal(live.value.records[0].level, "warning");
  await assert.rejects(pages.next(), NativeTraceHistoryExpired);
  assert.equal(cancelled.mock.calls.length, 1);
});

test("Remote Listener Saves with Its Own Revision and Rotates Certificates", async () => {
  const { readNativeListener, updateNativeListener, regenerateNativeCertificate } = await import("@/lib/api/native-remote");
  const revision = new Uint8Array([17]);
  const value = { enabled: false, endpoint: "127.0.0.1:0", certificateNames: ["localhost"] };
  fixture.extra.mockImplementation(async (path, request) => {
    if (path.endsWith("Listener/Set")) {
      assert.deepEqual(request.expectedRevision, revision);
      assert.equal(request.value.enabled, true);
      return { value: request.value, revision: new Uint8Array([18]), boundEndpoint: "127.0.0.1:4321" };
    }
    if (path.endsWith("Certificates/Create")) return { certificatePem: "rotated" };
    assert.ok(path.endsWith("Listener/Get"));
    return { value, revision };
  });
  const current = await readNativeListener(session);
  const saved = await updateNativeListener(session, current, { ...current.settings, enabled: true });
  assert.equal(saved.listeningAddress, "127.0.0.1:4321");
  await assert.rejects(updateNativeListener(session, current, { ...current.settings, tlsEnabled: false }));
  assert.equal((await regenerateNativeCertificate(session)).certificatePem, "rotated");
});
