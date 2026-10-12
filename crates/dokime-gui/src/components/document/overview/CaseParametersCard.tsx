/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

import { ListChecks } from "lucide-react";
import { CaseParameterGrid } from "@/components/CaseParameterGrid";
import { CollapsibleSection } from "@/components/CollapsibleSection";
import type { DocumentCardLayout, TestCase } from "@/lib/types";

export function CaseParametersCard({
  environment,
  layout,
  testCase,
}: {
  environment?: Record<string, unknown> | null;
  layout?: DocumentCardLayout;
  testCase?: TestCase;
}) {
  const parameterCount =
    Object.keys(testCase?.parameters ?? {}).length + Object.keys(environment ?? {}).length;

  return (
    <CollapsibleSection
      contentClassName="p-3"
      defaultOpen={false}
      eyebrow="Parameters"
      icon={<ListChecks size={20} />}
      layout={layout}
      summary={`${parameterCount} value${parameterCount === 1 ? "" : "s"}`}
      title="Case Parameters"
    >
      <CaseParameterGrid environment={environment} testCase={testCase} />
    </CollapsibleSection>
  );
}
