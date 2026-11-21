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
 * Exercise Template Planning With Declared Numeric Values
 */

/* === Imports === */

// Standard Library Imports
import assert from "node:assert/strict";

// Third Party Imports
import { create } from "@bufbuild/protobuf";
import {
  CasesCreateResponseSchema,
  CyclePhase,
  CyclesGetResponseSchema,
  ExecutionState,
  ExecutionsCreateResponseSchema,
  ParameterSchema,
  SuiteSchema,
  ValueType,
} from "@host/model";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { IDBFactory, IDBKeyRange } from "fake-indexeddb";
import { act, createElement } from "react";
import { createRoot } from "react-dom/client";
import { test, vi } from "vitest";

// Local Imports
import { TooltipProvider } from "@/components/ui/tooltip";
import type { OpenCycle } from "@/features/library/active-cycle-model";
import { TemplateDialog } from "@/features/library/TemplateDialog";
import { previewNativeTestCycle } from "@/lib/api/native-planning";
import { mapSuite } from "@/lib/host/suites";

/* === Re-Exports === */

/* === Constants === */

const boundary = vi.hoisted(() => ({ unary: vi.fn(), preview: vi.fn(), append: vi.fn(), closed: vi.fn() }));
vi.mock("@/lib/rpc/transport", () => ({ unary: boundary.unary }));
vi.mock("@tanstack/react-router", () => ({ useNavigate: () => vi.fn() }));
vi.mock("@/app/native-session/context", () => ({
  useNativeSession: () => ({ identity: { generation: "planning", isCurrent: () => true } }),
  useNativeSessionOptional: () => null,
}));
vi.mock("@/components/suite-picker/server-state", () => ({
  usePreviewTestCycleMutation: () => ({ mutateAsync: boundary.preview }),
  useAppendTestCycleMutation: () => ({ mutateAsync: boundary.append }),
}));
vi.mock("@/components/artifact-upload-mutation", () => ({ useOperatorArtifactUploadMutation: () => ({ mutateAsync: vi.fn() }) }));
vi.mock("@/lib/notifications", async (importOriginal) => {
  const actual = await importOriginal<typeof import("@/lib/notifications")>();
  return { ...actual, toast: { ...actual.toast, success: vi.fn() } };
});
vi.mock("@/stores/parameter-overrides-store", () => ({
  useParameterOverridesStatus: () => ({ ready: true, error: null }),
  useParameterOverridesStore: (select: (state: unknown) => unknown) =>
    select({
      overrides: {},
      setParameterOverride: vi.fn(),
      resetParameterOverride: vi.fn(),
      clearTemplateOverrides: vi.fn(),
    }),
}));

/* === Types === */

/* === Classes === */

/* === Public Functions === */

/* === Private Functions === */

/* === Entry Point === */

