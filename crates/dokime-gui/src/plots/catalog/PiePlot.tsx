/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Local Imports === */

import { formatNumber, formatUnitLabel } from "@/lib/format";
import { categoryFractions, categoryTotals } from "@/plots/categories";
import { PlotPlaceholder } from "@/plots/PlotPlaceholder";
import { plotUnavailableText } from "@/plots/plot-placeholder";
import type { PlotCardSpec } from "@/plots/types";

/* === Types === */

/* === Constants === */

const CENTER = 160;
const RADIUS = 130;

/* === Components === */

export function PiePlot({ plot }: { plot: PlotCardSpec }) {
  const totals = categoryTotals(plot);
  const hole = plot.hole ?? 0;
  if (totals.entries === undefined) return <PlotPlaceholder title={plot.title} {...plotUnavailableText(totals.error)} />;
  if (!Number.isFinite(hole) || hole < 0 || hole >= 1)
    return <PlotPlaceholder title={plot.title} {...plotUnavailableText("Pie hole must be finite and in [0, 1).")} />;
  const entries = totals.entries;
  const fractions = categoryFractions(entries);
  if (!fractions.some((fraction) => fraction > 0))
    return <PlotPlaceholder title={plot.title} label="No Positive Total" detail="Category totals sum to zero." />;
  let angle = -Math.PI / 2;
  const sectors = entries.map((entry, index) => {
    const start = angle;
    angle += fractions[index] * Math.PI * 2;
    return { ...entry, fraction: fractions[index], path: sectorPath(start, angle, hole) };
  });
  return (
    <article className="plot-card grid min-w-0 gap-3 overflow-hidden rounded-lg bg-muted/20 p-3" data-testid="pie-plot">
      <h4 className="truncate text-sm font-semibold">{formatUnitLabel(plot.title, plot.unit)}</h4>
      <svg viewBox="0 0 320 320" className="mx-auto aspect-square w-full max-w-80" role="img" aria-label={plot.title}>
        {sectors
          .filter((sector) => sector.fraction > 0)
          .map((sector) => (
            <path key={sector.key} data-category={sector.key} d={sector.path} fill={sector.color} stroke="var(--background)">
              <title>{`${sector.key}: ${formatUnitLabel(formatNumber(sector.value), plot.unit)} · ${formatNumber(sector.fraction * 100)}%`}</title>
            </path>
          ))}
      </svg>
      <ul className="flex flex-wrap gap-x-4 gap-y-1 text-xs">
        {sectors.map((sector) => (
          <li key={sector.key} className="flex items-center gap-2">
            <span className="size-2 shrink-0" style={{ backgroundColor: sector.color }} />
            {sector.key}: {formatUnitLabel(formatNumber(sector.value), plot.unit)}
          </li>
        ))}
      </ul>
    </article>
  );
}

/* === Public Functions === */

/* === Private Functions === */

function sectorPath(start: number, end: number, hole: number): string {
  const middle = (start + end) / 2;
  const position = (angle: number, radius: number) => `${CENTER + Math.cos(angle) * radius},${CENTER + Math.sin(angle) * radius}`;
  const outer = `M${position(start, RADIUS)} A${RADIUS},${RADIUS} 0 0 1 ${position(middle, RADIUS)} A${RADIUS},${RADIUS} 0 0 1 ${position(end, RADIUS)}`;
  if (hole === 0) return `${outer} L${CENTER},${CENTER} Z`;
  const inner = RADIUS * hole;
  return `${outer} L${position(end, inner)} A${inner},${inner} 0 0 0 ${position(middle, inner)} A${inner},${inner} 0 0 0 ${position(start, inner)} Z`;
}
