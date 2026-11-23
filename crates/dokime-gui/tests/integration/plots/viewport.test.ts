/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Plot Viewport Interaction
 */

/* === Imports === */

import assert from "node:assert/strict";
import { test, vi } from "vitest";
import { attachPan, pannedRange } from "@/plots/viewport";

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Public Functions === */

/* === Private Functions === */

test("Viewport Panning", () => {
  assert.deepEqual(pannedRange({ min: 0, max: 100 }, -0.25, false), { min: -25, max: 75 });
  const logarithmic = pannedRange({ min: 1, max: 100 }, 0.5, true);
  assert.ok(logarithmic);
  assert.ok(Math.abs(logarithmic.min - 10) <= Number.EPSILON * 10 * 8);
  assert.ok(Math.abs(logarithmic.max - 1000) <= Number.EPSILON * 1000 * 8);
  assert.equal(pannedRange({ min: 0, max: 100 }, 0.5, true), null);
  assert.equal(pannedRange({ min: 1, max: 100 }, Infinity, false), null);
});

test("Pan Listener Ownership", () => {
  const view = new EventTarget();
  const over = Object.assign(new EventTarget(), { clientWidth: 100, clientHeight: 100, ownerDocument: { defaultView: view } });
  const scales = { x: { min: 0, max: 100 }, y: { min: 0, max: 10 } };
  const setScale = vi.fn((axis: "x" | "y", range: { min: number; max: number }) => {
    scales[axis] = range;
  });
  const onPan = vi.fn();
  const setCursor = vi.fn();
  const plot = {
    over,
    scales,
    setScale,
    cursor: { left: 75, top: 70 },
    setCursor,
    batch: (effect: () => void) => effect(),
  } as unknown as Parameters<typeof attachPan>[0];
  const detach = attachPan(plot, { logX: false, logY: false, panY: true, onPan });
  const mouse = (type: string, values: Partial<MouseEvent>) => Object.assign(new Event(type, { cancelable: true }), values);
  const ordinary = mouse("mousedown", { button: 0, shiftKey: false, clientX: 50, clientY: 50 });
  over.dispatchEvent(ordinary);
  assert.equal(ordinary.defaultPrevented, false);
  const shifted = mouse("mousedown", { button: 0, shiftKey: true, clientX: 50, clientY: 50 });
  over.dispatchEvent(shifted);
  assert.equal(shifted.defaultPrevented, true);
  view.dispatchEvent(mouse("mousemove", { buttons: 1, clientX: 75, clientY: 70 }));
  assert.deepEqual(scales, { x: { min: -25, max: 75 }, y: { min: 2, max: 12 } });
  assert.deepEqual(onPan.mock.calls[0][0], { x: scales.x, y: scales.y });
  assert.deepEqual(setCursor.mock.calls[0], [{ left: 75, top: 70 }]);
  view.dispatchEvent(new Event("mouseup"));
  view.dispatchEvent(mouse("mousemove", { buttons: 1, clientX: 90, clientY: 90 }));
  assert.equal(onPan.mock.calls.length, 1);
  over.dispatchEvent(mouse("mousedown", { button: 1, shiftKey: false, clientX: 10, clientY: 50 }));
  view.dispatchEvent(mouse("mousemove", { buttons: 4, clientX: 0, clientY: 50 }));
  assert.deepEqual(scales.x, { min: -15, max: 85 });
  view.dispatchEvent(new Event("blur"));
  assert.equal(onPan.mock.calls.length, 2);
  detach();
  const removed = mouse("mousedown", { button: 1, clientX: 10, clientY: 50 });
  over.dispatchEvent(removed);
  assert.equal(removed.defaultPrevented, false);
  view.dispatchEvent(mouse("mousemove", { buttons: 4, clientX: 0, clientY: 50 }));
  assert.equal(onPan.mock.calls.length, 2);
});

/* === Entry Point === */
