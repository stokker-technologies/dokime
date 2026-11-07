/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Share Marker Kinds And Canvas Glyphs.
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
import type uPlot from "uplot";

// Local Imports
import type { PlotMarker } from "@/lib/types";
import { xValue } from "@/plots/axis";
import { type PromptMarker, promptMarkerLabel } from "@/plots/prompt-markers";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

export type MarkerKind = {
  key: string;
  glyph: "event" | "asked" | "answered";
  moments: Array<{ x: number; label: string }>;
};

export type MarkerTooltip = { key: string; label: string; left: number; width: number };

/* === Classes === */

/* === Public Functions === */

export function markerKinds(markers: readonly PlotMarker[], prompts: readonly PromptMarker[]): MarkerKind[] {
  const kinds = new Map<string, MarkerKind>();
  for (const marker of markers) {
    const x = xValue(marker.x, Number.NaN);
    if (!Number.isFinite(x)) continue;
    const key = `event:${marker.record_type}`;
    const kind = kinds.get(key) ?? { key, glyph: "event", moments: [] };
    kind.moments.push({ x, label: marker.label });
    kinds.set(key, kind);
  }
  for (const marker of prompts) {
    if (!Number.isFinite(marker.x)) continue;
    const key = `prompt:${marker.kind}`;
    const kind = kinds.get(key) ?? {
      key,
      glyph: marker.kind,
      moments: [],
    };
    kind.moments.push({ x: marker.x, label: promptMarkerLabel(marker) });
    kinds.set(key, kind);
  }
  return [...kinds.values()];
}

export function drawMarkerKinds(plot: uPlot, kinds: readonly MarkerKind[], colors: readonly string[]) {
  const { ctx, bbox } = plot;
  const ratio = window.devicePixelRatio || 1;
  const cap = 4 * ratio;
  ctx.save();
  ctx.lineWidth = 1.5 * ratio;
  kinds.forEach((kind, index) => {
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
  left: number,
): { key: string; label: string; left: number } | null {
  let best: { key: string; label: string; left: number } | null = null;
  let distance = 6;
  let event = false;
  for (const kind of kinds) {
    for (const marker of kind.moments) {
      const position = plot.valToPos(marker.x, "x");
      const gap = Math.abs(position - left);
      const preferred = gap < distance || (gap === distance && event && kind.glyph !== "event");
      if (Number.isFinite(position) && position >= 0 && position <= plot.over.clientWidth && preferred) {
        best = { key: kind.key, label: marker.label, left: position };
        distance = gap;
        event = kind.glyph === "event";
      }
    }
  }
  return best;
}

/** Both Marker Kinds Share Hit Testing And A Card-Bounded Tooltip Box. */
export function markerTooltip(plot: uPlot, kinds: readonly MarkerKind[], card: Pick<DOMRect, "left" | "width">): MarkerTooltip | null {
  const cursor = plot.cursor.left;
  if (cursor == null || cursor < 0 || cursor > plot.over.clientWidth) return null;
  const marker = nearestMarker(plot, kinds, cursor);
  if (!marker) return null;

  const inset = Math.min(4, card.width / 2);
  const width = Math.min(256, card.width - 2 * inset);
  const anchor = plot.over.getBoundingClientRect().left - card.left + marker.left;
  const left = Math.max(inset, Math.min(anchor - width / 2, card.width - inset - width));
  return { key: marker.key, label: marker.label, left, width };
}

/* === Private Functions === */

/* === Entry Point === */