test.each([undefined, "8192", "0"])("Template Dialog Preserves Declared Types and Refuses Invalid Inputs: %s", async (raw) => {
  vi.clearAllMocks();
  // Compose the Registered Numeric, Flag and List Defaults
  const defaults = [
    { kind: { case: "unsignedInteger" as const, value: 4096n } },
    { kind: { case: "number" as const, value: 0.25 } },
    { kind: { case: "integer" as const, value: -3n } },
    { kind: { case: "boolean" as const, value: false } },
    {
      kind: {
        case: "list" as const,
        value: { values: [{ kind: { case: "number" as const, value: 230 } }, { kind: { case: "number" as const, value: 260 } }] },
      },
    },
  ];
  const names = ["iterations", "duration_s", "offset", "realtime", "voltage_steps_v"];
  const types = [ValueType.UNSIGNED_INTEGER, ValueType.NUMBER, ValueType.INTEGER, ValueType.BOOLEAN, ValueType.LIST];
  const source = create(SuiteSchema, {
    metadata: { identifier: "benchmark", label: "Benchmark" },
    templates: [
      {
        metadata: { identifier: "heavy", label: "Heavy Observers" },
        parameters: names.map((identifier, index) =>
          create(ParameterSchema, {
            metadata: { identifier, label: identifier },
            type: types[index],
            elementType: index === 4 ? ValueType.NUMBER : ValueType.UNSPECIFIED,
            unit: index === 1 ? "s" : "",
            min: index === 0 ? 1 : undefined,
            max: index === 0 ? 65536 : undefined,
            expression: { kind: { case: "literal", value: defaults[index] } },
          }),
        ),
      },
    ],
  });
  const suite = mapSuite(source);
  const expected = defaults.map((value, index) =>
    index === 0 && raw !== undefined ? { kind: { case: "unsignedInteger" as const, value: BigInt(raw) } } : value,
  );
  const parameters = Object.fromEntries(names.map((name, index) => [name, expected[index]]));
  boundary.unary.mockImplementation(async (path: string) => {
    assert.ok(path.endsWith("Cases/Create"));
    return create(CasesCreateResponseSchema, {
      cases: [
        {
          caseId: "case",
          suiteId: "benchmark",
          templateId: "heavy",
          origin: { kind: { case: "pluginRun", value: { testCase: { parameters } } } },
        },
      ],
    });
  });
  boundary.preview.mockImplementation((request) => previewNativeTestCycle({ generation: "1", isCurrent: () => true }, request));
  boundary.append.mockResolvedValue(create(ExecutionsCreateResponseSchema));
  Object.assign(globalThis, { IS_REACT_ACT_ENVIRONMENT: true, indexedDB: new IDBFactory(), IDBKeyRange });
  const activeCycle: OpenCycle = {
    attentionCount: 0,
    caseCount: 0,
    sealedCount: 0,
    createdAtIso: null,
    createdLabel: "",
    cycleId: "cycle",
    executionCount: 0,
    label: "Cycle",
    phase: "Active",
    shortId: "cycle",
    status: "Ready",
    statusLabel: "Ready",
    statusTone: "neutral",
    updatedAtIso: null,
    updatedAtMs: null,
  };
  const container = document.createElement("div");
  document.body.append(container);
  const root = createRoot(container);
  try {
    // Open the Actual Dialog and Leave Every Field Untouched
    await act(async () =>
      root.render(
        createElement(
          TooltipProvider,
          null,
          createElement(TemplateDialog, {
            activeCycle,
            hasOpenCycles: true,
            onChangeCycle: () => {},
            onNewCycle: () => {},
            onOpenChange: boundary.closed,
            open: true,
            suite,
            template: suite.members[0],
          }),
        ),
      ),
    );
    const input = document.querySelector<HTMLInputElement>('[data-testid="parameter-input-iterations"]');
    assert.ok(input);
    assert.equal(input.type, "number");
    assert.equal(input.value, "4096");
    assert.equal(input.min, "1");
    assert.equal(input.max, "65536");
    assert.equal(input.step, "1");
    if (raw !== undefined) {
      const setValue = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")?.set;
      assert.ok(setValue);
      await act(async () => {
        setValue.call(input, raw);
        input.dispatchEvent(new Event("input", { bubbles: true }));
        input.dispatchEvent(new Event("change", { bubbles: true }));
      });
    }
    const next = Array.from(document.querySelectorAll("button")).find((button) => button.textContent === "Next");
    assert.ok(next);
    if (raw === "0") {
      assert.equal(next.disabled, true);
      assert.equal(boundary.preview.mock.calls.length, 0);
      return;
    }
    assert.equal(next.disabled, false);

    // Plan Through the Native Adapter and Complete the Single-Case Transaction
    await act(async () => next.click());
    assert.deepEqual(
      boundary.unary.mock.calls.map(([path]) => path),
      ["/dokime.host.v1.Cases/Create"],
    );
    const call = boundary.unary.mock.calls.find(([path]) => path.endsWith("Cases/Create"));
    assert.ok(call);
    assert.equal(call[4].validateOnly, true);
    assert.deepEqual(
      call[4].plan.selections[0].parameters,
      names.map((key, index) => ({ key, value: { values: [expected[index]] } })),
    );
    if (raw !== undefined) {
      assert.equal(boundary.append.mock.calls.length, 0);
      const finish = document.querySelector<HTMLButtonElement>('[data-testid="template-add"]');
      assert.ok(finish);
      await act(async () => finish.click());
    }
    assert.equal(boundary.append.mock.calls.length, 1);
    assert.deepEqual(boundary.closed.mock.calls, [[false]]);
  } finally {
    await act(async () => root.unmount());
    container.remove();
  }
});

test("Append Returns the Admission Without Rebuilding the Cycle or Refreshing the Catalog", async () => {
  vi.clearAllMocks();
  const { useAppendTestCycleMutation } = await vi.importActual<typeof import("@/components/suite-picker/server-state")>(
    "@/components/suite-picker/server-state",
  );
  const client = new QueryClient();
  const invalidations = vi.spyOn(client, "invalidateQueries");
  let admission = create(ExecutionsCreateResponseSchema);
  boundary.unary.mockImplementation(async (path: string, _input: unknown, _output: unknown, _session: unknown, request) => {
    if (path.endsWith("Cycles/Get")) {
      return create(CyclesGetResponseSchema, { cycle: { cycleId: "cycle", phase: CyclePhase.ACTIVE } });
    }
    if (path.endsWith("Cases/Create")) {
      return create(CasesCreateResponseSchema, { cases: [{ caseId: "registered", suiteId: "suite", templateId: "template" }] });
    }
    assert.equal(path, "/dokime.host.v1.Executions/Create");
    assert.equal(request.parent, "cycle");
    assert.deepEqual(
      request.executions.map((item: { caseId: string }) => item.caseId),
      ["registered"],
    );
    admission = create(ExecutionsCreateResponseSchema, {
      executions: request.executions.map((item: { executionId: string }) => ({
        summary: { executionId: item.executionId, state: ExecutionState.OPEN },
      })),
    });
    return admission;
  });
  let append: ReturnType<typeof useAppendTestCycleMutation>["mutateAsync"] | undefined;
  function Append() {
    append = useAppendTestCycleMutation().mutateAsync;
    return null;
  }
  const container = document.createElement("div");
  document.body.append(container);
  const root = createRoot(container);
  try {
    await act(async () => root.render(createElement(QueryClientProvider, { client }, createElement(Append))));
    let reply: unknown;
    await act(async () => {
      assert.ok(append);
      reply = await append({ cycleId: "cycle", selections: [{ suite_id: "suite", template_id: "template", parameters: {} }] });
    });
    assert.strictEqual(reply, admission);
    assert.deepEqual(
      boundary.unary.mock.calls.map(([path]) => path),
      ["/dokime.host.v1.Cycles/Get", "/dokime.host.v1.Cases/Create", "/dokime.host.v1.Executions/Create"],
    );
    assert.ok(invalidations.mock.calls.length > 0);
    assert.equal(
      invalidations.mock.calls.some(([filter]) => filter?.queryKey?.[0] === "suite"),
      false,
    );
  } finally {
    await act(async () => root.unmount());
    container.remove();
    client.clear();
  }
});
