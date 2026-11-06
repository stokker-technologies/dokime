/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Measure A Live Document In Chromium Without Wall-Clock Assertions.
 */

/* === Imports === */

// Standard Library Imports
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { once } from "node:events";
import { existsSync } from "node:fs";
import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { pathToFileURL } from "node:url";

// Third Party Imports
import { build } from "vite";
import { test } from "vitest";

// Local Imports

/* === Re-Exports === */

/* === Constants === */

const browserPath =
  process.env.DOKIME_E2E_CHROME ??
  (process.platform === "win32" ? "C:/Program Files/Google/Chrome/Application/chrome.exe" : "google-chrome");

/* === Types === */

type Reply = { id?: number; result?: { result?: { value?: unknown }; exceptionDetails?: unknown; data?: string }; error?: unknown };

/* === Classes === */

/* === Public Functions === */

/* === Private Functions === */

/* === Entry Point === */

test("Live Document Responsiveness", async () => {
  if (process.platform === "win32") assert.ok(existsSync(browserPath), "Chromium is available for canvas proof");
  const scratch = await mkdtemp(join(tmpdir(), "dokime-responsiveness-"));
  let child: ReturnType<typeof spawn> | undefined;
  let socket: WebSocket | undefined;
  try {
    await build({
      configFile: resolve("vite.config.ts"),
      mode: "report",
      logLevel: "error",
      resolve: { alias: [{ find: "react-dom/client", replacement: "react-dom/profiling" }] },
      plugins: [
        {
          name: "fixture-record-transport",
          transform(source, id) {
            if (
              process.env.DOKIME_GUI_PERF_SYNC_RECORDS === "1" &&
              id.replaceAll("\\", "/").endsWith("/src/features/cycle/server-state.ts")
            ) {
              return source.replace("frame ??= window.requestAnimationFrame(flush);", "flush();");
            }
            if (!id.replaceAll("\\", "/").endsWith("/src/lib/api/native-evidence.ts")) return;
            return (
              source
                .replace("export async function fetchNativeRecordTail", "async function originalFetchNativeRecordTail")
                .replace("export async function openNativeRecordStream", "async function originalOpenNativeRecordStream") +
              '\nexport { fetchNativeRecordTail, openNativeRecordStream } from "' +
              resolve("tests/integration/execution/responsiveness-source.ts").replaceAll("\\", "/") +
              '";'
            );
          },
        },
        {
          name: "responsiveness-fixture",
          transformIndexHtml: {
            order: "pre",
            handler: () =>
              '<!doctype html><html><body><div id="root" style="width:1200px"></div><script type="module" src="/tests/integration/execution/responsiveness-fixture.tsx"></script></body></html>',
          },
        },
      ],
      build: { outDir: join(scratch, "frozen"), rolldownOptions: { input: resolve("report.html") } },
    });
    child = spawn(
      browserPath,
      [
        "--headless=new",
        "--no-sandbox",
        "--disable-gpu",
        "--no-first-run",
        "--remote-debugging-port=0",
        `--user-data-dir=${join(scratch, "profile")}`,
        "about:blank",
      ],
      { windowsHide: true, stdio: ["ignore", "ignore", "pipe"] },
    );
    const browser = child;
    let diagnostics = "";
    const endpoint = await new Promise<string>((accept, reject) => {
      const timeout = setTimeout(() => reject(new Error(`Canvas browser startup timed out: ${diagnostics}`)), 30_000);
      browser.once("error", reject);
      browser.once("exit", (code) => {
        clearTimeout(timeout);
        reject(new Error(`Canvas browser exited (${code}): ${diagnostics}`));
      });
      browser.stderr?.on("data", (chunk) => {
        diagnostics += chunk.toString();
        const port = diagnostics.match(/DevTools listening on ws:\/\/127\.0\.0\.1:(\d+)/)?.[1];
        if (port) {
          clearTimeout(timeout);
          accept(`http://127.0.0.1:${port}`);
        }
      });
    });
    const target = await fetch(`${endpoint}/json/new?${encodeURIComponent(pathToFileURL(join(scratch, "frozen/report.html")).href)}`, {
      method: "PUT",
      signal: AbortSignal.timeout(30_000),
    }).then((response) => response.json());
    socket = new WebSocket(target.webSocketDebuggerUrl);
    await once(socket, "open", { signal: AbortSignal.timeout(30_000) });
    const connection = socket;
    let sequence = 0;
    const pending = new Map<number, (reply: Reply) => void>();
    socket.addEventListener("message", (event) => {
      const reply = JSON.parse(event.data) as Reply;
      if (reply.id) {
        pending.get(reply.id)?.(reply);
        pending.delete(reply.id);
      }
    });
    const command = async (method: string, params: Record<string, unknown>) => {
      const id = ++sequence;
      const response = new Promise<Reply>((accept, reject) => {
        const timeout = setTimeout(() => {
          pending.delete(id);
          reject(new Error(`Canvas browser command timed out: ${method}`));
        }, 30_000);
        pending.set(id, (reply) => {
          clearTimeout(timeout);
          accept(reply);
        });
      });
      connection.send(JSON.stringify({ id, method, params }));
      const reply = await response;
      assert.ok(!reply.error && !reply.result?.exceptionDetails, JSON.stringify(reply));
      return reply.result;
    };
    const evaluate = async (expression: string) =>
      (await command("Runtime.evaluate", { expression, awaitPromise: true, returnByValue: true }))?.result?.value;
    await evaluate(
      `new Promise(resolve => { const ready = () => window.responsiveness && document.querySelector("input") ? resolve(true) : requestAnimationFrame(ready); ready(); })`,
    );
    const modes = process.env.DOKIME_GUI_PERF_MODES?.split(",") ?? ["idle", "search", "modal", "ticks"];
    for (const mode of modes) {
      const result = await evaluate(`window.responsiveness.run(${JSON.stringify(mode)})`);
      console.log("RESPONSIVENESS", JSON.stringify(result));
      assert.ok(result);
      if (mode === "search") {
        const sample = result as { records: number; detailReads: number };
        assert.ok(sample.detailReads <= sample.records, "retained immutable records are not serialized again");
      }
      const plots = result as { canvases: number; plotCount: number };
      assert.ok(plots.canvases > 0 && plots.canvases < plots.plotCount, "only nearby live canvases mount");
    }
    const burst = (await evaluate(`window.responsiveness.burst()`)) as {
      updates: number;
      expected: string[];
      actual: string[];
      stableProjection: boolean;
    };
    console.log("RECORD_BURST", JSON.stringify({ pages: burst.expected.length, publications: burst.updates }));
    assert.equal(
      burst.updates,
      process.env.DOKIME_GUI_PERF_SYNC_RECORDS === "1" ? burst.expected.length : 1,
      "frame publication follows the selected workload",
    );
    assert.deepEqual(burst.actual, burst.expected, "every page in the burst is retained");
    assert.ok(burst.stableProjection, "record appends preserve other document projections");
    const scroll = (await evaluate(`window.responsiveness.scroll()`)) as {
      mounted: boolean;
      top: number;
      nextTop: number;
      height: number;
      nextHeight: number;
    };
    assert.ok(scroll.mounted, "scrolling mounts the requested plot");
    assert.equal(scroll.top, scroll.nextTop, "payloads preserve the viewed plot's scroll offset");
    assert.equal(scroll.height, scroll.nextHeight, "off-screen payloads preserve layout height");
    if (process.env.DOKIME_GUI_PERF_SYNC_RECORDS !== "1")
      assert.equal(await evaluate(`window.responsiveness.cancelPending()`), true, "unmount cancels queued publication");
  } finally {
    socket?.close();
    if (child && child.exitCode === null) {
      child.kill();
      await once(child, "exit");
    }
    await rm(scratch, { recursive: true, force: true, maxRetries: 20, retryDelay: 250 });
  }
});
