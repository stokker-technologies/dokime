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
import { create, fromBinary, toBinary } from "@bufbuild/protobuf";
import { FieldDescriptorProto_Type, FileDescriptorSetSchema } from "@bufbuild/protobuf/wkt";
import {
  ArchiveScope,
  CyclePhase,
  ExecutionMode,
  ExecutionState,
  LiveExecutionState,
  PromptFieldKind,
  PromptKind,
  ResultStatus,
  Severity,
  StreamMatchOperator,
} from "@generated/host/model";
import { QueryClient, QueryObserver } from "@tanstack/react-query";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { beforeEach, test, vi } from "vitest";

// Local Imports
import { fetchNativeArtifactBytes } from "@/lib/api/native-content";
import { openNativeExecutionProgressStream, openNativeRuntimeStream } from "@/lib/api/native-observation";
import { previewNativeTestCycle } from "@/lib/api/native-planning";
import { fetchNativePendingPrompts, respondNativePrompt } from "@/lib/api/native-prompts";
import { NativeIntegerValue, NativeNumberValue } from "@/lib/api/native-values";
import { jsonToNativeValue, nativeValueToJson } from "@/lib/host/values";
import { fetchNativeDocument } from "../../../src/lib/api/native-document-session";
import { projectExecutionStatus } from "@/lib/api/native-planning-projection";
import { ControlCard } from "@/features/cycle/ControlCard";
import { cycleControlState, executionControlState } from "@/features/cycle/control-model";
import { invalidateRuntimeDocuments, invalidateRuntimeProjectionQueries, workflowQueryKeys } from "@/features/cycle/server-state";
import {
  appendNativeCaseAttempt,
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
  watch: vi.fn(),
}));

const session = { generation: "lifecycle", isCurrent: () => true };

/* === Types === */

/* === Classes === */

/* === Public Functions === */

/* === Private Functions === */

function accepted() {
  return {
    execution: { summary: { executionId: "execution" }, controls: { requestId: "execution" } },
    receipt: { requestId: "execution", completed: true },
  };
}

/* === Entry Point === */

vi.mock("@/lib/rpc/transport", () => ({ unary: fixture.unary, openStream: fixture.stream }));

beforeEach(() => {
  vi.clearAllMocks();
  fixture.extra.mockReset();
  fixture.stream.mockReset();
  fixture.stream.mockImplementation(async (path, _input, _output, identity, request) => {
    assert.equal(path, "/dokime.host.v1.Cycles/Watch");
    return fixture.watch(identity, request);
  });
  fixture.unary.mockImplementation(async (path, input, output, identity, request) => {
    const decoded = fromBinary(input, toBinary(input, create(input, request)));
    const operations: Record<string, typeof fixture.append> = {
      "/dokime.host.v1.Executions/Create": fixture.append,
      "/dokime.host.v1.Executions/Start": fixture.start,
      "/dokime.host.v1.Executions/Stop": fixture.stop,
      "/dokime.host.v1.Cycles/Seal": fixture.seal,
    };
    const operation = operations[path];
    const reply = operation ? await operation(identity, request) : await fixture.extra(path, decoded);
    return fromBinary(output, toBinary(output, create(output, reply)));
  });
});

test("Append Without Start", async () => {
  fixture.append.mockResolvedValue({});
  const first = await appendNativeCaseAttempt(session, "cycle", "case");
  const second = await appendNativeCaseAttempt(session, "cycle", "case");
  assert.notEqual(first, second);
  assert.match(first, /^[0-9a-f-]{36}$/);
  assert.deepEqual(
    fixture.append.mock.calls.map((call) => ({ parent: call[1].parent, executions: call[1].executions })),
    [
      { parent: "cycle", executions: [{ executionId: first, caseId: "case" }] },
      { parent: "cycle", executions: [{ executionId: second, caseId: "case" }] },
    ],
  );
  assert.notEqual(fixture.append.mock.calls[0][1].requestId, fixture.append.mock.calls[1][1].requestId);
  assert.equal(fixture.start.mock.calls.length, 0);
});

test("Start Reuses Identity", async () => {
  fixture.start.mockResolvedValue(accepted());
  await startNativeTestExecution(session, "execution");
  await startNativeTestExecution(session, "execution");
  assert.deepEqual(
    fixture.start.mock.calls.map((call) => ({ name: call[1].name, mode: call[1].mode })),
    [
      { name: "execution", mode: ExecutionMode.AUTOMATIC },
      { name: "execution", mode: ExecutionMode.AUTOMATIC },
    ],
  );
  assert.notEqual(fixture.start.mock.calls[0][1].requestId, fixture.start.mock.calls[1][1].requestId);
});

