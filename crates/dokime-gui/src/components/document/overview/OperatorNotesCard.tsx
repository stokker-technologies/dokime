/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

import { StickyNote } from "lucide-react";
import { DocumentCard } from "@/components/document/DocumentCard";
import { MarkdownText } from "@/components/MarkdownText";
import { absoluteTimestampLabel } from "@/lib/time-format";
import type { DocumentCardLayout, OperatorNote } from "@/lib/types";
import { relativeTime } from "../document-overview-model";

export function OperatorNotesCard({
  layout,
  notes,
  title = "Operator Notes",
}: {
  layout?: DocumentCardLayout;
  notes: OperatorNote[];
  title?: string;
}) {
  return (
    <DocumentCard
      contentClassName="max-h-[70vh] overflow-y-auto p-4"
      eyebrow="Notes"
      icon={<StickyNote size={20} />}
      layout={layout}
      title={title}
    >
      {notes.length ? (
        <div className="grid gap-3">
          {[...notes].reverse().map((note) => (
            <article className="rounded-lg border bg-muted/25 p-3" key={note.note_id}>
              <time
                className="mb-2 block text-xs font-semibold uppercase text-muted-foreground"
                dateTime={note.timestamp}
                title={absoluteTimestampLabel(note.timestamp)}
              >
                {relativeTime(note.timestamp)} - {absoluteTimestampLabel(note.timestamp)}
              </time>
              <MarkdownText text={note.text} />
            </article>
          ))}
        </div>
      ) : (
        <p className="text-sm text-muted-foreground">No operator notes yet.</p>
      )}
    </DocumentCard>
  );
}
