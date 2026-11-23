// Dokime — SPDX-License-Identifier: Apache-2.0
import type { PlotCartesianFrame } from "../../lib/types";
import { instantTime } from "../instants";

type Frame = PlotCartesianFrame | null | undefined;

/** A faceted frame's samples in source-time order, read once per frame and cut at any of its instants. */
export type XyTimeline = { instants: string[]; until(instant: string | null): Frame };

export function xyTimeline(frame: Frame): XyTimeline {
  const stamps = new Map<bigint, string>();
  const rows = (frame?.mode === "faceted" ? frame.series : []).map((series) =>
    (series.instants ?? [])
      .flatMap((instant, row) => {
        const time = instantTime(instant);
        if (time !== null && !stamps.has(time)) {
          stamps.set(time, instant);
        }
        return time === null ? [] : [{ row, time }];
      })
      .sort((a, b) => (a.time < b.time ? -1 : a.time > b.time ? 1 : 0)),
  );
  const instants = [...stamps].sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0)).map(([, instant]) => instant);
  return {
    instants,
    // A column without instants has nothing to play back and keeps every sample.
    until(instant) {
      const limit = instant === null ? null : instantTime(instant);
      if (!frame || limit === null || !instants.length) {
        return frame;
      }
      const series = frame.series.map((column, index) => {
        if (!column.instants) {
          return column;
        }
        const shown = rows[index].filter((row) => row.time <= limit);
        return { x: shown.map(({ row }) => column.x[row]), y: shown.map(({ row }) => column.y[row]) };
      });
      return {
        ...frame,
        series,
        point_count: series.reduce((count, column) => count + column.y.length, 0),
        signature: `${frame.signature}@${instant}`,
      };
    },
  };
}