test("Start Admission Refusal", async () => {
  const refusal = { kind: "grpc", detail: { code: 9, message: "cycle_sealed", details: [] } };
  fixture.start.mockRejectedValue(refusal);
  await assert.rejects(startNativeTestExecution(session, "execution"), (error) => error === refusal);
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
  assert.equal(fixture.stop.mock.calls[0][1].name, "execution");
  assert.ok(fixture.stop.mock.calls[0][1].requestId);

  let failSeal!: (error: Error) => void;
  fixture.seal.mockImplementation(
    () =>
      new Promise<void>((_resolve, reject) => {
        failSeal = reject;
      }),
  );
  const seal = sealNativeTestCycle(session, "cycle");
  const failure = new Error("storage");
  const rejected = assert.rejects(seal, (error) => error === failure);
  failSeal(failure);
  await rejected;
  assert.equal(fixture.seal.mock.calls[0][1].name, "cycle");
  assert.ok(fixture.seal.mock.calls[0][1].requestId);
});

test("Archived Cycle Seal Awaits Completion and Raises Materialization Failure", async () => {
  fixture.seal.mockResolvedValue({ cycle: { cycleId: "cycle", phase: CyclePhase.SEALING, archived: true } });
  let finish!: () => void;
  const gate = new Promise<void>((resolve) => {
    finish = resolve;
  });
  const cancelled = vi.fn(async () => undefined);
  fixture.watch.mockImplementation(async (_identity, request) => ({
    cancel: cancelled,
    async *[Symbol.asyncIterator]() {
      if (request.filter?.archive !== ArchiveScope.ALL) {
        yield { cycles: [] };
        return;
      }
      yield { cycles: [{ cycleId: "cycle", phase: CyclePhase.SEALING, sealed: false }] };
      await gate;
      yield { cycles: [{ cycleId: "cycle", phase: CyclePhase.SEALED, sealed: true }] };
    },
  }));
  let sealed = false;
  const pending = sealNativeTestCycle(session, "cycle").then(() => {
    sealed = true;
  });
  await Promise.resolve();
  assert.equal(sealed, false);
  finish();
  await pending;
  assert.equal(sealed, true);
  assert.equal(cancelled.mock.calls.length, 1);
  fixture.watch.mockResolvedValue({
    cancel: cancelled,
    async *[Symbol.asyncIterator]() {
      yield { cycles: [{ cycleId: "cycle", phase: CyclePhase.SEAL_FAILED, sealed: false, sealFailure: "storage" }] };
    },
  });
  await assert.rejects(sealNativeTestCycle(session, "cycle"));
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
  // Sealing and Sealed Offer No Running Controls.
  assert.deepEqual(cycleControlState({ phase: "sealing", running: null }).actions, []);
  assert.deepEqual(cycleControlState({ phase: "sealed", running: null }).actions, []);

  // A Failed Seal Offers Retry and Archive, Never a Running Control
  assert.deepEqual(cycleControlState({ phase: "seal_failed", running: null }).actions, ["Retry Seal", "Archive"]);
});

test("Sealed Execution Without Its Plugin Has No New Attempt Button", () => {
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
  assert.equal(state.phase, "sealed");
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
    for (const unsubscribe of release) {
      unsubscribe();
    }
    client.clear();
  }
});

