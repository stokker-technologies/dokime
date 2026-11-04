/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Notification History, Faults And Delivery.
 */

/* === Imports === */

// Standard Library Imports
import assert from "node:assert/strict";
import { forceCloseDatabase, IDBFactory, IDBKeyRange, IDBObjectStore } from "fake-indexeddb";
// Third Party Imports
import { createElement, isValidElement, type ReactNode, type ReactElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { afterEach, beforeEach, test, vi } from "vitest";

// Local Imports

/* === Re-Exports === */

/* === Constants === */

const shown = vi.hoisted(() => ({
  error: vi.fn((_title: ReactNode, options?: { id?: string | number; duration?: number }) => options?.id ?? "error"),
  info: vi.fn((_title: ReactNode) => "info"),
  success: vi.fn((_title: ReactNode) => "success"),
  warning: vi.fn((_title: ReactNode) => "warning"),
}));
const controls = vi.hoisted(() => ({ close: () => {}, copy: () => {} }));
vi.mock("sonner", () => ({ toast: shown }));
vi.mock("react", async (original) => ({
  ...(await original<typeof import("react")>()),
  useId: () => "fixture-feedback",
  useEffect: (effect: () => void) => effect(),
  useSyncExternalStore: (_subscribe: unknown, snapshot: () => unknown) => snapshot(),
}));
vi.mock("@/components/ui/dialog", () => ({
  Dialog: ({ open, onOpenChange, children }: { open: boolean; onOpenChange: (open: boolean) => void; children: ReactNode }) => {
    controls.close = () => onOpenChange(false);
    return open ? children : null;
  },
  DialogContent: ({ children }: { children: ReactNode }) => createElement("section", { role: "dialog" }, children),
  DialogHeader: ({ children }: { children: ReactNode }) => createElement("header", null, children),
  DialogTitle: ({ children }: { children: ReactNode }) => createElement("h2", null, children),
  DialogDescription: ({ children }: { children: ReactNode }) => createElement("p", null, children),
  DialogFooter: ({ children }: { children: ReactNode }) => createElement("footer", null, children),
}));
vi.mock("@/components/ui/button", () => ({
  Button: ({ children, onClick }: { children: ReactNode; onClick?: () => void }) => {
    if (children === "Copy" && onClick) controls.copy = onClick;
    return createElement("button", { type: "button", onClick }, children);
  },
}));

beforeEach(() => {
  vi.resetModules();
  vi.clearAllMocks();
  vi.stubGlobal("indexedDB", new IDBFactory());
  vi.stubGlobal("IDBKeyRange", IDBKeyRange);
});
afterEach(() => {
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

/* === Types === */

/* === Classes === */

/* === Public Functions === */

/* === Private Functions === */

function record(index: number) {
  return {
    id: String(index).padStart(3, "0"),
    at: Math.floor(index / 2),
    source: "fixture",
    title: `row-${index}`,
    tone: "info" as const,
    unread: 1 as const,
  };
}

function buttonIn(node: ReactNode): ReactElement<{ onClick: () => void; type: string; "aria-haspopup": string }> | undefined {
  if (Array.isArray(node)) return node.map(buttonIn).find(Boolean);
  if (!isValidElement<{ children?: ReactNode }>(node)) return undefined;
  if (node.type === "button") return node as ReturnType<typeof buttonIn>;
  return buttonIn(node.props.children);
}

/* === Entry Point === */

test("History Paging", async () => {
  const history = await import("@/lib/notifications");
  assert.deepEqual((await history.notificationPage()).records, []);
  for (let index = 0; index < 53; index++) await history.recordNotification(record(index));
  vi.resetModules();
  const reopened = await import("@/lib/notifications");
  const first = await reopened.notificationPage();
  assert.deepEqual(
    first.records.map((value) => value.id),
    Array.from({ length: 25 }, (_, index) => record(52 - index).id),
  );
  assert.equal(first.newer, false);
  assert.equal(first.older, true);
  const second = await reopened.notificationPage(first.records.at(-1));
  assert.deepEqual(
    second.records.map((value) => value.id),
    Array.from({ length: 25 }, (_, index) => record(27 - index).id),
  );
  const last = await reopened.notificationPage(second.records.at(-1));
  assert.deepEqual(
    last.records.map((value) => value.id),
    ["002", "001", "000"],
  );
  assert.equal(last.older, false);
  assert.equal(last.newer, true);
  const previous = await reopened.notificationPage(last.records[0], "newer");
  assert.deepEqual(previous.records, second.records);
});

test("Unread And Clear", async () => {
  const history = await import("@/lib/notifications");
  await history.recordNotification(record(0));
  await history.recordNotification(record(1));
  await history.markNotificationsRead();
  assert.ok((await history.notificationPage()).records.every((value) => value.unread === 0));
  await history.recordNotification(record(2));
  assert.deepEqual(
    (await history.notificationPage()).records.map((value) => value.unread),
    [1, 0, 0],
  );
  const write = vi.spyOn(IDBObjectStore.prototype, "put").mockImplementationOnce(() => {
    throw new DOMException("fixture", "QuotaExceededError");
  });
  await assert.rejects(history.recordNotification({ ...record(0), title: "replacement" }), { name: "QuotaExceededError" });
  write.mockRestore();
  assert.equal((await history.notificationPage()).records.find((value) => value.id === "000")?.title, "row-0");
  await history.clearNotifications();
  assert.deepEqual((await history.notificationPage()).records, []);
});

test("Failed Open Retries", async () => {
  const history = await import("@/lib/notifications");
  const factory = indexedDB;
  vi.stubGlobal("indexedDB", {
    open: () => {
      throw new DOMException("fixture", "SecurityError");
    },
  });
  await assert.rejects(history.notificationPage(), { name: "SecurityError" });
  vi.stubGlobal("indexedDB", factory);
  await history.recordNotification(record(0));
  assert.equal((await history.notificationPage()).records.length, 1);
});

test("Sticky Error Delivery", async () => {
  const history = await import("@/lib/notifications");
  vi.spyOn(Date, "now").mockReturnValue(7);
  history.toast.error("fixture", { id: "fixture", source: "fixture-source", description: "fixture-detail", duration: 1 });
  assert.equal(shown.error.mock.calls.at(-1)?.[1]?.duration, Infinity);
  const page = await history.notificationPage();
  assert.equal(page.records.length, 1);
  assert.deepEqual(
    { at: page.records[0].at, source: page.records[0].source, description: page.records[0].description },
    { at: 7, source: "fixture-source", description: "fixture-detail" },
  );
});

test("Updated Toasts Stay Recorded", async () => {
  const history = await import("@/lib/notifications");
  history.toast.info("fixture-one", { id: "fixture" });
  history.toast.success("fixture-two", { id: "fixture" });
  const page = await history.notificationPage();
  assert.deepEqual(page.records.map((value) => value.title).sort(), ["fixture-one", "fixture-two"]);
});

test("Plugin Settings Action Survives History", async () => {
  const history = await import("@/lib/notifications");
  history.toast.error("fixture", {
    pluginSettings: "example",
    missingSecret: "api_token",
    consequences: ["workflow unavailable"],
  });
  const page = await history.notificationPage();
  assert.equal(page.records[0].pluginSettings, "example");
  assert.equal(page.records[0].missingSecret, "api_token");
  assert.deepEqual(page.records[0].consequences, ["workflow unavailable"]);
});

test("Notification Sources Never Use Empty Or Root Route Labels", async () => {
  const history = await import("@/lib/notifications");
  vi.stubGlobal("location", { pathname: "/" });
  for (const source of [undefined, "", " ", "/", " / "]) {
    history.toast.info("fixture", { source });
  }
  const page = await history.notificationPage();
  assert.equal(page.records.length, 5);
  assert.ok(page.records.every((value) => value.source.trim().length > 0 && value.source !== "/"));
});

test("Feedback Is Recorded", async () => {
  const { NotificationMessage } = await import("@/components/NotificationMessage");
  const history = await import("@/lib/notifications");
  assert.equal(
    NotificationMessage({ children: ["fixture-one", createElement("strong", null, "fixture-two")], source: "fixture-source" }),
    null,
  );
  const page = await history.notificationPage();
  assert.equal(page.records.length, 1);
  assert.equal(page.records[0].description, "fixture-one fixture-two");
  assert.equal(page.records[0].source, "fixture-source");
});

test("Quiet Page Notices Do Not Publish Feedback", async () => {
  const { PageNotice } = await import("@/shell/PageShell");
  const history = await import("@/lib/notifications");
  const status = PageNotice({ children: "fixture" });
  assert.equal(status.type, "p");
  assert.equal(status.props.role, "status");
  assert.equal((await history.notificationPage()).records.length, 0);
  assert.equal(shown.info.mock.calls.length, 0);
});

test("Closed Connections Recover", async () => {
  const opened = vi.spyOn(indexedDB, "open");
  const history = await import("@/lib/notifications");
  await history.recordNotification(record(0));
  const connection = opened.mock.results[0].value.result as IDBDatabase;
  const closed = new Promise<void>((resolve) => connection.addEventListener("close", () => resolve(), { once: true }));
  // The Driver Accepts An Instance; Its Declaration Names The Constructor.
  const close = forceCloseDatabase as unknown as (value: IDBDatabase) => void;
  close(connection);
  await closed;
  await history.recordNotification(record(1));
  assert.equal((await history.notificationPage()).records.length, 2);
});

test("Every Toast And History Entry Opens The Retained Details", async () => {
  const history = await import("@/lib/notifications");
  const { NotificationHistoryEntry } = await import("@/features/notifications/NotificationsSurface");
  for (const tone of ["error", "warning", "info", "success"] as const) {
    history.toast[tone]("fixture", { description: "complete message", source: "fixture-source" });
    const trigger = buttonIn(shown[tone].mock.calls.at(-1)?.[0]);
    assert.ok(trigger);
    assert.equal(trigger.props.type, "button");
    assert.equal(trigger.props["aria-haspopup"], "dialog");
    trigger.props.onClick();
    assert.equal(history.getNotificationDetails()?.tone, tone);
    assert.equal(history.getNotificationDetails()?.description, "complete message");
    history.openNotificationDetails(null);
  }
  const page = await history.notificationPage();
  const retained = page.records[0];
  const entry = NotificationHistoryEntry({ record: retained, now: retained.at });
  const trigger = buttonIn(entry);
  assert.ok(trigger);
  trigger.props.onClick();
  assert.deepEqual(history.getNotificationDetails(), retained);
});

test("Details Show Problems And Copy Collapsed Diagnostics Without Debug Formatting", async () => {
  const history = await import("@/lib/notifications");
  const { notificationFailure } = await import("@/lib/notification-details");
  const { NotificationDetailsDialog } = await import("@/features/notifications/NotificationDetailsDialog");
  const copy = vi.fn(async (_text: string) => {});
  vi.stubGlobal("navigator", { clipboard: { writeText: copy } });
  const failure = {
    reason: "validation",
    operation: "Planning.PreviewSelection",
    operationId: "dokime.planning.preview_selection",
    requestId: "request-7",
    invocationId: "invocation-8",
    causes: ["fixture-root", "fixture-wrapper"],
    problems: [
      { label: "Input Voltage", message: "fixture-invalid-number", causes: ["fixture-parser"] },
      { label: "Input File", message: "fixture-file-required" },
    ],
    diagnostics: "fixture-backtrace\nframe-1\nframe-2",
  };
  const error = Object.assign(new Error("fixture-summary"), { notificationFailure: failure });
  error.stack = "RuntimeError { debug_dump: fixture-private-stack }";
  history.toast.error("fixture-title", { description: error.message, error, consequences: ["fixture-dependent"] });
  const retained = (await history.notificationPage()).records[0];
  assert.deepEqual(retained.failure, failure);
  history.openNotificationDetails(retained);
  const markup = renderToStaticMarkup(createElement(NotificationDetailsDialog));
  for (const text of [
    "fixture-summary",
    "fixture-root",
    "fixture-wrapper",
    "Input Voltage",
    "fixture-invalid-number",
    "Input File",
    "request-7",
    "invocation-8",
    "fixture-backtrace",
  ])
    assert.ok(markup.includes(text));
  assert.ok(markup.indexOf("fixture-root") < markup.indexOf("fixture-wrapper"));
  assert.ok(!/<details[^>]*\bopen\b/.test(markup));
  assert.ok(!markup.includes("[object Object]"));
  assert.ok(!markup.includes("Error {"));
  assert.ok(!markup.includes("fixture-private-stack"));
  controls.copy();
  await vi.waitFor(() => assert.equal(copy.mock.calls.length, 1));
  const copied = copy.mock.calls[0][0];
  for (const text of [
    "fixture-backtrace\nframe-1\nframe-2",
    "fixture-dependent",
    "fixture-parser",
    new Date(retained.at).toISOString(),
    "Severity: error",
    retained.title,
    "fixture-summary",
    retained.source,
    failure.reason,
    failure.operation,
    failure.operationId,
    failure.requestId,
    failure.invocationId,
    "fixture-root",
    "fixture-wrapper",
    "fixture-invalid-number",
    "fixture-file-required",
  ])
    assert.ok(copied.includes(text));
  controls.close();
  assert.equal(history.getNotificationDetails(), null);
  const nested = new Error("outer", { cause: new Error("inner", { cause: new Error("root") }) });
  assert.deepEqual(notificationFailure(nested)?.causes, ["root", "inner"]);
});

test("Native Rejections And Captured Diagnostics Reach Notification History", async () => {
  const { create, toBinary } = await import("@bufbuild/protobuf");
  const { StatusSchema, RejectionSchema, CapturedRecordErrorSchema } = await import("@/lib/rpc/generated/errors");
  const { normalizeRpcError } = await import("@/lib/rpc/errors");
  const history = await import("@/lib/notifications");
  const { notificationDetailsText } = await import("@/lib/notification-details");
  const rejected = create(RejectionSchema, {
    code: "fixture-invalid",
    message: "fixture-request-refused",
    issues: [{ path: "parameters.voltage", message: "fixture-out-of-range", expected: "fixture-range" }],
  });
  const captured = create(CapturedRecordErrorSchema, {
    message: "fixture-summary",
    causeChain: [{ message: "fixture-wrapper" }, { message: "fixture-root", code: "fixture-root-code", source: "fixture-root.rs" }],
    source: "fixture.rs:42",
    diagnostics: "fixture-backtrace\nfixture-frame",
  });
  for (const [typeUrl, value] of [
    ["type.googleapis.com/dokime.v1.Rejection", toBinary(RejectionSchema, rejected)],
    ["type.googleapis.com/dokime.v1.CapturedRecordError", toBinary(CapturedRecordErrorSchema, captured)],
  ] as const) {
    const envelope = create(StatusSchema, { code: 3, message: "fixture-request-refused", details: [{ typeUrl, value }] });
    const error = normalizeRpcError({
      kind: "grpc",
      detail: { code: 3, message: "fixture-request-refused", details: Array.from(toBinary(StatusSchema, envelope)) },
    });
    assert.ok(error instanceof Error);
    history.toast.error("fixture-failure", { description: error.message, error });
  }
  const records = (await history.notificationPage()).records;
  const rejection = records.find((record) => record.failure?.reason === "fixture-invalid");
  assert.deepEqual(rejection?.failure?.problems, [
    { label: "parameters.voltage", message: "fixture-out-of-range", causes: ["Expected: fixture-range"] },
  ]);
  const diagnostic = records.find((record) => record.failure?.causes?.length);
  assert.ok(diagnostic);
  assert.deepEqual(diagnostic.failure?.causes, ["fixture-root-code: fixture-root", "fixture-wrapper"]);
  assert.ok(diagnostic.failure?.diagnostics?.includes("fixture-root.rs"));
  assert.ok(notificationDetailsText(diagnostic, "just now").includes(captured.diagnostics ?? "missing"));

  const ipc = await import("@/lib/rpc/ipc");
  const transport = await import("@/lib/rpc/transport");
  const refusal = normalizeRpcError({ kind: "busy" });
  vi.spyOn(ipc, "unary").mockRejectedValueOnce(refusal);
  const request = Object.assign(create(StatusSchema), { requestId: "fixture-request-id", invocationId: "fixture-invocation-id" });
  await assert.rejects(
    transport.unary("/dokime.v1.Execution/StartExecution", StatusSchema, StatusSchema, { generation: "1", isCurrent: () => true }, request),
    (error: unknown) => {
      assert.equal(error, refusal);
      history.toast.error("fixture-admission", { description: "fixture-busy", error });
      return true;
    },
  );
  const admission = (await history.notificationPage()).records.find((record) => record.title === "fixture-admission");
  assert.equal(admission?.failure?.operationId, "dokime.execution.start_execution");
  assert.equal(admission?.failure?.requestId, request.requestId);
  assert.equal(admission?.failure?.invocationId, request.invocationId);
});
