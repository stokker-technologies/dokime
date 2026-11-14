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
import { on, once } from "node:events";
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

type Reply = {
  id?: number;
  result?: {
    result?: { value?: unknown };
    exceptionDetails?: unknown;
    data?: string;
    loaderId?: string;
    errorText?: string;
    sessionId?: string;
  };
  error?: unknown;
};

/* === Classes === */

/* === Public Functions === */

/* === Private Functions === */

/* === Entry Point === */

test("Sparse Samples on Live and Frozen Canvas", async () => {
  if (process.platform === "win32") assert.ok(existsSync(browserPath), "Chromium is available for canvas proof");
  const scratch = await mkdtemp(join(tmpdir(), "dokime-plot-canvas-"));
  let child: ReturnType<typeof spawn> | undefined;
  let closed: Promise<void> | undefined;
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
    closed = new Promise((accept) => browser.once("close", () => accept()));
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
    const target = await fetch(`${endpoint}/json/new?about:blank`, {
      method: "PUT",
      signal: AbortSignal.timeout(30_000),
    })
      .then((response) => response.json())
      .catch((cause) => {
        throw new Error("Open Canvas Browser Target: Request Failed", { cause });
      });
    const version = await fetch(`${endpoint}/json/version`, { signal: AbortSignal.timeout(30_000) })
      .then((response) => response.json())
      .catch((cause) => {
        throw new Error("Read Canvas Browser Connection: Request Failed", { cause });
      });
    socket = new WebSocket(version.webSocketDebuggerUrl);
    await once(socket, "open", { signal: AbortSignal.timeout(30_000) }).catch((cause) => {
      throw new Error("Connect Canvas Browser Target: Socket Did Not Open", { cause });
    });
    const connection = socket;
    let sequence = 0;
    let pageSession: string | undefined;
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
      connection.send(
        JSON.stringify({
          id,
          method,
          params,
          sessionId: method.startsWith("Browser.") || method.startsWith("Target.") ? undefined : pageSession,
        }),
      );
      const reply = await response;
      assert.ok(!reply.error && !reply.result?.exceptionDetails, JSON.stringify(reply));
      return reply.result;
    };
    const evaluate = async (expression: string) =>
      (await command("Runtime.evaluate", { expression, awaitPromise: true, returnByValue: true }))?.result?.value;

    const attached = await command("Target.attachToTarget", { targetId: target.id, flatten: true });
    pageSession = attached?.sessionId;
    assert.ok(pageSession);
    // Attach Before Navigation And Await This Document's Loader.
    const events = on(connection, "message", { signal: AbortSignal.timeout(30_000) });
    await command("Page.enable", {});
    await command("Page.setLifecycleEventsEnabled", { enabled: true });
    const navigation = await command("Page.navigate", { url: pathToFileURL(join(scratch, "frozen/report.html")).href });
    assert.ok(navigation?.loaderId && !navigation.errorText, JSON.stringify(navigation));
    for await (const [event] of events) {
      const message = JSON.parse(event.data);
      if (message.method === "Page.lifecycleEvent" && message.params.name === "load" && message.params.loaderId === navigation.loaderId)
        break;
    }
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
        await evaluate(`window.canvasFixture.mount("points", false, ${zoom}, ${dark})`);
        await evaluate(`window.canvasFixture.toggle(0)`);
        await evaluate(`window.canvasFixture.toggle(1)`);
        assert.deepEqual(await evaluate(`window.canvasFixture.counts()`), [0]);
        const unmarked = await evaluate(`document.querySelector('canvas').toDataURL()`);
        await evaluate(`window.canvasFixture.mount("points", false, ${zoom}, ${dark}, true)`);
        assert.deepEqual(await evaluate(`Array.from(document.querySelectorAll('tbody tr')).map(row => row.cells[3].textContent)`), [
          "10",
          "10",
        ]);
        const image = await evaluate(`document.querySelector('canvas').toDataURL()`);
        await evaluate(`window.canvasFixture.toggle(0)`);
        assert.notEqual(await evaluate(`document.querySelector('canvas').toDataURL()`), image);
        assert.deepEqual(
          await evaluate(`Array.from(document.querySelectorAll('tbody tr')).map(row => !!row.querySelector('[data-trace-hidden]'))`),
          [true, false],
        );
        assert.deepEqual(await evaluate(`window.canvasFixture.counts()`), [10]);
        await evaluate(`window.canvasFixture.toggle(1)`);
        assert.deepEqual(await evaluate(`window.canvasFixture.counts()`), [0]);
        assert.notEqual(await evaluate(`document.querySelector('canvas').toDataURL()`), unmarked);
        await evaluate(`window.canvasFixture.toggle(0)`);
        assert.deepEqual(await evaluate(`window.canvasFixture.counts()`), [10]);
        await evaluate(`window.canvasFixture.toggle(1)`);
        assert.deepEqual(await evaluate(`window.canvasFixture.counts()`), [20]);
        assert.equal(await evaluate(`document.querySelector('canvas').toDataURL()`), image);
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
        await evaluate(`(async () => { for(let i=0;i<8;i++) await new Promise(requestAnimationFrame); })()`);
        assert.deepEqual(await evaluate(`Array.from(document.querySelectorAll('tbody tr')).map(row => row.cells[3].textContent)`), [
          "10",
          "10",
        ]);
      }
    for (const zoom of [false, true]) {
      for (const width of [200, 800]) {
        await evaluate(
          `document.getElementById('root').style.width = '${width}px'; window.canvasFixture.mount("points", false, ${zoom}, false, true)`,
        );
        const area = (await evaluate(
          `(() => { const box = document.querySelector('.u-over').getBoundingClientRect(); return { x: box.left, y: box.top, width: box.width, height: box.height }; })()`,
        )) as { x: number; y: number; width: number; height: number };
        for (const index of [4, 6, 12, 14]) {
          for (const y of [area.y + 1, area.y + area.height - 1]) {
            await command("Input.dispatchMouseEvent", { type: "mouseMoved", x: area.x + (area.width * index) / 19, y });
            const result = await evaluate(`(async () => {
              await (async () => { for(let i=0;i<8;i++) await new Promise(requestAnimationFrame); })();
              const tooltip = document.querySelector('[role="tooltip"]');
              if (!tooltip) return null;
              const box = tooltip.getBoundingClientRect();
              const card = tooltip.parentElement.getBoundingClientRect();
              const first = document.createRange();
              first.setStart(tooltip.firstChild, 0);
              first.setEnd(tooltip.firstChild, 1);
              const start = first.getBoundingClientRect();
              const style = getComputedStyle(tooltip);
              return {
                inside: box.left >= card.left && box.right <= card.right && box.top >= card.top && box.bottom <= card.bottom,
                startVisible: start.left >= box.left && start.right <= box.right && start.top >= box.top && start.bottom <= box.bottom,
                wraps: box.height > parseFloat(style.lineHeight) + parseFloat(style.paddingTop) + parseFloat(style.paddingBottom) + 2,
                truncated: tooltip.scrollHeight > tooltip.clientHeight,
              };
            })()`);
            assert.deepEqual(result, { inside: true, startVisible: true, wraps: true, truncated: true });
          }
        }
      }
    }
    await command("Network.enable", {});
    await command("Network.emulateNetworkConditions", { offline: true, latency: 0, downloadThroughput: 0, uploadThroughput: 0 });
    for (const frozen of [false, true]) {
      await evaluate(`window.canvasFixture.playback(${frozen})`);
      assert.deepEqual(
        await evaluate(`(() => [...document.querySelectorAll('[role="slider"]')].map(node => node.getAttribute('aria-valuenow')))()`),
        ["2", "2"],
      );
      await evaluate(
        `(() => { const slider = document.querySelector('[aria-label="Trajectory Instant"][role="slider"]'); slider.dispatchEvent(new KeyboardEvent('keydown', {key:'Home',code:'Home',bubbles:true})); })()`,
      );
      await evaluate(`(async () => { for(let i=0;i<8;i++) await new Promise(requestAnimationFrame); })()`);
      assert.deepEqual(
        await evaluate(`(() => [...document.querySelectorAll('[role="slider"]')].map(node => node.getAttribute('aria-valuenow')))()`),
        ["0", "2"],
      );
      assert.equal(
        await evaluate(`document.querySelector('[data-trajectory-instant]').getAttribute('data-trajectory-instant')`),
        "2026-01-01T00:00:00Z",
      );
      assert.equal(await evaluate(`document.querySelector('[data-cell]').textContent`), "2");
      const track = await evaluate(
        `(() => { const card = document.querySelector('[data-trajectory-instant]'); return {canvas:card.querySelectorAll('canvas').length, points:window.canvasFixture.counts()[0]}; })()`,
      );
      assert.deepEqual(track, { canvas: 1, points: 1 });
      // A Real Pointer Drag Selects This Grid Without Moving The Trajectory.
      const rect = (await evaluate(
        `(() => {const slider=document.querySelector('[aria-label="Grid Instant"][data-slot="slider"]');slider.scrollIntoView({block:"center"});const box=slider.getBoundingClientRect();return {x:box.left+box.width*0.5,y:box.top+box.height*0.5};})()`,
      )) as { x: number; y: number };
      await command("Input.dispatchMouseEvent", { type: "mousePressed", x: rect.x, y: rect.y, button: "left", clickCount: 1 });
      await command("Input.dispatchMouseEvent", { type: "mouseReleased", x: rect.x, y: rect.y, button: "left", clickCount: 1 });
      await evaluate(`(async () => { for(let i=0;i<8;i++) await new Promise(requestAnimationFrame); })()`);
      assert.deepEqual(
        await evaluate(`(() => [...document.querySelectorAll('[role="slider"]')].map(node => node.getAttribute('aria-valuenow')))()`),
        ["0", "1"],
      );
      assert.equal(await evaluate(`document.querySelector('[data-cell]').textContent`), "1");
    }
    // Summary Marks Must Have Painted Area In Both Themes And Densities.
    for (const dark of [false, true]) {
      for (const touch of [false, true]) {
        await evaluate(`window.canvasFixture.summaries(${dark}, ${touch})`);
        assert.deepEqual(
          await evaluate(
            `(() => [...document.querySelectorAll('[data-testid="contour-plot"]')].map(chart => ({bins:chart.querySelectorAll('rect[data-bin]').length,gaps:chart.querySelectorAll('rect[data-bin="1"],rect[data-bin="2"]').length})))()`,
          ),
          [
            { bins: 2, gaps: 0 },
            { bins: 2, gaps: 0 },
          ],
        );
        assert.equal(
          await evaluate(
            `document.querySelectorAll('[data-testid="grouped-bar-plot"] rect[data-category], [data-testid="stacked-bar-plot"] rect[data-category]').length`,
          ),
          12,
        );
        assert.deepEqual(
          await evaluate(
            `(() => [...document.querySelectorAll('[data-testid="pareto-plot"] circle[data-percent]')].map(node => Number(node.getAttribute('data-percent'))))()`,
          ),
          [37.5, 75, 100],
        );
        const interiors = await evaluate(`(() => [...document.querySelectorAll('[data-pie-hole]')].map(chart => {
          const svg = chart.querySelector('svg');
          const box = svg.viewBox.baseVal;
          const point = new DOMPoint(box.width * 0.51, box.height * 0.513);
          return [...svg.querySelectorAll('path')].some(path => path.isPointInFill(point));
        }))()`);
        assert.deepEqual(interiors, [true, false]);
        const marks = await evaluate(`(() => {
          const selectors = {
            histogram: 'button > span',
            bar: 'div[style*="background-color"]',
            box: 'span[style*="right"]',
            grid: '[data-valid="true"]',
          };
          return Object.entries(selectors).map(([kind, selector]) => {
            const chart = document.querySelector('[data-plot-kind="' + kind + '"]');
            return { kind, marks: [...chart.querySelectorAll(selector)].filter(node => node.style.height !== '0px').map(node => {
              const bounds = node.getBoundingClientRect();
              const color = getComputedStyle(node).backgroundColor;
              const surface = getComputedStyle(chart.querySelector('section, article')).backgroundColor;
              return { area: bounds.width > 0 && bounds.height > 0, filled: color !== 'rgba(0, 0, 0, 0)' && color !== 'transparent' && color !== surface };
            }) };
          });
        })()`);
        assert.deepEqual(
          marks,
          [
            {
              kind: "histogram",
              marks: [
                { area: true, filled: true },
                { area: true, filled: true },
              ],
            },
            {
              kind: "bar",
              marks: [
                { area: true, filled: true },
                { area: true, filled: true },
              ],
            },
            {
              kind: "box",
              marks: [
                { area: true, filled: true },
                { area: true, filled: true },
              ],
            },
            {
              kind: "grid",
              marks: [
                { area: true, filled: true },
                { area: true, filled: true },
              ],
            },
          ],
          `Summary geometry and fills: dark=${dark}, touch=${touch}`,
        );
      }
    }
    assert.equal(await evaluate(`document.querySelectorAll('script[src],link[rel="stylesheet"][href]').length`), 0);
    await command("Browser.close", {});
    connection.close();
    if (browser.exitCode === null)
      await once(browser, "exit", { signal: AbortSignal.timeout(30_000) }).catch((cause) => {
        throw new Error("Close Canvas Browser: Process Did Not Exit After Closing Its Transport", { cause });
      });
  } finally {
    socket?.close();
    if (child && child.exitCode === null && child.signalCode === null) {
      child.kill();
    }
    await closed;
    await rm(scratch, { recursive: true, force: true });
  }
});
