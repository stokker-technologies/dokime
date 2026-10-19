/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/*
 * Per-plot display settings.
 *
 * The gear opens one dialog for one plot: how many points it draws and how the
 * host reduces to that budget. Both are persisted into the execution's frozen
 * document, so the live view and the sealed report render the same frame; the
 * sealed report has no gear and reads what the operator chose.
 */

/* === External Imports === */

import { Settings2 } from "lucide-react";
import { type FormEvent, useId, useState } from "react";

/* === Local Imports === */

import { Button } from "@/components/ui/button";
import { Dialog, DialogClose, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { useReportDefaults } from "@/lib/application-settings";
import type { DocumentDownsampleTechnique } from "@/lib/types";
import type { PlotSettingsButtonProps } from "./plot-settings";
import { NATIVE_PLOT_MAX_POINTS } from "@/plots/reduce";

/* === Constants === */

/** Fewer points than this cannot show a waveform; the host default stays reachable by clearing the field. */
const MIN_DISPLAY_POINTS = 50;
const DISPLAY_POINTS_STEP = 50;

export const TECHNIQUE_OPTIONS: Array<{ value: DocumentDownsampleTechnique; label: string }> = [
  { value: "auto", label: "Auto" },
  { value: "lttb", label: "Largest Triangle (LTTB)" },
  { value: "min_max", label: "Min / Max Buckets" },
  { value: "raw", label: "Raw Samples" },
];

/* === Types === */

type PlotSettingsFormProps = PlotSettingsButtonProps & {
  onClose: () => void;
};

/* === Components === */

export function PlotSettingsButton({ plot, onSubmit }: PlotSettingsButtonProps) {
  const [open, setOpen] = useState(false);
  return (
    <>
      <Button
        aria-label={`Plot settings for ${plot.title}`}
        onClick={() => setOpen(true)}
        size="icon-sm"
        title="Plot settings"
        type="button"
        variant="ghost"
      >
        <Settings2 />
      </Button>
      <Dialog open={open} onOpenChange={setOpen}>
        {open ? <PlotSettingsForm plot={plot} onClose={() => setOpen(false)} onSubmit={onSubmit} /> : null}
      </Dialog>
    </>
  );
}

/** Mounted only while open so every opening starts from the plot's persisted values. */
function PlotSettingsForm({ plot, onClose, onSubmit }: PlotSettingsFormProps) {
  const id = useId();
  const defaults = useReportDefaults();
  const defaultTechnique = TECHNIQUE_OPTIONS.find((option) => option.value === defaults.plotDownsample)?.label;
  /* `auto` on a plot means the plot follows the application default. */
  const options = TECHNIQUE_OPTIONS.map((option) =>
    option.value === "auto" && defaults.plotDownsample !== "auto" ? { ...option, label: `Default (${defaultTechnique})` } : option,
  );
  const [pointsRaw, setPointsRaw] = useState(plot.display_points == null ? "" : String(plot.display_points));
  const [technique, setTechnique] = useState<DocumentDownsampleTechnique>(plot.downsample ?? "auto");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const points = parseDisplayPoints(pointsRaw);
  const invalid = points === undefined;

  const submit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (invalid || busy) return;
    setBusy(true);
    setError(null);
    try {
      await onSubmit({ display_points: points, downsample: technique });
      onClose();
    } catch (cause: unknown) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusy(false);
    }
  };

  return (
    <DialogContent className="sm:max-w-md">
      <form className="grid gap-4" onSubmit={submit}>
        <DialogHeader>
          <DialogTitle>Plot Settings</DialogTitle>
          <DialogDescription>{plot.title}. Saved with the running cycle, so the sealed report draws the same points.</DialogDescription>
        </DialogHeader>

        {/* Label and control share one column pair per row so the two fields
            line up whatever their label length. */}
        <div className="grid grid-cols-[minmax(0,9rem)_minmax(0,1fr)] items-center gap-x-3 gap-y-3">
          <Label htmlFor={`${id}-points`}>Display Points</Label>
          <Input
            aria-invalid={invalid || undefined}
            id={`${id}-points`}
            inputMode="numeric"
            max={NATIVE_PLOT_MAX_POINTS}
            min={0}
            onChange={(event) => setPointsRaw(event.target.value)}
            placeholder={`Default (${defaults.plotPoints.toLocaleString()})`}
            step={DISPLAY_POINTS_STEP}
            type="number"
            value={pointsRaw}
          />
          <Label htmlFor={`${id}-technique`}>Reduction</Label>
          <Select value={technique} onValueChange={(value) => setTechnique(value as DocumentDownsampleTechnique)}>
            <SelectTrigger className="w-full" id={`${id}-technique`}>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {options.map((option) => (
                <SelectItem key={option.value} value={option.value}>
                  {option.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        <p className="text-xs text-muted-foreground">
          {invalid
            ? `Enter between ${MIN_DISPLAY_POINTS} and ${NATIVE_PLOT_MAX_POINTS.toLocaleString()} points, use 0 for every sample, or leave empty for the default.`
            : `Leave the points empty for the host default, or use 0 for every sample; a long live run is compacted near ${NATIVE_PLOT_MAX_POINTS.toLocaleString()} samples per trace.`}
        </p>
        {error ? (
          <p className="text-sm text-destructive" role="alert">
            {error}
          </p>
        ) : null}

        <DialogFooter>
          <DialogClose asChild>
            <Button type="button" variant="outline">
              Cancel
            </Button>
          </DialogClose>
          <Button disabled={invalid || busy} type="submit">
            {busy ? "Saving" : "Apply"}
          </Button>
        </DialogFooter>
      </form>
    </DialogContent>
  );
}

/* === Private Functions === */

/** `null` for the default, a bounded integer, or `undefined` when the text is not one. */
function parseDisplayPoints(raw: string): number | null | undefined {
  const text = raw.trim();
  if (!text) return null;
  const value = Number(text);
  if (!Number.isSafeInteger(value)) return undefined;
  if (value !== 0 && (value < MIN_DISPLAY_POINTS || value > NATIVE_PLOT_MAX_POINTS)) return undefined;
  return value;
}
