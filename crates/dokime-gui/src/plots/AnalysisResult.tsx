/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Render Declared Analysis Outputs
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
import { useId, useMemo, useState } from "react";

// Local Imports
import type { EvidenceAnalysisResult } from "@/lib/types";
import { CartesianPlot } from "./CartesianPlot";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */
type AnalysisResultProps = {
  result: EvidenceAnalysisResult;
  title: string;
  stateLabel?: string;
  stateDetail?: string;
};

/* === Components === */
export function AnalysisResult({ result, title, stateLabel, stateDetail }: AnalysisResultProps) {
  const inputId = useId();
  const [selected, setSelected] = useState(0);
  const entries = useMemo(
    () =>
      result.series.flatMap((series) =>
        series.outputs.map((output) => ({
          series,
          output,
          schema: result.definition.outputs.find((schema) => schema.key === output.key),
        })),
      ),
    [result],
  );
  const entry = entries[selected] ?? entries[0];
  if (!entry?.schema) return <p role="alert">Analysis output has no declared presentation.</p>;
  const { series, output, schema } = entry;
  const displayIndices = output.displayIndices?.length ? output.displayIndices : output.axis.map((_, index) => index);
  const label = `${series.label} · ${schema.label}`;
  return (
    <section className="grid gap-2" aria-label={title}>
      <div className="flex flex-wrap items-center gap-2">
        <h3 className="text-base font-semibold">
          {title} · {result.definition.label}
        </h3>
        {entries.length > 1 && (
          <>
            <label htmlFor={inputId}>Output</label>
            <select
              id={inputId}
              value={selected < entries.length ? selected : 0}
              className="rounded-lg border bg-background px-2 py-1 text-sm"
              onChange={(event) => setSelected(Number(event.target.value))}
            >
              {entries.map((item, index) => (
                <option key={`${item.series.key}/${item.output.key}`} value={index}>
                  {item.series.label} · {item.schema?.label ?? item.output.key}
                </option>
              ))}
            </select>
          </>
        )}
      </div>
      {stateLabel && (
        <p role="status" className="text-sm text-muted-foreground">
          {stateLabel}
          {stateDetail ? ` · ${stateDetail}` : ""}
        </p>
      )}
      {schema.axis ? (
        <CartesianPlot
          title={label}
          timeAxis={false}
          xLabel={[schema.axis.key, output.axisUnit].filter(Boolean).join(" · ")}
          unit={output.unit}
          viewportMode="fit"
          series={[
            {
              key: `${series.key}/${output.key}`,
              label,
              unit: output.unit,
              raw_points: output.values.length,
              downsampled: displayIndices.length < output.values.length,
              points: displayIndices.map((index) => ({
                x: output.axis[index],
                y: output.values[index],
              })),
            },
          ]}
        />
      ) : (
        <dl className="grid gap-1">
          <dt className="text-sm text-muted-foreground">{label}</dt>
          <dd className="text-xl tabular-nums">
            {output.values[0]} {output.unit}
          </dd>
        </dl>
      )}
      <p className="text-xs text-muted-foreground">
        {series.inputSamples} source samples · Version {result.selection.version}
        {series.origin ? ` · Origin ${series.origin}` : ""}
      </p>
    </section>
  );
}

/* === Hooks === */

/* === Public Functions === */

/* === Private Functions === */
