/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Share Marker Kinds, Legend Rows And Canvas Glyphs.
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
import type uPlot from "uplot";

// Local Imports
import { SERIES_STATISTICS, type SeriesLegendRow } from "@/lib/statistics";
import type { PlotMarker } from "@/lib/types";
import { xValue } from "@/plots/axis";
import { type PromptMarker, promptMarkerLabel } from "@/plots/prompt-markers";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

export type MarkerKind = {
  key: string;
  label: string;
  glyph: NonNullable<SeriesLegendRow["glyph"]>;
  moments: Array<{ x: number; label: string }>;
};

export type MarkerRange = { min: number; max: number } | null;

export type MarkerTooltip = { label: string; left: number; width: number };

/* === Classes === */

/* === Public Functions === */

export function markerKinds(markers: readonly PlotMarker[], prompts: readonly PromptMarker[]): MarkerKind[] {
  const kinds = new Map<string, MarkerKind>();
  for (const marker of markers) {
    const x = xValue(marker.x, Number.NaN);
    if (!Number.isFinite(x)) continue;
    const key = `event:${marker.record_type}`;
    const kind = kinds.get(key) ?? { key, label: marker.record_type, glyph: "event", moments: [] };
    kind.moments.push({ x, label: marker.label });
    kinds.set(key, kind);
  }
  for (const marker of prompts) {
    if (!Number.isFinite(marker.x)) continue;
    const key = `prompt:${marker.kind}`;
    const kind = kinds.get(key) ?? {
      key,
      label: marker.kind === "asked" ? "Prompt Asked" : "Prompt Answered",
      glyph: marker.kind,
      moments: [],
    };
    kind.moments.push({ x: marker.x, label: promptMarkerLabel(marker) });
    kinds.set(key, kind);
  }
  return [...kinds.values()];
}

export function markerLegendRows(
  kinds: readonly MarkerKind[],
  hidden: ReadonlySet<string>,
  range: MarkerRange,
  colors: readonly string[],
): SeriesLegendRow[] {
  return kinds.map((kind, index) => ({
    key: kind.key,
    label: kind.label,
    color: colors[index] ?? "currentColor",
    glyph: kind.glyph,
    hidden: hidden.has(kind.key),
    xCursor: "",
    yCursor: "",
    cells: Object.fromEntries(
      SERIES_STATISTICS.map(([id]) => [
        id,
        id === "count" ? String(kind.moments.filter(({ x }) => !range || (x >= range.min && x <= range.max)).length) : "",
      ]),
    ) as SeriesLegendRow["cells"],
  }));
}

export function drawMarkerKinds(plot: uPlot, kinds: readonly MarkerKind[], hidden: ReadonlySet<string>, colors: readonly string[]) {
  const { ctx, bbox } = plot;
  const ratio = window.devicePixelRatio || 1;
  const cap = 4 * ratio;
  ctx.save();
  ctx.lineWidth = 1.5 * ratio;
  kinds.forEach((kind, index) => {
    if (hidden.has(kind.key)) return;
    ctx.strokeStyle = colors[index] ?? "currentColor";
    ctx.fillStyle = ctx.strokeStyle;
    for (const marker of kind.moments) {
      const left = plot.valToPos(marker.x, "x", true);
      if (!Number.isFinite(left) || left < bbox.left || left > bbox.left + bbox.width) continue;
      ctx.setLineDash(kind.glyph === "answered" ? [] : [3 * ratio, 3 * ratio]);
      ctx.beginPath();
      ctx.moveTo(left, bbox.top + (kind.glyph === "event" ? 0 : cap));
      ctx.lineTo(left, bbox.top + bbox.height);
      ctx.stroke();
      if (kind.glyph === "event") continue;
      ctx.setLineDash([]);
      ctx.beginPath();
      ctx.moveTo(left - cap, bbox.top);
      ctx.lineTo(left + cap, bbox.top);
      ctx.lineTo(left, bbox.top + cap);
      ctx.closePath();
      if (kind.glyph === "asked") ctx.stroke();
      else ctx.fill();
    }
  });
  ctx.restore();
}

export function nearestMarker(
  plot: uPlot,
  kinds: readonly MarkerKind[],
  hidden: ReadonlySet<string>,
  left: number,
): { label: string; left: number } | null {
  let best: { label: string; left: number } | null = null;
  let distance = 6;
  let event = false;
  for (const kind of kinds) {
    if (hidden.has(kind.key)) continue;
    for (const marker of kind.moments) {
      const position = plot.valToPos(marker.x, "x");
      const gap = Math.abs(position - left);
      const preferred = gap < distance || (gap === distance && event && kind.glyph !== "event");
      if (Number.isFinite(position) && position >= 0 && position <= plot.over.clientWidth && preferred) {
        best = { label: marker.label, left: position };
        distance = gap;
        event = kind.glyph === "event";
      }
    }
  }
  return best;
}

/** Both Marker Kinds Share Hit Testing And A Card-Bounded Tooltip Box. */
export function markerTooltip(
  plot: uPlot,
  kinds: readonly MarkerKind[],
  hidden: ReadonlySet<string>,
  card: Pick<DOMRect, "left" | "width">,
): MarkerTooltip | null {
  const cursor = plot.cursor.left;
  if (cursor == null || cursor < 0 || cursor > plot.over.clientWidth) return null;
  const marker = nearestMarker(plot, kinds, hidden, cursor);
  if (!marker) return null;

  const inset = Math.min(4, card.width / 2);
  const width = Math.min(256, card.width - 2 * inset);
  const anchor = plot.over.getBoundingClientRect().left - card.left + marker.left;
  const left = Math.max(inset, Math.min(anchor - width / 2, card.width - inset - width));
  return { label: marker.label, left, width };
}

/* === Private Functions === */

/* === Entry Point === */
