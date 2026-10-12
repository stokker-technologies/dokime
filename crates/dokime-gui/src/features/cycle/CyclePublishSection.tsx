/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { FileText, Loader2, Package, Plus } from "lucide-react";

/* === Local Imports === */

import { DocumentGrid, DocumentSection } from "@/components/document/Document";
import { DocumentCard } from "@/components/document/DocumentCard";
import { Button } from "@/components/ui/button";

/* === Types === */

type CyclePublishSectionProps = {
  bundleBusy: boolean;
  bundleError: string | null;
  onAddCycleSummaryCard?: () => void;
  onDownloadCycleBundle: () => void;
  onWriteReport: () => void;
  reportBusy: boolean;
};

/* === Components === */

export function CyclePublishSection({
  bundleBusy,
  bundleError,
  onAddCycleSummaryCard,
  onDownloadCycleBundle,
  onWriteReport,
  reportBusy,
}: CyclePublishSectionProps) {
  return (
    <DocumentSection
      description="Cycle-level report actions and export packages."
      title="Publish And Export"
    >
      <DocumentGrid>
        <DocumentCard
          eyebrow="Cycle Summary"
          icon={<FileText size={20} />}
          layout={{
            col_span: { base: 12, md: 12, xl: 12 },
            density: "normal",
            order: 0,
            rail: "main",
            row_span: 1,
          }}
          title="Report Actions"
        >
          <div className="flex flex-wrap gap-2">
            {onAddCycleSummaryCard ? (
              <Button onClick={onAddCycleSummaryCard} variant="outline">
                <Plus data-icon="inline-start" />
                Add Card
              </Button>
            ) : null}
            <Button disabled={reportBusy} onClick={onWriteReport} variant="default">
              <FileText data-icon="inline-start" />
              {reportBusy ? "Generating..." : "Download Report"}
            </Button>
            <Button disabled={bundleBusy} onClick={onDownloadCycleBundle} variant="outline">
              {bundleBusy ? (
                <Loader2 className="animate-spin" data-icon="inline-start" />
              ) : (
                <Package data-icon="inline-start" />
              )}
              {bundleBusy ? "Bundling..." : "Cycle ZIP"}
            </Button>
          </div>
          {bundleError ? (
            <p className="rounded-md border border-destructive/30 bg-destructive/10 p-2 text-xs text-destructive">
              {bundleError}
            </p>
          ) : null}
        </DocumentCard>
      </DocumentGrid>
    </DocumentSection>
  );
}