test("Planning Preserves Selection Order and Exact Recursive Parameters", async () => {
  const requests: Record<string, unknown>[] = [];
  fixture.extra.mockImplementation(async (path, request) => {
    if (path === "/dokime.host.v1.Suites/List") {
      return {
        values: [
          { metadata: { identifier: "fixture", label: "Fixture" }, templates: [{ metadata: { identifier: "fixture.run", label: "Run" } }] },
        ],
      };
    }
    if (path === "/dokime.host.v1.Suites/Get") {
      return {
        metadata: { identifier: "fixture", label: "Fixture" },
        templates: [{ metadata: { identifier: "fixture.run", label: "Run" } }],
      };
    }
    assert.equal(path, "/dokime.host.v1.Cases/Create");
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
    if (path === "/dokime.host.v1.Prompts/List") {
      assert.equal(request.cycleId, "cycle");
      return {
        prompts: [
          {
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
                        {
                          kind: { case: "stream", value: { stream: "temperature", field: "value", operator: StreamMatchOperator.EXISTS } },
                        },
                      ],
                    },
                  },
                },
              },
            },
          },
        ],
      };
    }
    assert.equal(path, "/dokime.host.v1.prompts.Answers/Create");
    replies.push(request);
    assert.equal(request.values.values.count.kind.case, "unsignedInteger");
    assert.equal(request.values.values.count.kind.value, 18446744073709551615n);
    assert.equal(request.action, "confirm");
    assert.equal(request.accepted, true);
    return { answer: { promptId: "prompt", submittedAt: { seconds: 8n } }, receipt: { requestId: "answer", completed: true } };
  });
  const [prompt] = await fetchNativePendingPrompts(session, { cycleId: "cycle" });
  assert.equal(prompt.kind, "form");
  assert.equal(prompt.level, "warning");
  assert.equal(prompt.fields[0].kind, "int");
  assert.equal(prompt.actions[0].id, "confirm");
  assert.equal(prompt.scope?.executionId, "execution");
  assert.equal(prompt.verification?.condition.kind, "host");
  const receipt = await respondNativePrompt(session, prompt.correlation_id, "confirm", { count: prompt.fields[0].default });
  assert.equal(receipt.command_id, "answer");
  assert.equal(receipt.result?.accepted, true);
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
    assert.equal(envelope, undefined);
    const decoded = fromBinary(input, toBinary(input, create(input, request)));
    const frame = path.endsWith("Overview/Watch")
      ? { opening: true, resolvedAt: { positions: [{ history: { store: "host", epoch: "runtime" }, sequence: 9n }] } }
      : { sealed: true };
    if (path === "/dokime.host.v1.executions.Progress/Watch") {
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
    assert.equal(frame.opening, true);
    assert.equal(frame.resolvedAt?.positions[0]?.sequence, 9n);
    break;
  }
  assert.equal(canceled.mock.calls.length, 1);
  assert.equal(release.mock.calls.length, 1);
  const progress = await openNativeExecutionProgressStream(observation, "execution");
  const frames = [];
  for await (const frame of progress) {
    frames.push(frame);
  }
  assert.equal(frames[0].progress, null);
  assert.equal(frames[0].sealed, true);
  assert.equal(release.mock.calls.length, 2);
  await progress.cancel();
  for (const dispose of owned) {
    dispose();
  }
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

test("Frozen Descriptor Pages Are Verified and Reused Across Executions", async () => {
  const { FrozenSchemaCache } = await import("@/lib/host/schemas");
  const types = create(FileDescriptorSetSchema, {
    file: [
      {
        name: "reading.proto",
        dependency: ["base.proto"],
        package: "fixture",
        syntax: "proto3",
        messageType: [{ name: "Reading", field: [{ name: "count", number: 1, type: FieldDescriptorProto_Type.UINT64 }] }],
      },
      { name: "base.proto", package: "fixture", syntax: "proto3", messageType: [{ name: "Other" }] },
    ],
  });
  const digest = new Uint8Array(await crypto.subtle.digest("SHA-256", toBinary(FileDescriptorSetSchema, types)));
  const at = { history: { store: "host", epoch: "execution" }, sequence: 7n };
  fixture.extra.mockImplementation(async (path, request) => {
    assert.equal(path, "/dokime.host.v1.executions.Schemas/Get");
    assert.equal(request.name, "one");
    return { digest, types, resolvedAt: at };
  });
  const cache = new FrozenSchemaCache();
  const detail = (executionId: string) => create(awaitedExecutionSchema, { summary: { executionId }, schemas: { digest } });
  const awaitedExecutionSchema = (await import("@generated/host/model")).ExecutionDetailSchema;
  const first = await cache.get(session, detail("one"));
  const second = await cache.get(session, detail("two"));
  assert.equal(first, second);
  assert.ok(first.getMessage("fixture.Reading"));
  assert.ok(first.getMessage("fixture.Other"));
  assert.equal(fixture.extra.mock.calls.length, 1);
  const corrupt = new FrozenSchemaCache();
  fixture.extra.mockResolvedValue({ digest, types: { file: [] }, resolvedAt: at });
  await assert.rejects(corrupt.get(session, detail("three")));
});

