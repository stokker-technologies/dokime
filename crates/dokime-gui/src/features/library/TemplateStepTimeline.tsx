/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The Library's step timeline, with what the lake knows about it (concept 14).
 *
 * The timeline itself is a presentation component shared with the rest of the
 * picker, so the query lives here instead: one template, one aggregate, and
 * the formatted durations handed down as text. */

/* === External Imports === */

import { useMemo } from "react";

/* === Local Imports === */

import { suiteDiagramSteps } from "@/components/suite-picker/model";
import { StepDiagram } from "@/components/suite-picker/StepDiagram";
import { stepDurationLabels, useTemplateStepStatistics } from "@/features/library/step-statistics";
import type { SuiteTemplateEntry } from "@/lib/types";

/* === Components === */

export function TemplateStepTimeline({ template }: { template: SuiteTemplateEntry }) {
  /* The lake keys steps by the template's own identity, not by the identifier
     the picker uses for selection, so the query states `template_id`. */
  const statistics = useTemplateStepStatistics(template.template_id);
  const durations = useMemo(() => stepDurationLabels(statistics), [statistics]);

  return <StepDiagram durations={durations} steps={suiteDiagramSteps(template)} template={template} />;
}
