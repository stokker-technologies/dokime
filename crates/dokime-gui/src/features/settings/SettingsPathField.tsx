/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* A path, in a field rather than loose under its label.
 *
 * A directory is a value, so it looks like one: a read-only field on the value
 * column, mono for character alignment, truncated at the start so the leaf —
 * the part that differs — stays visible, with the whole path available through
 * the title and, inside the field, a Copy button and — where the path is local
 * to this machine — an Open button that hands it to the file manager.
 */

/* === External Imports === */

import { Check, Copy, FolderOpen } from "lucide-react";
import { useState } from "react";

/* === Local Imports === */

import { Button } from "@/components/ui/button";
import { openLocalPath } from "@/lib/desktop-open";

const GUI_COPIED_INDICATOR_MS = 1_500;

/* === Types === */

type SettingsPathFieldProps = {
  id?: string;
  /** Reveals the path in the file manager; omit it for a path on another host. */
  openable?: boolean;
  /** Shown in muted type when there is no path. */
  placeholder?: string;
  value: string | null;
};

/* === Components === */

export function SettingsPathField({ id, openable = false, placeholder = "Not set", value }: SettingsPathFieldProps) {
  const [copied, setCopied] = useState(false);

  return (
    <div className="flex h-[var(--control-height)] w-full min-w-0 items-center gap-1 rounded-lg border bg-background pr-1 pl-2">
      <span className="min-w-0 flex-1 truncate text-left font-mono text-xs [direction:rtl]" id={id} title={value ?? undefined}>
        {value ? (
          /* bdi keeps the path readable while the box truncates from the left. */
          <bdi>{value}</bdi>
        ) : (
          <span className="text-muted-foreground">{placeholder}</span>
        )}
      </span>
      <Button
        aria-label="Copy Path"
        disabled={!value}
        onClick={() => {
          if (!value) return;
          void navigator.clipboard?.writeText(value).then(() => {
            setCopied(true);
            setTimeout(() => setCopied(false), GUI_COPIED_INDICATOR_MS);
          });
        }}
        size="icon-sm"
        title="Copy Path"
        variant="ghost"
      >
        {copied ? <Check /> : <Copy />}
      </Button>
      {openable ? (
        <Button
          aria-label="Open Path"
          disabled={!value}
          onClick={() => {
            if (!value) return;
            void openLocalPath(value).catch(() => undefined);
          }}
          size="icon-sm"
          title="Open Path"
          variant="ghost"
        >
          <FolderOpen />
        </Button>
      ) : null}
    </div>
  );
}