test("Artifact Ranges Require a Stable Cut and Verified Digest", async () => {
  const data = new Uint8Array([1, 2, 3, 4]);
  const digest = new Uint8Array(await crypto.subtle.digest("SHA-256", data));
  const at = { positions: [{ history: { store: "files", epoch: "one" }, sequence: 1n }] };
  let corrupt = false;
  fixture.extra.mockImplementation(async (path, request) => {
    if (path === "/dokime.host.v1.content.Artifacts/Get") {
      return { file: { length: 4n, sha256: digest }, resolvedAt: at };
    }
    if (path === "/dokime.host.v1.Files/Get") {
      return { file: { length: 4n, sha256: digest }, chunkBytes: 2n, messageBytes: 1024n, resolvedAt: at };
    }
    assert.equal(path, "/dokime.host.v1.Files/Read");
    assert.ok(request.offset === 0n || request.offset === 2n);
    assert.equal(request.maxBytes, 2n);
    const selected = corrupt ? new Uint8Array([4, 3, 2, 1]) : data;
    return { data: selected.slice(Number(request.offset), Number(request.offset) + 2), totalLength: 4n, sha256: digest, resolvedAt: at };
  });
  const owned = { ...session, own: () => () => undefined };
  assert.deepEqual(await fetchNativeArtifactBytes(owned, "attachment"), data);
  corrupt = true;
  await assert.rejects(fetchNativeArtifactBytes(owned, "attachment"));
});

