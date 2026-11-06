/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Prove Sparse Plot Samples On Chromium's Canvas And In Frozen HTML.
 */

/* === Imports === */

// Standard Library Imports
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { once } from "node:events";
import { existsSync } from "node:fs";
import { mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";
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

test("Sparse Samples On Live And Frozen Canvas", async () => {
  if (process.platform === "win32") assert.ok(existsSync(browserPath), "Chromium is available for canvas proof");
  const scratch = await mkdtemp(join(tmpdir(), "dokime-plot-canvas-"));
  let child: ReturnType<typeof spawn> | undefined;
  let socket: WebSocket | undefined;
  try {
    await build({
      configFile: resolve("vite.config.ts"),
      mode: "report",
      logLevel: "error",
      plugins: [
        {
          name: "plot-canvas-fixture",
          transformIndexHtml: {
            order: "pre",
            handler: () =>
              '<!doctype html><html><body><div id="root" style="width:800px"></div><script type="module" src="/tests/integration/plots/canvas-fixture.tsx"></script></body></html>',
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
      `new Promise(resolve => { const ready = () => window.canvasFixture ? resolve(true) : requestAnimationFrame(ready); ready(); })`,
    );
    const results: unknown[] = [];
    for (const zoom of [false, true])
      for (const dark of [false, true])
        for (const faceted of [false, true])
          for (const style of ["points", "line", "line_points", "step"]) {
            const counts = await evaluate(
              `(async () => { await window.canvasFixture.mount(${JSON.stringify(style)}, ${faceted}, ${zoom}, ${dark}); return window.canvasFixture.counts(); })()`,
            );
            results.push({ style, faceted, zoom, dark, counts });
          }
    assert.deepEqual(
      results.filter((result) => !(result as { counts: number[] }).counts.some((count) => count === 20)),
      [],
      "Every style paints all 20 isolated samples before hover",
    );
    for (const zoom of [false, true])
      for (const dark of [false, true]) {
        await evaluate(`window.canvasFixture.mount("points", false, ${zoom}, ${dark}, true)`);
        assert.deepEqual(
          await evaluate(`Array.from(document.querySelectorAll('tbody tr')).slice(2).map(row => row.cells[3].textContent)`),
          ["2", "1", "1", "1"],
        );
        const image = await evaluate(`document.querySelector('canvas').toDataURL()`);
        await evaluate(`window.canvasFixture.toggle(2)`);
        assert.notEqual(await evaluate(`document.querySelector('canvas').toDataURL()`), image);
        assert.deepEqual(
          await evaluate(`Array.from(document.querySelectorAll('tbody tr')).map(row => !!row.querySelector('[data-trace-hidden]'))`),
          [false, false, true, false, false, false],
        );
        assert.deepEqual(await evaluate(`window.canvasFixture.counts()`), [20]);
        await evaluate(`window.canvasFixture.toggle(2)`);
        assert.equal(await evaluate(`document.querySelector('canvas').toDataURL()`), image);
        await evaluate(`window.canvasFixture.toggle(4)`);
        assert.notEqual(await evaluate(`document.querySelector('canvas').toDataURL()`), image);
        await evaluate(`window.canvasFixture.toggle(4)`);
        assert.equal(await evaluate(`document.querySelector('canvas').toDataURL()`), image);
        await evaluate(`window.canvasFixture.toggle(0)`);
        assert.deepEqual(await evaluate(`window.canvasFixture.counts()`), [10]);
        await evaluate(`window.canvasFixture.toggle(0)`);
        assert.deepEqual(await evaluate(`window.canvasFixture.counts()`), [20]);
        const screenshot = await command("Page.captureScreenshot", { format: "png", captureBeyondViewport: true });
        assert.ok(screenshot?.data);
        const evidence = resolve("../../target/tests/plots");
        await mkdir(evidence, { recursive: true });
        await writeFile(
          join(evidence, `${zoom ? "report" : "live"}-${dark ? "dark" : "light"}.png`),
          Buffer.from(screenshot.data, "base64"),
        );
        const area = (await evaluate(
          `(() => { const box = document.querySelector('.u-over').getBoundingClientRect(); return { x: box.left, y: box.top, width: box.width, height: box.height }; })()`,
        )) as { x: number; y: number; width: number; height: number };
        const from = { x: area.x + (area.width * 5) / 19, y: area.y + area.height / 2 };
        const to = { x: area.x + (area.width * 13) / 19, y: from.y };
        await command("Input.dispatchMouseEvent", { type: "mouseMoved", ...from });
        await command("Input.dispatchMouseEvent", { type: "mousePressed", button: "left", clickCount: 1, ...from });
        await command("Input.dispatchMouseEvent", { type: "mouseMoved", buttons: 1, ...to });
        await command("Input.dispatchMouseEvent", { type: "mouseReleased", button: "left", clickCount: 1, ...to });
        await evaluate(`new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve)))`);
        assert.deepEqual(
          await evaluate(`Array.from(document.querySelectorAll('tbody tr')).slice(2).map(row => row.cells[3].textContent)`),
          ["0", "1", "1", "1"],
        );
      }
    assert.equal(await evaluate(`document.querySelectorAll('script[src],link[rel="stylesheet"][href]').length`), 0);
    await command("Browser.close", {});
    if (browser.exitCode === null) await once(browser, "exit", { signal: AbortSignal.timeout(30_000) });
  } finally {
    socket?.close();
    if (child && child.exitCode === null) {
      const exited = once(child, "exit");
      child.kill();
      await exited;
    }
    await rm(scratch, { recursive: true, force: true });
  }
});
