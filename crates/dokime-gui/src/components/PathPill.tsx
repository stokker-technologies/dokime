/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { Button } from "@/components/ui/button";
import { Clipboard, ExternalLink } from "lucide-react";

/* === Local Imports === */

import { compactPath } from "@/lib/path-display";

/* === Types === */

type PathPillProps = {
  value: string;
  align?: "left" | "right";
  copy?: boolean;
  compact?: boolean;
  /** Opens the path with the system handler; shows an open button beside copy. */
  onOpen?: () => void;
};

/* === Components === */

export function PathPill({ value, align = "left", copy = true, compact = false, onOpen }: PathPillProps) {
  const action = `${compact ? "size-4" : "size-5"} text-muted-foreground hover:bg-background hover:text-foreground`;

  async function copyPath() {
    await navigator.clipboard?.writeText(value);
  }

  return (
    <span
      className={`inline-flex min-w-0 max-w-full items-center gap-1 overflow-hidden rounded-none border bg-muted/60 py-1 pl-2 pr-1 font-mono text-foreground ring-1 ring-border/40 ${compact ? "w-full text-xs leading-4" : "text-xs leading-5"} ${align === "right" ? "justify-self-end" : ""}`}
      title={value}
    >
      <span className="min-w-0 truncate">{compactPath(value)}</span>
      {copy ? (
        <Button
          variant="plain"
          size="inline"
          aria-label="Copy Full Path"
          className={action}
          title="Copy Full Path"
          type="button"
          onClick={(event) => {
            event.stopPropagation();
            void copyPath();
          }}
        >
          <Clipboard size={12} />
        </Button>
      ) : null}
      {onOpen ? (
        <Button
          variant="plain"
          size="inline"
          aria-label="Open with Default Application"
          className={action}
          title="Open with Default Application"
          type="button"
          onClick={(event) => {
            event.stopPropagation();
            onOpen();
          }}
        >
          <ExternalLink size={12} />
        </Button>
      ) : null}
    </span>
  );
}