test("Default Execution Documents Preserve Ordered Frames Across Unary Pages", async () => {
  const at = { positions: [{ history: { store: "host", epoch: "execution" }, sequence: 0n }] };
  fixture.extra.mockImplementation(async (path, request) => {
    assert.equal(path, "/dokime.host.v1.Documents/Get");
    assert.equal(request.documentId, "");
    if (!request.pageToken) {
      return {
        frames: [
          {
            frame: {
              kind: {
                case: "header",
                value: {
                  documentId: "execution",
                  metadata: { identifier: "execution" },
                  scope: { executionId: "execution", cycleId: "cycle" },
                  frontiers: { processEpoch: "product-epoch", stateRevision: 3n },
                },
              },
            },
          },
          { frame: { kind: { case: "structureComplete", value: { frameCount: 2n } } } },
        ],
        resolvedAt: at,
        nextPageToken: "next",
      };
    }
    assert.equal(request.pageToken, "next");
    assert.equal(request.at.positions[0].sequence, 0n);
    return {
      frames: [
        {
          frame: {
            kind: {
              case: "evidenceCut",
              value: { cut: { store: "host", epoch: "execution", position: 0n }, telemetry: true, cacheable: false },
            },
          },
        },
        { frame: { kind: { case: "completion", value: { frameCount: 4n } } } },
      ],
      resolvedAt: at,
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
  assert.equal(fixture.extra.mock.calls.length, 2);
});

test("History Projects Declared Outcomes Without Inventing Criterion Counts", async () => {
  const { fetchNativeExecutionPage } = await import("@/lib/api/native-checkpoints");
  const at = { positions: [{ history: { store: "host", epoch: "inventory" }, sequence: 1n }] };
  fixture.extra.mockImplementation(async (path) => {
    if (path === "/dokime.host.v1.Cycles/List") {
      return { cycles: [], resolvedAt: at };
    }
    assert.equal(path, "/dokime.host.v1.Executions/List");
    return {
      executions: [
        {
          summary: { executionId: "imported", caseId: "case", state: ExecutionState.SEALED },
          outcome: { attribution: { case: "declared", value: { status: ResultStatus.PASSED } } },
        },
      ],
      resolvedAt: at,
    };
  });
  const page = await fetchNativeExecutionPage(session, null);
  assert.equal(page.entries[0].metadata.status, "passed");
  assert.equal(page.entries[0].metadata.result, null);
});

test("Notes Publish Through Canonical Owner and Keep Their Mutation Identity", async () => {
  const { createNativeNote } = await import("@/lib/api/native-annotations");
  fixture.extra.mockImplementation(async (path, request) => {
    assert.equal(path, "/dokime.host.v1.evidence.Notes/Create");
    assert.deepEqual(request.owner.owner.kind, {
      case: "execution",
      value: create((await import("@generated/host/model")).ExecutionNoteOwnerSchema, {
        cycleId: "cycle",
        executionId: "run",
      }),
    });
    assert.ok(request.requestId);
    return { note: { noteId: "note", createdAt: { seconds: 1n, nanos: 0 } }, receipt: { completed: true, requestId: request.requestId } };
  });
  const receipt = await createNativeNote(session, "Measured", { cycleId: "cycle", executionId: "run" });
  assert.equal(receipt.result?.accepted, true);
});

test("Settings Read Canonical Cards and Save Typed Plugin Values", async () => {
  const { fetchNativeSettingsFrame, getNativePluginSettings, updateNativePluginSettings } = await import("@/lib/api/native-settings");
  const { SettingsCardState } = await import("@generated/host/model");
  const settings = {
    declarations: [{ name: "power", title: "Power", default: { value: { case: "flag", value: false } } }],
    value: { values: { power: { value: { case: "flag", value: true } } } },
    revision: new Uint8Array([4]),
  };
  fixture.extra.mockImplementation(async (path, request) => {
    if (path.endsWith("Settings/Schema")) {
      return {
        schema: {
          sections: [{ sectionId: "entry", owner: { kind: { case: "installedPluginId", value: "entry" } } }],
          cards: [
            { cardId: "state", state: SettingsCardState.READY, body: { blocks: [{ kind: { kind: { case: "text", value: "Ready" } } }] } },
          ],
        },
      };
    }
    if (path.endsWith("plugins.Secrets/List")) {
      return { secrets: [] };
    }
    if (path.endsWith("plugins.Settings/Set")) {
      assert.deepEqual(request.value.values.power.value, { case: "flag", value: false });
      assert.deepEqual(request.expectedRevision, new Uint8Array([4]));
    } else {
      assert.ok(path.endsWith("plugins.Settings/List"));
    }
    return settings;
  });
  const frame = await fetchNativeSettingsFrame(session);
  assert.equal(frame.sections[0].owner?.kind.case, "installedPluginId");
  assert.equal(frame.cards[0].state, SettingsCardState.READY);
  assert.equal(frame.cards[0].body?.blocks[0].kind?.kind.case, "text");
  const projected = await getNativePluginSettings(session, "entry");
  assert.equal(projected.values.power, true);
  const contribution = projected.settings.contributions?.[0];
  assert.equal(contribution?.kind, "setting");
  if (contribution?.kind === "setting") {
    assert.equal(contribution.declaration.default?.value.case, "flag");
  }
  await updateNativePluginSettings(session, "entry", { power: false });
});

test("Diagnostics Read Stable Ranges and Verify the Whole Archive", async () => {
  const { collectDiagnosticsBundle } = await import("@/lib/api/native-diagnostics");
  const bytes = new Uint8Array(262_145).fill(13);
  const digest = new Uint8Array(await crypto.subtle.digest("SHA-256", bytes));
  const name = Array.from(digest, (byte) => byte.toString(16).padStart(2, "0")).join("");
  let corrupt = false;
  fixture.extra.mockImplementation(async (path, request) => {
    assert.equal(path, "/dokime.host.v1.status.Diagnostics/Get");
    assert.equal(request.name, request.offset === 0n ? "" : name);
    const data = bytes.slice(Number(request.offset), Number(request.offset + request.maxBytes));
    if (corrupt) {
      data[0] ^= 1;
    }
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
    assert.equal(path, "/dokime.host.v1.status.TraceEvents/List");
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
    assert.equal(path, "/dokime.host.v1.status.TraceEvents/Watch");
    assert.deepEqual(request.at, create((await import("@generated/host/model")).PositionSchema, at));
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
    if (path.endsWith("Certificates/Create")) {
      return { certificatePem: "rotated" };
    }
    assert.ok(path.endsWith("Listener/Get"));
    return { value, revision };
  });
  const current = await readNativeListener(session);
  const saved = await updateNativeListener(session, current, { ...current.settings, enabled: true });
  assert.equal(saved.listeningAddress, "127.0.0.1:4321");
  await assert.rejects(updateNativeListener(session, current, { ...current.settings, tlsEnabled: false }));
  assert.equal((await regenerateNativeCertificate(session)).certificatePem, "rotated");
});

test("Plot Snapshot Continuations Keep Their Read Cut", async () => {
  const { fetchNativePlotPayloads } = await import("@/lib/api/native-plots");
  const at = { positions: [{ history: { store: "host", epoch: "evidence" }, sequence: 7n }] };
  fixture.extra.mockImplementation(async (path, request) => {
    assert.equal(path, "/dokime.host.v1.plots.Snapshots/List");
    if (!request.page.pageToken) {
      assert.equal(request.at, undefined);
      return { resolvedAt: at, nextPageToken: "next" };
    }
    assert.equal(request.page.pageToken, "next");
    assert.equal(request.at.positions[0].sequence, 7n);
    assert.equal(request.at.positions[0].history.epoch, "evidence");
    return { resolvedAt: at };
  });
  assert.deepEqual(await fetchNativePlotPayloads(session, []), []);
  assert.equal(fixture.extra.mock.calls.length, 2);
  fixture.extra.mockImplementation(async (_path, request) => ({ resolvedAt: request.at ? { positions: [] } : at, nextPageToken: "next" }));
  await assert.rejects(fetchNativePlotPayloads(session, []));
});
