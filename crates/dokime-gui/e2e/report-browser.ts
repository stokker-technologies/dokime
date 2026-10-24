/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Own Native Report Browser Tabs And Cleanup
 */

/* === Imports === */

// Standard Library Imports
import assert from "node:assert/strict";
import { type ChildProcess, spawn } from "node:child_process";
import { once } from "node:events";
import { realpath } from "node:fs/promises";
import http from "node:http";
import net from "node:net";
import { isAbsolute, join, relative, sep } from "node:path";
import { setTimeout as delay } from "node:timers/promises";
import { pathToFileURL } from "node:url";

// Third Party Imports

// Local Imports

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

type Target = { id: string; url: string; type: string };

/* === Classes === */

export class ReportBrowser {
  private readonly tabs = new Set<string>();
  private accepting = true;
  private readonly process: ChildProcess;
  private readonly debug: string;
  private readonly server: http.Server;
  readonly endpoint: string;

  private constructor(process: ChildProcess, debug: string, server: http.Server, endpoint: string) {
    this.process = process;
    this.debug = debug;
    this.server = server;
    this.endpoint = endpoint;
  }

  static async start(root: string, executable: string): Promise<ReportBrowser> {
    const reserved = net.createServer();
    await new Promise<void>((resolve) => reserved.listen(0, "127.0.0.1", resolve));
    const address = reserved.address();
    assert.ok(address && typeof address === "object");
    const port = address.port;
    await new Promise<void>((resolve, reject) => reserved.close((error) => (error ? reject(error) : resolve())));
    const debug = `http://127.0.0.1:${port}`;
    const child = spawn(
      executable,
      [
        "--headless=new",
        "--disable-gpu",
        "--no-sandbox",
        "--no-first-run",
        `--user-data-dir=${join(root, "report-browser")}`,
        `--remote-debugging-port=${port}`,
        "about:blank",
      ],
      { windowsHide: true, stdio: "ignore" },
    );
    let startup: Error | undefined;
    child.on("error", (error) => {
      startup = error;
    });
    let browser: ReportBrowser | undefined;
    const server = http.createServer(async (request, response) => {
      try {
        assert.ok(browser?.accepting && request.method === "POST" && request.url === "/open");
        let body = "";
        for await (const chunk of request) {
          body += chunk;
          assert.ok(body.length <= 16 * 1024, "report opener request is bounded");
        }
        const requested = JSON.parse(body).path;
        assert.equal(typeof requested, "string");
        assert.ok(isAbsolute(requested));
        const path = await realpath(requested);
        const boundary = await realpath(root);
        const inside = relative(boundary, path);
        assert.ok(inside && inside !== ".." && !inside.startsWith(`..${sep}`) && !isAbsolute(inside));
        assert.ok(path.endsWith(".html"), "the report opener accepts owned HTML reports");
        const url = pathToFileURL(path).href;
        const created = await fetch(`${debug}/json/new?${encodeURIComponent(url)}`, { method: "PUT", signal: AbortSignal.timeout(120000) });
        assert.ok(created.ok);
        const target = (await created.json()) as Target;
        browser.tabs.add(target.id);
        response.writeHead(204).end();
      } catch {
        response.writeHead(400).end("Owned report browser refused the request");
      }
    });
    try {
      const deadline = Date.now() + 120000;
      while (true) {
        if (startup) throw startup;
        if (child.exitCode !== null) throw new Error("Owned report browser exited during startup");
        if (
          await fetch(`${debug}/json/version`, { signal: AbortSignal.timeout(120000) })
            .then((r) => r.ok)
            .catch(() => false)
        )
          break;
        if (Date.now() >= deadline) throw new Error("Owned report browser startup timed out");
        await delay(100);
      }
      await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
      const listening = server.address();
      assert.ok(listening && typeof listening === "object");
      browser = new ReportBrowser(child, debug, server, `http://127.0.0.1:${listening.port}/open`);
      return browser;
    } catch (error) {
      child.kill();
      server.close();
      throw error;
    }
  }

  async closeTabs(): Promise<void> {
    for (const id of this.tabs) {
      const response = await fetch(`${this.debug}/json/close/${encodeURIComponent(id)}`, { signal: AbortSignal.timeout(120000) });
      assert.ok(response.ok, `owned report tab ${id} closed`);
      this.tabs.delete(id);
    }
    const targets = (await fetch(`${this.debug}/json/list`, { signal: AbortSignal.timeout(120000) }).then((r) => r.json())) as Target[];
    assert.ok(!targets.some((target) => target.type === "page" && target.url.startsWith("file:")), "no owned report tab survives cleanup");
  }

  async close(): Promise<void> {
    this.accepting = false;
    await new Promise<void>((resolve, reject) => this.server.close((error) => (error ? reject(error) : resolve())));
    const failures: unknown[] = [];
    try {
      await this.closeTabs();
    } catch (error) {
      failures.push(error);
    }
    // Close Only The Browser Started With This Owner's Unique Profile.
    if (this.process.exitCode === null && this.process.signalCode === null) {
      const exited = once(this.process, "exit");
      const timer = new AbortController();
      let socket: WebSocket | undefined;
      try {
        const version = await fetch(`${this.debug}/json/version`, { signal: AbortSignal.timeout(120000) }).then((r) => r.json());
        socket = new WebSocket(version.webSocketDebuggerUrl);
        await new Promise<void>((resolve, reject) => {
          socket?.addEventListener("open", () => resolve(), { once: true });
          socket?.addEventListener("error", () => reject(new Error("Owned browser control failed")), { once: true });
        });
        socket.send(JSON.stringify({ id: 1, method: "Browser.close" }));
        await Promise.race([
          exited,
          delay(120000, undefined, { signal: timer.signal }).then(() => {
            throw new Error("Owned report browser did not exit");
          }),
        ]);
      } catch (error) {
        this.process.kill();
        failures.push(error);
      } finally {
        timer.abort();
        socket?.close();
      }
    }
    if (failures.length) throw new AggregateError(failures, "Owned report browser cleanup failed");
  }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Entry Point === */
