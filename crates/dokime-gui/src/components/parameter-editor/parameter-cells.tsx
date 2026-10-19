/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The value cells of the parameter editor (concept 11a).
 *
 * One cell per parameter kind, all of them filling the grid's value column and
 * obeying the same height so the column reads as one rhythm. The WiFi selector
 * keeps its dialog behaviour but now lives inside the cell contract. */

/* === External Imports === */

import { ChevronDown, CircleHelp, Plus, Wifi, X } from "lucide-react";
import { useRef, useState } from "react";

/* === Local Imports === */

import {
  type FileRowState,
  type ParameterChoice,
  parameterInputId,
  type WifiChannelChoice,
  wifiChannelChoices,
  wifiSelectionSummary,
} from "@/components/start-cycle-form-model";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Separator } from "@/components/ui/separator";
import { Switch } from "@/components/ui/switch";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import {
  emptyListItem,
  expandChipValue,
  formatListCandidates,
  type ListChip,
  listCandidates,
  listChips,
  parseParameterInput,
} from "@/lib/parameter-parser";
import type { FileDefinition, ParameterDefinition } from "@/lib/types";
import { cn } from "@/lib/utils";
import { type EditorParameterRow, isExpandedParameter, parameterChoiceList, parameterValueType } from "./model";

/* === Constants === */

/** Every value control matches the row so the value column reads as one rule. */
const CELL_HEIGHT = "h-[calc(var(--parameter-row-height)-0.5rem)]";

/**
 * The gap between a value's right edge and its unit, plus the field's own
 * right inset. The editor sets `--parameter-unit-width` from the widest unit
 * in the template, so every numeric value in the step ends on one edge and
 * every unit starts on one line, whatever the units say.
 */
const UNIT_GAP = "1.25rem";

/**
 * The expression syntaxes a chip or an expanded number accepts, each with one
 * example and what that example generates.
 *
 * Stated as data on one grid because the operator is comparing forms: the
 * question this answers is "which of these gives me half steps", and that is a
 * column to read down rather than five sentences to read through.
 */
const EXPRESSION_REFERENCE: { example: string; result: string; syntax: string }[] = [
  { example: "10, 20, 30", result: "10, 20, 30", syntax: "a, b, c" },
  { example: "1..5", result: "1, 2, 3, 4, 5", syntax: "1..5" },
  { example: "0..2:0.5", result: "0, 0.5, 1, 1.5, 2", syntax: "0..10:0.5" },
  { example: "linspace(0, 1, 3)", result: "0, 0.5, 1", syntax: "linspace(start, stop, n)" },
  { example: "arange(0, 3)", result: "0, 1, 2", syntax: "arange(start, stop[, step])" },
];

/* === Components === */

/** The value control for one parameter row. */
export function ParameterValueCell({ row, onChange }: { row: EditorParameterRow; onChange: (raw: string) => void }) {
  const { definition } = row;
  const inputId = parameterInputId(definition.parameter_id);
  const invalid = !row.parsed.ok;

  switch (row.kind) {
    case "boolean":
      return <BooleanCell inputId={inputId} raw={row.raw} onChange={onChange} />;
    case "list":
      return <ListChipsCell definition={definition} invalid={invalid} raw={row.raw} onChange={onChange} />;
    case "segmented":
      return (
        <SegmentedCell
          choices={parameterChoiceList(definition)}
          multiple={isExpandedParameter(definition)}
          raw={row.raw}
          onChange={onChange}
        />
      );
    case "select":
      return <SelectCell choices={parameterChoiceList(definition)} inputId={inputId} raw={row.raw} onChange={onChange} />;
    case "wifi":
      return (
        <WifiChannelCell
          definition={definition}
          choices={wifiChannelChoices(definition, parameterChoiceList(definition))}
          inputId={inputId}
          raw={row.raw}
          onChange={onChange}
        />
      );
    default:
      return (
        <TextCell
          definition={definition}
          inputId={inputId}
          invalid={invalid}
          numeric={row.kind === "number"}
          raw={row.raw}
          onChange={onChange}
        />
      );
  }
}

/**
 * A plain string, or a number right-aligned in tabular figures with the unit
 * rendered inside the field as a suffix.
 */
