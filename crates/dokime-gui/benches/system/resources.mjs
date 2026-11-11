// Dokime — SPDX-License-Identifier: Apache-2.0
import { execFileSync } from "node:child_process";
import { readFile, readdir } from "node:fs/promises";
import { setTimeout as delay } from "node:timers/promises";

const roots = new Set(process.argv.slice(2).map(Number));
const ticks = Number(execFileSync("getconf", ["CLK_TCK"], { encoding: "utf8" }).trim());
const pages = Number(execFileSync("getconf", ["PAGESIZE"], { encoding: "utf8" }).trim());
while (true) {
  const processes = new Map();
  for (const name of await readdir("/proc")) {
    if (!/^\d+$/.test(name)) continue;
    try {
      const raw = await readFile(`/proc/${name}/stat`, "utf8");
      const fields = raw.slice(raw.lastIndexOf(")") + 2).split(" ");
      processes.set(Number(name), {
        parent: Number(fields[1]),
        cpu: (Number(fields[11]) + Number(fields[12])) / ticks,
        memory: Number(fields[21]) * pages,
      });
    } catch (error) {
      if (!["ENOENT", "ESRCH", "EACCES"].includes(error.code)) throw error;
    }
  }
  const owned = new Set(roots);
  let changed = true;
  while (changed) {
    changed = false;
    for (const [pid, entry] of processes) {
      if (!owned.has(pid) && owned.has(entry.parent)) {
        owned.add(pid);
        changed = true;
      }
    }
  }
  const selected = [...processes].filter(([pid]) => owned.has(pid)).map(([, entry]) => entry);
  process.stdout.write(
    `${JSON.stringify({ time_ms: Date.now(), cpu_s: selected.reduce((sum, entry) => sum + entry.cpu, 0), memory_bytes: selected.reduce((sum, entry) => sum + entry.memory, 0) })}\n`,
  );
  await delay(200);
}
