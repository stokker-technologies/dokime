/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Select An Admitted Evidence Analysis
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
import { useEffect, useId, useState } from "react";

// Local Imports
import { useNativeSessionOptional } from "@/app/native-session/NativeSessionProvider";
import { editorRow } from "@/components/parameter-editor/model";
import { ParameterValueCell } from "@/components/parameter-editor/parameter-cells";
import { parameterInputId, registeredParameterRaw } from "@/components/start-cycle-form-model";
import { Button } from "@/components/ui/button";
import { listNativeAnalyses } from "@/lib/api/native-plots";
import type {
  DocumentViewPlot,
  EvidenceAnalysisDefinition,
  EvidenceAnalysisSelection,
} from "@/lib/types";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */
type AnalysisControlsProps = {
  plots: DocumentViewPlot[];
  onChange: (plotId: string, selection: EvidenceAnalysisSelection | null) => void;
};

/* === Components === */
export function AnalysisControls({ plots, onChange }: AnalysisControlsProps) {
  const session = useNativeSessionOptional();
  const id = useId();
  const [definitions, setDefinitions] = useState<EvidenceAnalysisDefinition[]>([]);
  const [next, setNext] = useState<string>();
  const [after, setAfter] = useState("");
  const [error, setError] = useState<string>();
  const [loading, setLoading] = useState(false);
  const [plotId, setPlotId] = useState(plots[0]?.plot_id ?? "");
  const [analysisId, setAnalysisId] = useState("");
  useEffect(() => {
    if (!session) return;
    const controller = new AbortController();
    setLoading(true);
    setError(undefined);
    void listNativeAnalyses(session.identity, after, controller.signal)
      .then((page) => {
        if (controller.signal.aborted) return;
        setDefinitions((previous) =>
          after ? [...previous, ...page.definitions] : page.definitions,
        );
        setNext(page.nextAfterId);
      })
      .catch((cause: unknown) => {
        if (!controller.signal.aborted)
          setError(cause instanceof Error ? cause.message : "Analysis discovery failed.");
      })
      .finally(() => {
        if (!controller.signal.aborted) setLoading(false);
      });
    return () => controller.abort();
  }, [session, after]);
  const plot = plots.find((plot) => plot.plot_id === plotId) ?? plots[0];
  const definition = definitions.find((definition) => definition.id === analysisId);
  if (!plot) return null;
  return (
    <details className="mb-3 grid gap-2">
      <summary className="cursor-pointer text-sm font-semibold">Evidence Analysis</summary>
      <div className="grid gap-3 py-2">
        <label htmlFor={`${id}-plot`}>Plot</label>
        <select
          id={`${id}-plot`}
          className="rounded-md border bg-background px-2 py-1"
          value={plot.plot_id}
          onChange={(event) => setPlotId(event.target.value)}
        >
          {plots.map((plot) => (
            <option key={plot.plot_id} value={plot.plot_id}>
              {plot.title}
            </option>
          ))}
        </select>
        <label htmlFor={`${id}-analysis`}>Analysis</label>
        <select
          id={`${id}-analysis`}
          className="rounded-md border bg-background px-2 py-1"
          value={analysisId}
          disabled={loading}
          onChange={(event) => setAnalysisId(event.target.value)}
        >
          <option value="">Original Plot</option>
          {definitions.map((definition) => (
            <option key={definition.id} value={definition.id}>
              {definition.label}
            </option>
          ))}
        </select>
        {error && <p role="alert">{error}</p>}
        {next && (
          <Button type="button" variant="outline" disabled={loading} onClick={() => setAfter(next)}>
            More Analyses
          </Button>
        )}
        {definition ? (
          <AnalysisParameters
            key={`${plot.plot_id}/${definition.id}/${definition.version}`}
            definition={definition}
            selection={plot.analysis}
            onApply={(selection) => onChange(plot.plot_id, selection)}
          />
        ) : (
          <Button type="button" variant="outline" onClick={() => onChange(plot.plot_id, null)}>
            Use Original Plot
          </Button>
        )}
        <p className="text-xs text-muted-foreground">
          Analysis uses the original samples in the selected window. View changes apply to this open
          document.
        </p>
      </div>
    </details>
  );
}

function AnalysisParameters({
  definition,
  selection,
  onApply,
}: {
  definition: EvidenceAnalysisDefinition;
  selection?: EvidenceAnalysisSelection | null;
  onApply: (selection: EvidenceAnalysisSelection) => void;
}) {
  const [raw, setRaw] = useState<Record<string, string>>(() =>
    Object.fromEntries(
      definition.parameters.map((parameter) => {
        const value =
          selection?.id === definition.id && selection.version === definition.version
            ? selection.parameters[parameter.name]
            : undefined;
        return [
          parameter.name,
          value === undefined
            ? registeredParameterRaw(parameter)
            : Array.isArray(value)
              ? JSON.stringify(value)
              : String(value),
        ];
      }),
    ),
  );
  const rows = definition.parameters.map((parameter) =>
    editorRow(parameter, raw[parameter.name] ?? ""),
  );
  const invalid = rows.some((row) => !row.parsed.ok || row.parsed.values.length > 1);
  return (
    <div className="grid gap-2">
      <p className="text-sm text-muted-foreground">{definition.description}</p>
      {rows.map((row) => (
        <div className="grid gap-1" key={row.name}>
          <label htmlFor={parameterInputId(row.definition.parameter_id)}>
            {row.definition.label ?? row.name}
          </label>
          <ParameterValueCell
            row={row}
            onChange={(value) => setRaw((previous) => ({ ...previous, [row.name]: value }))}
          />
          {(!row.parsed.ok || row.parsed.values.length > 1) && (
            <p role="alert">Enter one valid parameter value.</p>
          )}
        </div>
      ))}
      <Button
        type="button"
        disabled={invalid}
        onClick={() => {
          const parameters = Object.fromEntries(
            rows.flatMap((row) =>
              row.parsed.ok && row.parsed.values.length === 1
                ? [[row.name, row.parsed.values[0]]]
                : [],
            ),
          );
          onApply({ id: definition.id, version: definition.version, parameters });
        }}
      >
        Apply Analysis
      </Button>
    </div>
  );
}

/* === Hooks === */

/* === Public Functions === */

/* === Private Functions === */