export function TextCell({
  definition,
  inputId,
  invalid,
  numeric,
  raw,
  onChange,
}: {
  definition: ParameterDefinition;
  inputId: string;
  invalid: boolean;
  numeric: boolean;
  raw: string;
  onChange: (raw: string) => void;
}) {
  const unit = definition.unit?.trim() ?? "";
  return (
    <div className="relative min-w-0">
      <Input
        aria-invalid={invalid}
        className={cn("w-full", CELL_HEIGHT, numeric && "text-right font-mono tabular-nums")}
        data-testid={`parameter-input-${definition.name}`}
        id={inputId}
        onChange={(event) => onChange(event.target.value)}
        spellCheck={false}
        style={
          numeric
            ? {
                paddingRight: `calc(var(--parameter-unit-width, 3ch) + ${UNIT_GAP})`,
              }
            : undefined
        }
        value={raw}
      />
      {numeric && unit ? (
        <span
          aria-hidden
          className="pointer-events-none absolute inset-y-0 right-2 flex w-[var(--parameter-unit-width,3ch)] items-center truncate font-mono text-xs text-muted-foreground"
        >
          {unit}
        </span>
      ) : null}
      {!numeric && unit ? <span className="sr-only">{unit}</span> : null}
    </div>
  );
}

/** A Switch at the value column's left edge with the state word beside it. */
export function BooleanCell({ inputId, raw, onChange }: { inputId: string; raw: string; onChange: (raw: string) => void }) {
  const checked = raw.trim().toLowerCase() === "true";
  return (
    <div className={cn("flex items-center gap-2", CELL_HEIGHT)}>
      <Switch checked={checked} id={inputId} onCheckedChange={(next) => onChange(next ? "true" : "false")} />
      <Label className="text-sm text-muted-foreground" htmlFor={inputId}>
        {checked ? "On" : "Off"}
      </Label>
    </div>
  );
}

/** Four or fewer choices; `multiple` when the parameter expands into cases. */
export function SegmentedCell({
  choices,
  multiple,
  raw,
  onChange,
}: {
  choices: ParameterChoice[];
  multiple: boolean;
  raw: string;
  onChange: (raw: string) => void;
}) {
  const selected = selectedChoiceValues(raw);
  if (multiple) {
    return (
      <ToggleGroup
        className={cn("flex-wrap", CELL_HEIGHT)}
        onValueChange={(values) =>
          onChange(
            choices
              .map((choice) => choice.value)
              .filter((value) => values.includes(value))
              .join(", "),
          )
        }
        spacing={0}
        type="multiple"
        value={choices.map((choice) => choice.value).filter((value) => selected.has(value))}
        variant="outline"
      >
        {choices.map((choice) => (
          <ToggleGroupItem key={choice.value} value={choice.value}>
            {choice.label}
          </ToggleGroupItem>
        ))}
      </ToggleGroup>
    );
  }
  return (
    <ToggleGroup
      className={cn("flex-wrap", CELL_HEIGHT)}
      onValueChange={(value) => {
        if (value) onChange(value);
      }}
      spacing={0}
      type="single"
      value={raw.trim()}
      variant="outline"
    >
      {choices.map((choice) => (
        <ToggleGroupItem key={choice.value} value={choice.value}>
          {choice.label}
        </ToggleGroupItem>
      ))}
    </ToggleGroup>
  );
}

