/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Export Payload-Free Desktop Story Spans
 */

/* === Imports === */

// Standard Library Imports
import { AsyncLocalStorage } from "node:async_hooks";
import { closeSync, mkdirSync, openSync, writeSync, fsyncSync } from "node:fs";
import { basename, join } from "node:path";

// Third Party Imports

// Local Imports

/* === Re-Exports === */

/* === Constants === */

const directory = process.env.DOKIME_TEST_TRACE_DIR;
const context = new AsyncLocalStorage();
const started = performance.now();
const epoch = Date.now() * 1000;
let file;
let position = 16;
let count = 0;
let next = 1;
const story = basename(process.argv[1], ".mjs");

/* === Types === */

/* === Classes === */

/* === Public Functions === */

export async function traceSpan(name, category, work) {
  if (file === undefined) return work();
  const id = ++next;
  const parent = context.getStore() ?? 1;
  const begin = now();
  try {
    return await context.run(id, work);
  } finally {
    record(name, category, id, parent, begin);
  }
}

/* === Private Functions === */

function now() {
  return epoch + (performance.now() - started) * 1000;
}

function record(name, category, id, parent, begin) {
  const event = {
    ph: "X",
    name,
    cat: category,
    pid: process.pid,
    tid: id,
    ts: begin,
    dur: now() - begin,
    args: { span_id: id, parent_id: parent },
  };
  const data = Buffer.from(`${count ? ",\n" : ""}${JSON.stringify(event)}`);
  writeSync(file, data, 0, data.length, position);
  position += data.length;
  writeSync(file, Buffer.from("]}"), 0, 2, position);
  count++;
}

/* === Entry Point === */

if (directory) {
  mkdirSync(directory, { recursive: true });
  file = openSync(join(directory, `${story}-${process.pid}.pftrace`), "w");
  const header = Buffer.from('{"traceEvents":[');
  writeSync(file, header);
  position = header.length;
  writeSync(file, Buffer.from("]}"), 0, 2, position);
  process.once("exit", () => {
    record(story, "story", 1, null, epoch);
    fsyncSync(file);
    closeSync(file);
  });
}
