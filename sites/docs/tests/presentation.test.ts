// Dokime — SPDX-License-Identifier: Apache-2.0

/* === Imports === */

import assert from "node:assert/strict";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { Window } from "happy-dom";
import { act } from "react";
import type { Root } from "react-dom/client";
import { createServer } from "vite";
import workspace from "../../../vite.config.ts";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Public Functions === */

/* === Private Functions === */

/* === Entry Point === */

test("presentation examples mount a populated plot without a desktop host", async (context) => {
  const window = new Window();
  const globals = {
    window,
    document: window.document,
    navigator: window.navigator,
    HTMLElement: window.HTMLElement,
    Element: window.Element,
    Node: window.Node,
    Event: window.Event,
    CustomEvent: window.CustomEvent,
    getComputedStyle: window.getComputedStyle.bind(window),
    ResizeObserver: window.ResizeObserver,
    matchMedia: window.matchMedia.bind(window),
    devicePixelRatio: 1,
    IS_REACT_ACT_ENVIRONMENT: true,
    Path2D: class {
      moveTo() {}
      lineTo() {}
      arc() {}
      rect() {}
      closePath() {}
    },
  };
  const descriptors = new Map(Object.keys(globals).map((key) => [key, Object.getOwnPropertyDescriptor(globalThis, key)]));
  for (const [key, value] of Object.entries(globals)) {
    Object.defineProperty(globalThis, key, { configurable: true, writable: true, value });
  }
  context.mock.method(window.HTMLElement.prototype, "getBoundingClientRect", () => new window.DOMRect(0, 0, 640, 230));
  const strokes: unknown[][] = [];
  const canvas = new Proxy(
    {},
    {
      get: (_, key) => {
        if (key === "measureText") return (text: string) => ({ width: text.length * 7 });
        if (key === "stroke") return (...args: unknown[]) => strokes.push(args);
        return () => undefined;
      },
    },
  );
  context.mock.method(window.HTMLCanvasElement.prototype, "getContext", () => canvas);
  const server = await createServer({
    configFile: false,
    root: fileURLToPath(new URL("..", import.meta.url)),
    resolve: workspace.resolve,
    server: { middlewareMode: true, hmr: false },
    appType: "custom",
  });
  let root: Root | undefined;
  try {
    const { mountPresentationExamples } = await server.ssrLoadModule("/src/components/PresentationExamples.tsx");
    const element = window.document.createElement("div");
    window.document.body.append(element);
    await act(async () => {
      root = mountPresentationExamples(element);
      await window.happyDOM.waitUntilComplete();
    });
    await act(async () => {
      await window.happyDOM.waitUntilComplete();
    });
    assert.equal(element.querySelector("[role=alert]"), null);
    assert.ok(element.querySelector(".uplot canvas"));
    assert.ok(strokes.length > 0);
  } finally {
    await act(async () => root?.unmount());
    await server.close();
    await window.happyDOM.close();
    for (const [key, descriptor] of descriptors) {
      if (descriptor) Object.defineProperty(globalThis, key, descriptor);
      else Reflect.deleteProperty(globalThis, key);
    }
  }
});
