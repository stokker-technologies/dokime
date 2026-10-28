/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The template's step timeline (concept 10a).
 *
 * Numbered nodes evenly distributed on one hairline rule, the step label
 * centred beneath each, and one shared panel below carrying the selected
 * step's authored description beside the template's facts. No eyebrow and no
 * per-step expander: the timeline says what it is. */

import { useState } from "react";

import { MarkdownText } from "@/components/MarkdownText";
import type { SuiteTemplateEntry } from "@/lib/types";
import { cn } from "@/lib/utils";
import type { DiagramStep } from "./model";

/** Beyond this the timeline scrolls horizontally instead of compressing. */
const SCROLLING_STEP_COUNT = 8;

/* The connector is drawn per cell as two half segments so it stops at the
   node's edge instead of running under the circle. The inset is the node's
   radius; the touch node is larger, so it takes a larger inset. */
const CONNECTOR = "pointer-events-none absolute top-6 border-t [[data-density=touch]_&]:top-6";

export function StepDiagram({
  durations,
  steps,
  template,
}: {
  /** The mean duration to print under a step, already formatted, by step key. */
  durations?: Record<string, string>;
  steps: DiagramStep[];
  template?: SuiteTemplateEntry;
}) {
  const [selectedKey, setSelectedKey] = useState<string | null>(null);
  const selectedIndex = Math.max(
    steps.findIndex((step) => step.key === selectedKey),
    0,
  );
  const selected = steps[selectedIndex] ?? null;
  const description = selected?.description?.trim() ?? "";
  const scrolls = steps.length > SCROLLING_STEP_COUNT;

  if (!steps.length) return null;

  return (
    <section aria-label={`Workflow timeline with ${steps.length} steps`} className="grid gap-3">
      <div className={cn("min-w-0", scrolls && "overflow-x-auto")}>
        <ol
          className={cn("grid items-start pt-3", scrolls && "w-max min-w-full [--step-node-min:7rem]")}
          style={{ gridTemplateColumns: `repeat(${steps.length}, minmax(0, 1fr))` }}
        >
          {steps.map((step, index) => {
            const active = step.key === selected?.key;
            return (
              <li className="relative grid min-w-[var(--step-node-min,0)] justify-items-center" key={step.key}>
                {index > 0 ? <span aria-hidden className={cn(CONNECTOR, "right-1/2 left-0 mr-3 [[data-density=touch]_&]:mr-3")} /> : null}
                {index < steps.length - 1 ? (
                  <span aria-hidden className={cn(CONNECTOR, "right-0 left-1/2 ml-3 [[data-density=touch]_&]:ml-3")} />
                ) : null}
                <button
                  aria-current={active ? "step" : undefined}
                  className={cn(
                    "relative z-10 grid size-6 place-items-center rounded-full border bg-background text-xs font-semibold tabular-nums transition-colors [[data-density=touch]_&]:size-7",
                    active
                      ? "border-primary bg-primary text-primary-foreground ring-2 ring-primary/40"
                      : "border-border text-muted-foreground hover:border-primary/60",
                  )}
                  onClick={() => setSelectedKey(step.key)}
                  type="button"
                >
                  {step.order}
                </button>
                {/* Only the selected label is bold; the rest stay quiet. */}
                <span
                  className={cn(
                    "mt-2 line-clamp-2 px-1 text-center text-xs leading-tight",
                    active ? "font-semibold text-foreground" : "text-muted-foreground",
                  )}
                  title={step.label}
                >
                  {step.label}
                </span>
                {/* The line is reserved whether or not the template has ever
                    passed, so a suite that gains its first statistic does not
                    shift every other timeline on the page. */}
                <span className="mt-1 h-4 text-xs leading-4 text-muted-foreground tabular-nums">{durations?.[step.key] ?? "\u00A0"}</span>
              </li>
            );
          })}
        </ol>
      </div>

      {/* Description left, template facts right; the panel is only as tall as
          the description makes it and the facts align to its top. */}
      <div className="grid min-h-12 items-start gap-4 rounded-lg bg-muted/20 px-3 py-2 md:grid-cols-[minmax(0,1fr)_auto]">
        <div className="min-w-0 max-w-[var(--prose-measure)] text-sm text-muted-foreground [&_li]:leading-[1.5] [&_p]:leading-[1.5]">
          {description ? <MarkdownText text={flowSoftBreaks(description)} /> : <p className="text-sm">This step has no description.</p>}
        </div>
        {template ? <TemplateFacts selectedIndex={selectedIndex} stepCount={steps.length} template={template} /> : null}
      </div>
    </section>
  );
}

/** What the catalog projection knows about the template, on one label/value grid. */
function TemplateFacts({ selectedIndex, stepCount, template }: { selectedIndex: number; stepCount: number; template: SuiteTemplateEntry }) {
  const facts: [string, string][] = [
    ["Steps", `${selectedIndex + 1} of ${stepCount}`],
    ["Parameters", String(template.parameter_definitions.length)],
    ["Files", String(template.file_definitions.length)],
  ];
  return (
    <dl className="grid grid-cols-[auto_auto] items-baseline gap-x-4 gap-y-1 text-xs md:min-w-48">
      {facts.map(([label, value]) => (
        <div className="contents" key={label}>
          <dt className="text-muted-foreground">{label}</dt>
          <dd className="justify-self-end font-mono tabular-nums">{value}</dd>
        </div>
      ))}
    </dl>
  );
}

/**
 * Join the soft line breaks a rustdoc paragraph carries so the text flows to
 * the column width. Blank lines still separate paragraphs and list lines keep
 * their own line, which is what Markdown means by a soft break.
 */
export function flowSoftBreaks(text: string): string {
  return text
    .split(/\n{2,}/)
    .map((block) =>
      block
        .split(/\r?\n/)
        .reduce<string[]>((lines, line) => {
          const trimmed = line.trim();
          const previous = lines[lines.length - 1];
          if (!trimmed) return lines;
          if (
            previous === undefined ||
            trimmed.startsWith("- ") ||
            trimmed.startsWith("* ") ||
            previous.startsWith("- ") ||
            previous.startsWith("* ")
          ) {
            lines.push(trimmed);
            return lines;
          }
          lines[lines.length - 1] = `${previous} ${trimmed}`;
          return lines;
        }, [])
        .join("\n"),
    )
    .join("\n\n");
}
