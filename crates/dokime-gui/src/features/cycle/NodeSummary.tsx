/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/** Cycle And Execution Status Beneath The Shared Panel Header. */

import { type ComponentProps, type ReactNode, useState } from "react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { STATUS_PILL_CLASS } from "./cycle-workspace-model";
import { EditableLabel } from "./EditableLabel";

export type NodeSummaryProps = {
  title: string;
  /** Text before the title that names the node, "Execution 1 · ". */
  prefix?: string;
  subject: "Cycle" | "Execution";
  /** Renames the node; `null` keeps the title plain. */
  onRename: ((label: string) => Promise<unknown>) | null;
  status: { label: string; variant: ComponentProps<typeof Badge>["variant"] };
  /** The progress text beside the pill. */
  progress: string | null;
  description?: ReactNode;
  /** False in the frozen report: no pencil. */
  live: boolean;
};

export function NodeSummary({ description, live, onRename, prefix, progress, status, subject, title }: NodeSummaryProps) {
  const [renaming, setRenaming] = useState(false);
  return (
    <section className="grid min-w-0 gap-2" aria-label={`${subject} Status`}>
      {!live ? (
        <h2 className="font-heading text-lg font-semibold">
          {prefix}
          {title}
        </h2>
      ) : null}
      <div className="flex items-center gap-3">
        <Badge className={STATUS_PILL_CLASS} variant={status.variant}>
          {status.label}
        </Badge>
        <span className="text-xs text-muted-foreground">{progress}</span>
        {live && onRename ? (
          <Button className="ml-auto" size="sm" variant="outline" onClick={() => setRenaming(true)}>
            Rename
          </Button>
        ) : null}
      </div>
      {description ? <div className="text-sm text-muted-foreground">{description}</div> : null}
      <Dialog open={renaming} onOpenChange={setRenaming}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Rename {subject}</DialogTitle>
          </DialogHeader>
          {renaming && onRename ? (
            <EditableLabel
              label={title}
              subject={subject}
              initiallyEditing
              onRename={async (label) => {
                await onRename(label);
                setRenaming(false);
              }}
            />
          ) : null}
        </DialogContent>
      </Dialog>
    </section>
  );
}
