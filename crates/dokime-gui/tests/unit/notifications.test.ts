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
import { createElement, isValidElement, type ReactElement, type ReactNode } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { afterEach, beforeEach, test, vi } from "vitest";

// Local Imports
import { Button } from "@dokime/plots/components/ui/button";
import { failureOf } from "@dokime/plots/lib/notification-details";

/* === Re-Exports === */

/* === Constants === */

const shown = vi.hoisted(() => ({
  error: vi.fn((_title: ReactNode, options?: { id?: string | number; duration?: number }) => options?.id ?? "error"),
  info: vi.fn((_title: ReactNode) => "info"),
  success: vi.fn((_title: ReactNode) => "success"),
  warning: vi.fn((_title: ReactNode) => "warning"),
  dismiss: vi.fn(),
}));
const controls = vi.hoisted(() => ({ close: () => {}, copy: () => {}, buttons: [] as (() => void)[] }));
vi.mock("sonner", () => ({ toast: shown }));
vi.mock("react", async (original) => ({
  ...(await original<typeof import("react")>()),
  useId: () => "fixture-feedback",
  useEffect: (effect: () => void) => effect(),
  useSyncExternalStore: (_subscribe: unknown, snapshot: () => unknown) => snapshot(),
}));
vi.mock("@dokime/plots/components/ui/dialog", () => ({
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
vi.mock("@dokime/plots/components/ui/button", () => ({
  Button: ({ children, onClick }: { children: ReactNode; onClick?: () => void }) => {
    if (onClick) controls.buttons.push(onClick);
    if (children === "Copy" && onClick) controls.copy = onClick;
    return createElement("button", { type: "button", onClick }, children);
  },
}));

beforeEach(() => {
  vi.resetModules();
  vi.clearAllMocks();
  controls.buttons.length = 0;
  vi.spyOn(console, "groupCollapsed").mockImplementation(() => {});
  vi.spyOn(console, "groupEnd").mockImplementation(() => {});
  vi.spyOn(console, "error").mockImplementation(() => {});
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
  if (node.type === "button" || node.type === Button) return node as ReturnType<typeof buttonIn>;
  return buttonIn(node.props.children);
}

/* === Entry Point === */

test("History Paging", async () => {
  const history = await import("@dokime/plots/lib/notifications");
  assert.deepEqual((await history.notificationPage()).records, []);
  for (let index = 0; index < 53; index++) await history.recordNotification(record(index));
  vi.resetModules();
  const reopened = await import("@dokime/plots/lib/notifications");
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

test("Unread and Clear", async () => {
  const history = await import("@dokime/plots/lib/notifications");
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

test("Discarded Plugin Notifications Clear After Reopening", async () => {
  const history = await import("@dokime/plots/lib/notifications");
  const failed = { ...record(0), id: "1:failed:boot", pluginSettings: "failed", tone: "error" as const };
  const kept = { ...record(1), id: "1:kept:boot", pluginSettings: "kept", tone: "error" as const };
  const otherSession = { ...record(2), id: "2:remote:boot", pluginSettings: "remote", tone: "error" as const };
  await history.recordNotification(failed);
  await history.recordNotification(kept);
  await history.recordNotification(otherSession);
  await history.recordNotification(record(3));
  vi.resetModules();
  const reopened = await import("@dokime/plots/lib/notifications");
  reopened.openNotificationDetails(failed);
  await reopened.clearPluginNotifications("1", new Set(["kept"]));
  const remaining = (await reopened.notificationPage()).records;
  assert.deepEqual(new Set(remaining.map((row) => row.id)), new Set([kept.id, otherSession.id, record(3).id]));
  assert.deepEqual(shown.dismiss.mock.calls, [[failed.id]]);
  assert.equal(reopened.getNotificationDetails(), null);
});

test("Failed Open Retries", async () => {
  const history = await import("@dokime/plots/lib/notifications");
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
  const history = await import("@dokime/plots/lib/notifications");
  vi.spyOn(Date, "now").mockReturnValue(7);
  history.toast.error(failureOf("fixture-detail", "fixture"), { id: "fixture", source: "fixture-source", duration: 1 });
  assert.equal(shown.error.mock.calls.at(-1)?.[1]?.duration, Infinity);
  const page = await history.notificationPage();
  assert.equal(page.records.length, 1);
  assert.deepEqual(
    { at: page.records[0].at, source: page.records[0].source, description: page.records[0].description },
    { at: 7, source: "fixture-source", description: "fixture-detail" },
  );
});

test("Updated Toasts Stay Recorded", async () => {
  const history = await import("@dokime/plots/lib/notifications");
  history.toast.info("fixture-one", { id: "fixture" });
  history.toast.success("fixture-two", { id: "fixture" });
  const page = await history.notificationPage();
  assert.deepEqual(page.records.map((value) => value.title).sort(), ["fixture-one", "fixture-two"]);
});

test("Plugin Settings Action Survives History", async () => {
  const history = await import("@dokime/plots/lib/notifications");
  history.toast.error(failureOf("fixture", "fixture"), {
    pluginSettings: "example",
    missingSecret: "api_token",
    consequences: ["workflow unavailable"],
  });
  const page = await history.notificationPage();
  assert.equal(page.records[0].pluginSettings, "example");
  assert.equal(page.records[0].missingSecret, "api_token");
  assert.deepEqual(page.records[0].consequences, ["workflow unavailable"]);
});

test("Notification Sources Never Use Empty or Root Route Labels", async () => {
  const history = await import("@dokime/plots/lib/notifications");
  vi.stubGlobal("location", { pathname: "/" });
  for (const source of [undefined, "", " ", "/", " / "]) {
    history.toast.info("fixture", { source });
  }
  const page = await history.notificationPage();
  assert.equal(page.records.length, 5);
  assert.ok(page.records.every((value) => value.source.trim().length > 0 && value.source !== "/"));
});

test("Quiet Page Notices Do Not Publish Feedback", async () => {
  const { PageNotice } = await import("@dokime/document/shell/PageShell");
  const history = await import("@dokime/plots/lib/notifications");
  const status = PageNotice({ children: "fixture" });
  assert.equal(status.type, "p");
  assert.equal(status.props.role, "status");
  assert.equal((await history.notificationPage()).records.length, 0);
  assert.equal(shown.info.mock.calls.length, 0);
});

test("Closed Connections Recover", async () => {
  const opened = vi.spyOn(indexedDB, "open");
  const history = await import("@dokime/plots/lib/notifications");
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

test("Every Toast and History Entry Opens the Retained Details", async () => {
  const history = await import("@dokime/plots/lib/notifications");
  const { NotificationHistoryEntry } = await import("@/features/notifications/NotificationsSurface");
  for (const tone of ["error", "warning", "info", "success"] as const) {
    if (tone === "error") history.toast.error(failureOf("complete message", "fixture"), { source: "fixture-source" });
    else history.toast[tone]("fixture", { description: "complete message", source: "fixture-source" });
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

test("Failure Routes and Console Report Once per Occurrence", async () => {
  const { failureOf, failureText } = await import("@dokime/plots/lib/notification-details");
  const { routeFailure, failureDestinations, reportFailure } = await import("@dokime/plots/lib/failure-routing");
  assert.deepEqual(failureDestinations("view"), { inline: true, toast: false, history: false, console: true });
  assert.deepEqual(failureDestinations("action"), { inline: false, toast: true, history: true, console: true });
  assert.deepEqual(failureDestinations("background"), { inline: false, toast: false, history: true, console: true });
  const group = vi.spyOn(console, "groupCollapsed").mockImplementation(() => {});
  const output = vi.spyOn(console, "error").mockImplementation(() => {});
  const end = vi.spyOn(console, "groupEnd").mockImplementation(() => {});
  const failure = failureOf(
    Object.assign(new Error("fixture-message"), { requestId: "fixture-request", diagnostics: "fixture-span" }),
    "fixture-title",
  );
  routeFailure(failure, "view");
  routeFailure(failure, "action");
  reportFailure(JSON.parse(JSON.stringify(failure)), "fixture-context");
  assert.equal(group.mock.calls.length, 1);
  assert.deepEqual(output.mock.calls, [[failureText(failure)]]);
  assert.equal(end.mock.calls.length, 1);
  reportFailure(failureOf(new Error(failure.message), failure.title), "fixture-context");
  assert.equal(output.mock.calls.length, 2);
});

test("ErrorCard Renders Full Record Retry and Copy Without Toast", async () => {
  const { failureOf, failureText } = await import("@dokime/plots/lib/notification-details");
  const { ErrorCard } = await import("@dokime/plots/components/ErrorCard");
  const failure = failureOf(
    Object.assign(new Error("fixture-message"), { diagnostics: "fixture-span", requestId: "fixture-request" }),
    "fixture-title",
  );
  const retry = vi.fn();
  const copy = vi.fn(async (_text: string) => {});
  vi.stubGlobal("navigator", { clipboard: { writeText: copy } });
  const markup = renderToStaticMarkup(createElement(ErrorCard, { failure, onRetry: retry }));
  for (const value of [failure.title, failure.message, failure.diagnostics, failure.requestId]) assert.ok(value && markup.includes(value));
  assert.ok(markup.includes('role="alert"'));
  assert.ok(!/<details[^>]*\bopen\b/.test(markup));
  controls.buttons[0]();
  assert.equal(retry.mock.calls.length, 1);
  controls.buttons[1]();
  await vi.waitFor(() => assert.deepEqual(copy.mock.calls, [[failureText(failure)]]));
  assert.equal(shown.error.mock.calls.length, 0);
});

test("Backend Recovery Actions Take Precedence Over Generic Retries", async () => {
  const { ErrorCard, ErrorRecoveryContext } = await import("@dokime/plots/components/ErrorCard");
  const { failureRecovery } = await import("@/lib/failure-recovery");
  const assign = vi.fn();
  const retry = vi.fn();
  vi.stubGlobal("window", { location: { assign } });
  for (const [plugin, destination] of [
    ["bench", "/settings/plugins/bench"],
    ["", "/settings"],
  ]) {
    controls.buttons.length = 0;
    assign.mockClear();
    const failure = failureOf({ notificationFailure: { action: "Open Settings", plugin, message: "bench_source" } }, "bench_title");
    renderToStaticMarkup(
      createElement(
        ErrorRecoveryContext.Provider,
        { value: (failure) => failureRecovery(null, failure) },
        createElement(ErrorCard, { failure, onRetry: retry }),
      ),
    );
    controls.buttons[0]();
    assert.deepEqual(assign.mock.calls, [[destination]]);
    assert.equal(retry.mock.calls.length, 0);
  }
});

test("Shared Failure Delivers Unchanged Once to Toast and History", async () => {
  const { failureOf } = await import("@dokime/plots/lib/notification-details");
  const history = await import("@dokime/plots/lib/notifications");
  const failure = failureOf(new Error("fixture-message"), "fixture-title");
  history.toast.error(failure, { context: "background" });
  history.toast.error(failure, { context: "action" });
  assert.equal(shown.error.mock.calls.length, 1);
  assert.deepEqual((await history.notificationPage()).records[0].failure, failure);
});

test("Captured Canonical Library Failure Preserves Every Actionable Field", async () => {
  const { normalizeRpcError } = await import("@/lib/rpc/errors");
  const error = normalizeRpcError({
    kind: "grpc",
    detail: {
      code: 9,
      message: "operation request failed",
      details: Array.from(
        Buffer.from(
          "CAESEnNldHRpbmdzX25vdF9yZWFkeRoUTGlzdCBQbHVnaW4gU2V0dGluZ3MiLVdhaXQgZm9yIHRoZSBwbHVnaW4gdG8gcmVwb3J0IGl0cyBkZXNjcmlwdGlvbioUTGlzdCBQbHVnaW4gU2V0dGluZ3MyBWJlbmNoOj0SE2Rlc2NyaXB0aW9uX3BlbmRpbmcaJmJlbmNoIGhhcyBub3QgcmVwb3J0ZWQgaXRzIGRlc2NyaXB0aW9u",
          "base64",
        ),
      ),
    },
  });
  const failure = failureOf(error, "bench_title");
  assert.equal(failure.reason, "settings_not_ready");
  const { fromBinary } = await import("@bufbuild/protobuf");
  const { FailureSchema } = await import("@/lib/rpc/generated/errors");
  const expected = fromBinary(FailureSchema, Uint8Array.from((error as { detail: { details: number[] } }).detail.details));
  assert.equal(failure.action, expected.action);
  assert.equal(failure.failureOperation, expected.operation);
  assert.equal(failure.message, expected.message);
  assert.equal(failure.plugin, "bench");
  assert.deepEqual(failure.causes, [expected.cause?.message]);
});

test("Product Plain and Undecodable Statuses Keep Code and Message", async () => {
  const { normalizeRpcError } = await import("@/lib/rpc/errors");
  const { create, toBinary } = await import("@bufbuild/protobuf");
  const { FailureSchema } = await import("@/lib/rpc/generated/errors");
  const product = create(FailureSchema, { code: "bench_refused", message: "bench_source", action: "bench_action", plugin: "bench" });
  const failure = failureOf(
    normalizeRpcError({
      kind: "grpc",
      detail: { code: 7, message: "bench_status", details: Array.from(toBinary(FailureSchema, product)) },
    }),
    "bench_title",
  );
  assert.equal(failure.message, product.message);
  assert.equal(failure.action, product.action);
  for (const details of [[], [255]]) {
    const plain = failureOf(normalizeRpcError({ kind: "grpc", detail: { code: 14, message: "bench_transport", details } }), "bench_title");
    assert.equal(plain.reason, "UNAVAILABLE");
    if (details.length) {
      assert.ok(plain.message.includes("bench_transport"));
      assert.ok(plain.message.includes(plain.reason));
    } else assert.equal(plain.message, "bench_transport");
  }
});

test("Failure Presentation Copies Decoded Fields and Omits Client Stack", async () => {
  const { ErrorCard } = await import("@dokime/plots/components/ErrorCard");
  const { failureText } = await import("@dokime/plots/lib/notification-details");
  const error = Object.assign(new Error("bench_source"), {
    notificationFailure: { action: "bench_action", plugin: "bench", causes: ["bench_cause"] },
  });
  const failure = failureOf(error, "bench_title");
  const html = renderToStaticMarkup(createElement(ErrorCard, { failure }));
  for (const field of [failure.message, failure.action, failure.plugin, ...(failure.causes ?? [])])
    assert.ok(field && html.includes(field));
  assert.ok(!html.includes(error.stack ?? "missing_stack"));
  assert.ok(!failureText(failure).includes(error.stack ?? "missing_stack"));
  assert.ok(!/<details[^>]*\bopen\b/.test(html));
  assert.equal(controls.buttons.length, 1);
});

test("Quiet Autosave States Never Render a Failure", async () => {
  const { SettingsFieldStatus } = await import("@/features/settings/SettingsFieldStatus");
  const { ErrorCard } = await import("@dokime/plots/components/ErrorCard");
  const retry = vi.fn();
  for (const kind of ["idle", "saving", "saved"] as const) {
    const content = SettingsFieldStatus({ state: { kind }, onRetry: retry });
    assert.notEqual(content.type, ErrorCard);
  }
  const error = new Error("bench_write");
  const content = SettingsFieldStatus({ state: { kind: "error", message: error.message, failure: error }, onRetry: retry });
  assert.equal(content.type, ErrorCard);
  assert.equal(content.props.failure.message, error.message);
  content.props.onRetry();
  assert.equal(retry.mock.calls.length, 1);
});

test("Description Readiness Remains a Quiet Waiting State", async () => {
  const { ErrorCard } = await import("@dokime/plots/components/ErrorCard");
  const history = await import("@dokime/plots/lib/notifications");
  const failure = failureOf(
    { notificationFailure: { reason: "settings_not_ready", message: "bench_wait", plugin: "bench" } },
    "bench_section",
  );
  const html = renderToStaticMarkup(createElement(ErrorCard, { failure }));
  assert.ok(html.includes('role="status"'));
  assert.ok(!html.includes('role="alert"'));
  assert.ok(html.includes(failure.plugin ?? "missing_plugin"));
  history.toast.error(failure, { context: "action" });
  assert.equal(shown.error.mock.calls.length, 0);
  assert.equal((await history.notificationPage()).records.length, 0);
});

test("Feedback Is Recorded", async () => {
  const history = await import("@dokime/plots/lib/notifications");
  const { ErrorCard } = await import("@dokime/plots/components/ErrorCard");
  const { NotificationDetailsDialog } = await import("@dokime/document/features/notifications/NotificationDetailsDialog");
  const failure = failureOf(new Error("fixture-action"), "fixture-title");
  const write = vi.spyOn(IDBObjectStore.prototype, "put");
  for (let render = 0; render < 2; render++) renderToStaticMarkup(createElement(ErrorCard, { failure, notificationContext: "action" }));
  await vi.waitFor(() => assert.equal(write.mock.calls.length, 1));
  const page = await history.notificationPage();
  assert.deepEqual(
    page.records.map((entry) => entry.failure),
    [failure],
  );
  assert.equal(shown.error.mock.calls.length, 1);
  history.openNotificationDetails(page.records[0]);
  renderToStaticMarkup(createElement(NotificationDetailsDialog));
  renderToStaticMarkup(createElement(ErrorCard, { failure: failureOf(new Error("fixture-load"), "fixture-load-title") }));
  assert.equal(write.mock.calls.length, 1);
  assert.equal(shown.error.mock.calls.length, 1);
});

test("Failure Stem Retains Identity and All Context", async () => {
  const { failureOf } = await import("@dokime/plots/lib/notification-details");
  assert.ok(failureOf(new Error(" "), "fixture-title").message.trim());
  const root = new Error("fixture-root");
  const wrapper = new Error("fixture-wrapper", { cause: root });
  const error = Object.assign(new Error("fixture-message", { cause: wrapper }), {
    operation: "fixture-operation",
    operationId: "fixture-op-id",
    requestId: "fixture-request",
    invocationId: "fixture-invocation",
    diagnostics: "fixture-redacted-span",
    backtrace: "fixture-backtrace",
    problems: [{ label: "fixture-field", message: "fixture-problem", causes: ["fixture-field-root"] }],
  });
  const failure = failureOf(error, "fixture-title");
  assert.equal(failureOf(error, "fixture-other"), failure);
  assert.equal(failureOf(failure, "fixture-other"), failure);
  assert.deepEqual(failure.causes, [root.message, wrapper.message]);
  for (const key of ["operation", "operationId", "requestId", "invocationId", "problems"] as const)
    assert.deepEqual(failure[key], error[key]);
  assert.equal(failure.diagnostics, [error.diagnostics, error.backtrace].join("\n\n"));
  assert.notEqual(failureOf(new Error(error.message), failure.title).id, failure.id);
  assert.equal(failureOf("fixture", "fixture", "fixture-event"), failureOf("fixture", "fixture", "fixture-event"));
});

test("Failure Copy Retains Notification Metadata and Field Problems", async () => {
  const { failureText, notificationDetailsText } = await import("@dokime/plots/lib/notification-details");
  const { NotificationDetailsDialog } = await import("@dokime/document/features/notifications/NotificationDetailsDialog");
  const history = await import("@dokime/plots/lib/notifications");
  const failure = failureOf(
    Object.assign(new Error("fixture-failure"), {
      problems: [{ label: "fixture-field", message: "fixture-problem", causes: ["fixture-field-cause"] }],
      requestId: "fixture-request",
      operationId: "fixture-operation",
      invocationId: "fixture-invocation",
    }),
    "fixture-title",
  );
  const entry = { ...record(2), failure, consequences: ["fixture-consequence"] };
  const copied = notificationDetailsText(entry, "fixture-relative");
  assert.ok(copied.includes(failureText(failure)));
  for (const value of [
    entry.source,
    entry.tone,
    new Date(entry.at).toISOString(),
    "fixture-relative",
    ...entry.consequences,
    "fixture-field",
    "fixture-problem",
    "fixture-field-cause",
    "fixture-request",
    "fixture-operation",
    "fixture-invocation",
  ])
    assert.ok(copied.includes(value));
  const copy = vi.fn(async (_text: string) => {});
  vi.stubGlobal("navigator", { clipboard: { writeText: copy } });
  history.openNotificationDetails(entry);
  const html = renderToStaticMarkup(createElement(NotificationDetailsDialog));
  assert.ok(!/<details[^>]*\bopen\b/.test(html));
  await controls.copy();
  assert.equal(copy.mock.calls.length, 1);
  assert.ok(copy.mock.calls[0][0].includes(failureText(failure)));
  assert.ok(copy.mock.calls[0][0].includes(new Date(entry.at).toISOString()));
});

test("Transport Admission Failures Retain All Request Identifiers", async () => {
  const rejection = new Error("fixture-admission");
  vi.doMock("@/lib/rpc/ipc", () => ({ unary: vi.fn().mockRejectedValue(rejection) }));
  try {
    const { unary } = await import("@/lib/rpc/transport");
    const { FailureSchema } = await import("@/lib/rpc/generated/errors");
    const identifiers = { operationId: "fixture-operation", requestId: "fixture-request", invocationId: "fixture-invocation" };
    const request = { ...identifiers, message: "fixture-request-body" };
    await assert.rejects(
      unary("fixture/admit", FailureSchema, FailureSchema, { generation: "1", isCurrent: () => true }, request),
      (error) => error === rejection,
    );
    const failure = failureOf(rejection, "fixture-title");
    assert.equal(failure.operation, "fixture/admit");
    for (const key of ["operationId", "requestId", "invocationId"] as const) assert.equal(failure[key], identifiers[key]);
  } finally {
    vi.doUnmock("@/lib/rpc/ipc");
  }
});

test("Plugin Settings Can Restart While Retaining the Settings Recovery Record", async () => {
  const { ErrorCard } = await import("@dokime/plots/components/ErrorCard");
  const retry = vi.fn();
  const failure = failureOf({ notificationFailure: { action: "Open Settings", plugin: "bench", message: "fixture-start" } }, "fixture");
  renderToStaticMarkup(createElement(ErrorCard, { failure, onRetry: retry, retryLabel: "Restart Plugin", retryFirst: true }));
  controls.buttons[0]();
  assert.equal(retry.mock.calls.length, 1);
  assert.equal(failure.action, "Open Settings");
});
