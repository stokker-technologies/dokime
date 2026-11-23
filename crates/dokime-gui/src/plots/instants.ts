// Dokime — SPDX-License-Identifier: Apache-2.0
export function instantTime(instant: string): bigint | null {
  const millis = Date.parse(instant);
  if (!Number.isFinite(millis)) {
    return null;
  }
  const fraction = instant.match(/\.(\d+)(?:Z|[+-]\d\d:\d\d)$/)?.[1] ?? "";
  return BigInt(millis) * 1_000_000n + BigInt(fraction.padEnd(9, "0").slice(3, 9) || "0");
}

export function orderedInstants(values: Iterable<string>): string[] {
  const times = new Map<bigint, string>();
  for (const value of values) {
    const time = instantTime(value);
    if (time !== null && !times.has(time)) {
      times.set(time, value);
    }
  }
  return [...times].sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0)).map(([, value]) => value);
}