/** More than four choices on a single-valued parameter. */
export function SelectCell({
  choices,
  inputId,
  raw,
  onChange,
}: {
  choices: ParameterChoice[];
  inputId: string;
  raw: string;
  onChange: (raw: string) => void;
}) {
  return (
    <Select onValueChange={onChange} value={raw.trim()}>
      <SelectTrigger className={cn("w-full", CELL_HEIGHT)} id={inputId}>
        <SelectValue placeholder="Select A Value" />
      </SelectTrigger>
      <SelectContent>
        {choices.map((choice) => (
          <SelectItem key={choice.value} value={choice.value}>
            {choice.label}
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  );
}

/**
 * A list parameter's values as chips inside the field.
 *
 * One chip per value, never a chip holding a comma: a registered default that
 * arrives as one array — `[[100, 50, 10]]`, an ordered ladder that is one case
 * — shows its three values, and the candidate they belong to is preserved in
 * the raw so the preview still counts one case. Each chip removes on its own,
 * the field wraps, and the row grows with it.
 */
export function ListChipsCell({
  definition,
  invalid,
  raw,
  onChange,
}: {
  definition: ParameterDefinition;
  invalid: boolean;
  raw: string;
  onChange: (raw: string) => void;
}) {
  const elementType = (definition.list?.element_type ?? parameterValueType(definition)).toLowerCase();
  const label = definition.label ?? definition.name;
  /* Only an expanded list makes a case per candidate; a single list is one
     case whatever its chips say. */
  const expands = isExpandedParameter(definition);
  const candidates = listCandidates(raw, elementType, expands);
  const chips = listChips(candidates);

  function write(next: unknown[][]) {
    onChange(formatListCandidates(next, expands));
  }

  function replace(chip: ListChip, values: unknown[]) {
    write(
      candidates.map((candidate, position) =>
        position === chip.candidate ? candidate.flatMap((value, index) => (index === chip.index ? values : [value])) : candidate,
      ),
    );
  }

  /* On commit, not on every keystroke: `1..` is a half-typed range and has to
     survive the keystrokes that finish it. A lone chip that becomes several
     values becomes several candidates, which is what the parser reads back;
     inside a longer candidate the values stay in the candidate they were
     typed into. */
  function commit(chip: ListChip, text: string) {
    const expanded = expandChipValue(text, elementType);
    if (expanded && expands && (candidates[chip.candidate]?.length ?? 0) === 1) {
      write(candidates.flatMap((candidate, position) => (position === chip.candidate ? expanded.map((value) => [value]) : [candidate])));
      return;
    }
    if (expanded) {
      replace(chip, expanded);
      return;
    }
    /* Nothing to expand, so this is the normalizing write: whatever the row
       shows as chips becomes the stored shape, and a legacy comma string stops
       being one value the moment it is touched. */
    write(candidates);
  }

  return (
    <div
      aria-invalid={invalid}
      className={cn(
        "flex min-h-[calc(var(--parameter-row-height)-0.5rem)] flex-wrap items-center gap-1.5 rounded-md border bg-background px-1.5 py-1",
        invalid && "border-destructive",
      )}
      data-testid={`parameter-chips-${definition.name}`}
    >
      {chips.map((chip, position) => (
        <span
          className={cn("inline-flex h-6 items-center gap-0.5 rounded-md border bg-muted/40 pl-2", "[[data-density=touch]_&]:h-7")}
          key={`${definition.name}-${chip.candidate}-${chip.index}`}
        >
          <input
            aria-label={`${label} value ${position + 1}`}
            className="min-w-6 bg-transparent py-0.5 font-mono text-xs tabular-nums outline-none focus-visible:ring-2 focus-visible:ring-ring"
            onBlur={(event) => commit(chip, event.target.value)}
            onChange={(event) => replace(chip, [event.target.value])}
            onKeyDown={(event) => {
              if (event.key === "Enter") {
                event.preventDefault();
                commit(chip, event.currentTarget.value);
              }
            }}
            size={Math.max(String(chip.value).length, 1)}
            spellCheck={false}
            value={String(chip.value)}
          />
          {/* The mark stays small; at touch the pseudo-element gives it the
              44px target a finger needs without inflating the chip. */}
          <Button
            aria-label={`Remove ${label} value ${position + 1}`}
            className={cn(
              "relative size-5 rounded-l-none",
              "[[data-density=touch]_&]:after:absolute [[data-density=touch]_&]:after:-inset-2.5 [[data-density=touch]_&]:after:content-['']",
            )}
            onClick={() => replace(chip, [])}
            size="icon-xs"
            type="button"
            variant="ghost"
          >
            <X aria-hidden />
          </Button>
        </span>
      ))}
      <Button
        aria-label={`Add ${label} value`}
        className="size-6 [[data-density=touch]_&]:size-11"
        onClick={() =>
          write(expands ? [...candidates, [emptyListItem(elementType)]] : [[...(candidates[0] ?? []), emptyListItem(elementType)]])
        }
        size="icon-xs"
        type="button"
        variant="ghost"
      >
        <Plus aria-hidden />
      </Button>
    </div>
  );
}

/**
 * The expression help beside a value that accepts one.
 *
 * A popover rather than a tooltip because the operator reads it while typing
 * into the field behind it: it has to stay open, and it has to be reachable
 * from the keyboard. The sentence about cartesian and zip is here rather than
 * under the row because it answers "what do five values mean", which is the
 * question the reference itself provokes.
 */
export function ExpressionReferencePopover() {
  return (
    <Popover>
      <PopoverTrigger asChild>
        <Button
          aria-label="List Expression Help"
          className="size-6 [[data-density=touch]_&]:size-11"
          size="icon-xs"
          type="button"
          variant="ghost"
        >
          <CircleHelp aria-hidden />
        </Button>
      </PopoverTrigger>
      <PopoverContent align="end" className="w-[min(38rem,calc(100vw-2rem))]" data-testid="expression-reference">
        <div className="grid gap-2">
          <p className="font-heading text-sm font-semibold">Expression Reference</p>
          {/* Three content-sized columns: a result list reads as one line, never
              one number per row. */}
          <dl className="grid grid-cols-[auto_auto_auto] items-baseline gap-x-4 gap-y-1 text-xs">
            {EXPRESSION_REFERENCE.map((entry) => (
              <div className="contents" key={entry.syntax}>
                <dt className="font-mono whitespace-nowrap text-foreground">{entry.syntax}</dt>
                <dd className="font-mono whitespace-nowrap text-muted-foreground">{entry.example}</dd>
                <dd className="font-mono whitespace-nowrap tabular-nums text-muted-foreground">{entry.result}</dd>
              </div>
            ))}
          </dl>
          <p className="text-xs text-muted-foreground">
            Cartesian parameters cross: every combination of the expanded lists becomes a case, while zip parameters advance together, one
            case per position.
          </p>
        </div>
      </PopoverContent>
    </Popover>
  );
}

/** One file parameter: the chosen name, then a Choose button. */
export function FileValueCell({
  definition,
  row,
  onPathChange,
  onFileChange,
}: {
  definition: FileDefinition;
  row: FileRowState;
  onPathChange: (path: string) => void;
  onFileChange: (file: File | undefined) => void;
}) {
  const label = definition.label ?? definition.name;
  const chosen = row.path.trim();
  const picker = useRef<HTMLInputElement>(null);
  return (
    <div className={cn("flex items-center gap-2", CELL_HEIGHT)}>
      <Input
        aria-invalid={Boolean(row.error) || (definition.required && !(chosen || definition.default?.trim()))}
        aria-label={`${label} path`}
        className={cn("min-w-0 flex-1 font-mono text-xs", CELL_HEIGHT)}
        disabled={row.uploading}
        id={`${parameterInputId(definition.name)}-file`}
        onChange={(event) => onPathChange(event.target.value)}
        placeholder={definition.default ?? "No file chosen"}
        spellCheck={false}
        value={row.path}
      />
      <input
        accept={definition.media_types?.join(",")}
        aria-label={`${label} file`}
        className="sr-only"
        disabled={row.uploading}
        onChange={(event) => {
          const file = event.target.files?.[0];
          event.target.value = "";
          onFileChange(file);
        }}
        ref={picker}
        tabIndex={-1}
        type="file"
      />
      <Button
        aria-label={`Choose ${label}`}
        disabled={row.uploading}
        onClick={() => picker.current?.click()}
        size="sm"
        type="button"
        variant="outline"
      >
        {row.uploading ? "Choosing…" : "Choose"}
      </Button>
    </div>
  );
}

/**
 * The WiFi stream selector, unchanged in behaviour: the cell is a trigger that
 * summarizes the selection and opens the band and stream picker.
 */
export function WifiChannelCell({
  choices,
  definition,
  inputId,
  raw,
  onChange,
}: {
  choices: WifiChannelChoice[];
  definition: ParameterDefinition;
  inputId: string;
  raw: string;
  onChange: (raw: string) => void;
}) {
  const [open, setOpen] = useState(false);
  const valueType = parameterValueType(definition);
  const parsed = parseParameterInput(raw, {
    ...definition,
    choices: [],
    expansion: { kind: "cartesian" },
    metadata: {},
    value_type: valueType,
    values: [],
  });
  const selected = new Set<string>(
    parsed.ok
      ? parsed.values.map(String)
      : raw
          .split(",")
          .map((entry) => entry.trim())
          .filter(Boolean),
  );
  const selectedChoices = choices.filter((choice) => selected.has(choice.value));

  function setSelected(values: string[]) {
    onChange(
      choices
        .map((choice) => choice.value)
        .filter((value) => values.includes(value))
        .join(", "),
    );
  }

  function toggle(choice: WifiChannelChoice, checked: boolean | "indeterminate") {
    const next = new Set(selected);
    if (checked === true) {
      next.add(choice.value);
    } else {
      next.delete(choice.value);
    }
    setSelected(Array.from(next));
  }

  function selectWhere(predicate: (choice: WifiChannelChoice) => boolean) {
    setSelected(Array.from(new Set<string>([...selected, ...choices.filter(predicate).map((choice) => choice.value)])));
  }

  return (
    <Dialog onOpenChange={setOpen} open={open}>
      <DialogTrigger asChild>
        <Button
          className={cn("w-full justify-between", CELL_HEIGHT)}
          data-testid="wifi-stream-selector-trigger"
          id={inputId}
          type="button"
          variant="outline"
        >
          <span className="flex min-w-0 items-center gap-2">
            <Wifi aria-hidden size={14} />
            <span className="truncate">{wifiSelectionSummary(selectedChoices)}</span>
          </span>
          <ChevronDown aria-hidden size={14} />
        </Button>
      </DialogTrigger>
      <DialogContent className="max-h-[min(88vh,46rem)] overflow-hidden p-0 sm:max-w-3xl">
        <DialogHeader className="px-4 pt-4">
          <DialogTitle>WiFi Channel</DialogTitle>
          <DialogDescription>Select auto-band options or explicit streams for generated WiFi cases.</DialogDescription>
        </DialogHeader>
        <div className="flex flex-wrap gap-2 px-4">
          <Button onClick={() => selectWhere((choice) => choice.band === "2.4 GHz")} size="sm" type="button" variant="outline">
            Select 2.4 GHz
          </Button>
          <Button onClick={() => selectWhere((choice) => choice.band === "5 GHz")} size="sm" type="button" variant="outline">
            Select 5 GHz
          </Button>
          <Button onClick={() => onChange("")} size="sm" type="button" variant="ghost">
            Use Default
          </Button>
        </div>
        <Separator />
        <ScrollArea className="max-h-[min(58vh,28rem)] px-4">
          <div className="grid gap-4 pb-4">
            <WifiChannelGroup
              choices={choices.filter((choice) => choice.mode === "auto")}
              inputId={inputId}
              onToggle={toggle}
              selected={selected}
              title="Auto"
            />
            <WifiChannelGroup
              choices={choices.filter((choice) => choice.mode !== "auto" && choice.band === "2.4 GHz")}
              inputId={inputId}
              onToggle={toggle}
              selected={selected}
              title="2.4 GHz Streams"
            />
            <WifiChannelGroup
              choices={choices.filter((choice) => choice.mode !== "auto" && choice.band === "5 GHz")}
              inputId={inputId}
              onToggle={toggle}
              selected={selected}
              title="5 GHz Streams"
            />
          </div>
        </ScrollArea>
        <DialogFooter>
          <div className="min-w-0 flex-1 text-xs text-muted-foreground">
            {selectedChoices.length ? `${selectedChoices.length} selected` : "Template default"}
          </div>
          <DialogClose asChild>
            <Button type="button">Done</Button>
          </DialogClose>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

export function WifiChannelGroup({
  choices,
  inputId,
  selected,
  title,
  onToggle,
}: {
  choices: WifiChannelChoice[];
  inputId: string;
  selected: Set<string>;
  title: string;
  onToggle: (choice: WifiChannelChoice, checked: boolean | "indeterminate") => void;
}) {
  if (!choices.length) {
    return null;
  }
  return (
    <section className="grid gap-2">
      <div className="flex items-center justify-between gap-3">
        <strong className="text-sm">{title}</strong>
        <Badge variant="secondary">{choices.length}</Badge>
      </div>
      <div className="grid gap-2 [grid-template-columns:repeat(auto-fit,minmax(min(100%,7.5rem),1fr))]">
        {choices.map((choice) => {
          const checked = selected.has(choice.value);
          const choiceId = `${inputId}-${choice.value}`;
          return (
            <Label
              className={cn(
                "grid min-h-12 cursor-pointer grid-cols-[auto_minmax(0,1fr)] items-center gap-2 rounded-md border px-2.5 py-2 text-sm transition-colors",
                checked ? "border-primary bg-primary/5" : "bg-background hover:bg-muted/45",
              )}
              htmlFor={choiceId}
              key={choice.value}
            >
              <Checkbox checked={checked} id={choiceId} onCheckedChange={(next) => onToggle(choice, next)} />
              <span className="min-w-0">
                <span className="block truncate font-medium">{choice.label}</span>
                {choice.band ? (
                  <span className="mt-1 inline-flex">
                    <Badge className="px-1 py-0 text-[10px]" variant="outline">
                      {choice.band}
                    </Badge>
                  </span>
                ) : null}
              </span>
            </Label>
          );
        })}
      </div>
    </section>
  );
}

/* === Private Functions === */

function selectedChoiceValues(raw: string): Set<string> {
  return new Set(
    raw
      .split(",")
      .map((entry) => entry.trim())
      .filter(Boolean),
  );
}
