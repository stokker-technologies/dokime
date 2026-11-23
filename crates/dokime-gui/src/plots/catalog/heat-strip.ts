// Dokime — SPDX-License-Identifier: Apache-2.0
import { cutKey } from "../../lib/rpc/canonical";
import type { PlotPoint, PlotSeries } from "../../lib/types";
import { instantTime } from "../instants";

export function heatStripSample(series: PlotSeries[], instant: string | null): PlotSeries[] {
  const time = instant ? instantTime(instant) : null;
  return series.map((series) => {
    const cells = new Map<string, PlotPoint>();
    for (const point of series.points) {
      const key = cutKey(point.x);
      if (!cells.has(key)) {
        cells.set(key, { x: point.x, y: null, instant });
      }
      const stamp = point.instant ? instantTime(point.instant) : null;
      if (time !== null && stamp === time) {
        cells.set(key, { ...point });
      }
    }
    return { ...series, points: [...cells.values()] };
  });
}
