/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The points and strategy every plot starts from.
 *
 * A plot its author and its operator left alone draws with these, live and in
 * every report; a plot's own settings popover still overrides them. Both are
 * the connected backend's, because the backend renders the reports.
 */

/* === External Imports === */

import { type ReactNode, useId, useState } from "react";

/* === Local Imports === */

import { useNativeSessionOptional } from "@/app/native-session/NativeSessionProvider";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { SettingsCard } from "@/features/settings/SettingsCard";
import { SettingsFieldStatus } from "@/features/settings/SettingsFieldStatus";
import { SettingsRow } from "@/features/settings/SettingsRow";
import { useAutosave } from "@/features/settings/settings-autosave";
import { NATIVE_PLOT_MAX_POINTS } from "@/lib/api/native-plots";
import {
  applicationSettingsError,
  type ReportDefaults,
  useApplicationSettings,
  useSaveReportDefaults,
} from "@/lib/application-settings";
import type { DocumentDownsampleTechnique } from "@/lib/types";
import { TECHNIQUE_OPTIONS } from "@/plots/PlotSettingsDialog";

/* === Components === */

export function ReportDefaultsCard() {
  const session = useNativeSessionOptional();
  const settings = useApplicationSettings();
  const save = useSaveReportDefaults();
  const autosave = useAutosave<Partial<ReportDefaults>>(
    (patch) => save.mutateAsync(patch),
    applicationSettingsError,
  );
  const canEdit =
    session?.snapshot.connection.backend?.methods.includes(
      "/dokime.v1.Settings/UpdateReportDefaults",
    ) ?? false;
  const view = settings.data;

  if (!session) return null;
  return (
    <SettingsCard
      description="What every plot draws with until its own settings say otherwise, live and in reports."
      title="Report"
    >
      {settings.error ? (
        <SettingsRow label="Status">
          <span className="text-sm text-destructive" role="alert">
            {applicationSettingsError(settings.error)}
          </span>
        </SettingsRow>
      ) : null}
      {view ? (
        <fieldset className="divide-y" disabled={!canEdit}>
          <PointsRow
            onChange={(plotPoints) => autosave.change({ plotPoints })}
            status={<SettingsFieldStatus onRetry={autosave.retry} state={autosave.state} />}
            value={view.report.plotPoints}
          />
          <StrategyRow
            onChange={(plotDownsample) => autosave.commit({ plotDownsample })}
            value={view.report.plotDownsample}
          />
        </fieldset>
      ) : null}
    </SettingsCard>
  );
}

/* === Private Components === */

function PointsRow({
  onChange,
  status,
  value,
}: {
  onChange: (points: number) => void;
  status: ReactNode;
  value: number;
}) {
  const id = useId();
  const [text, setText] = useState(String(value));
  const parsed = parsePoints(text);

  return (
    <SettingsRow
      description={
        parsed === null
          ? `Enter between 1 and ${NATIVE_PLOT_MAX_POINTS.toLocaleString()} points.`
          : "The points a plot keeps when nobody chose how many."
      }
      htmlFor={id}
      label="Default Points"
      status={status}
    >
      <Input
        aria-invalid={parsed === null || undefined}
        className="max-w-40"
        id={id}
        max={NATIVE_PLOT_MAX_POINTS}
        min={1}
        onChange={(event) => {
          setText(event.target.value);
          const next = parsePoints(event.target.value);
          if (next !== null) onChange(next);
        }}
        step={1}
        type="number"
        value={text}
      />
    </SettingsRow>
  );
}

function StrategyRow({
  onChange,
  value,
}: {
  onChange: (technique: DocumentDownsampleTechnique) => void;
  value: DocumentDownsampleTechnique;
}) {
  const id = useId();

  return (
    <SettingsRow
      description="How those points are selected from the samples."
      htmlFor={id}
      label="Default Strategy"
    >
      <Select onValueChange={(next) => onChange(next as DocumentDownsampleTechnique)} value={value}>
        <SelectTrigger className="w-56" id={id}>
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          {TECHNIQUE_OPTIONS.map((option) => (
            <SelectItem key={option.value} value={option.value}>
              {option.label}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    </SettingsRow>
  );
}

/* === Private Functions === */

function parsePoints(text: string): number | null {
  const value = Number(text);
  return Number.isSafeInteger(value) && value >= 1 && value <= NATIVE_PLOT_MAX_POINTS
    ? value
    : null;
}
