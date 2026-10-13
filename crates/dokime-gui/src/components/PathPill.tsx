/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

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

export function PathPill({
  value,
  align = "left",
  copy = true,
  compact = false,
  onOpen,
}: PathPillProps) {
  const action = `${compact ? "size-4" : "size-5"} grid shrink-0 place-items-center rounded-full text-muted-foreground hover:bg-background hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/40`;

  async function copyPath() {
    await navigator.clipboard?.writeText(value);
  }

  return (
    <span
      className={`inline-flex min-w-0 max-w-full items-center gap-1 overflow-hidden rounded-full border bg-muted/60 py-0.5 pl-2 pr-1 font-mono text-foreground ring-1 ring-border/40 ${compact ? "w-full text-[0.68rem] leading-4" : "text-xs leading-5"} ${align === "right" ? "justify-self-end" : ""}`}
      title={value}
    >
      <span className="min-w-0 truncate">{compactPath(value)}</span>
      {copy ? (
        <button
          aria-label="Copy full path"
          className={action}
          title="Copy full path"
          type="button"
          onClick={(event) => {
            event.stopPropagation();
            void copyPath();
          }}
        >
          <Clipboard size={12} />
        </button>
      ) : null}
      {onOpen ? (
        <button
          aria-label="Open with default application"
          className={action}
          title="Open with default application"
          type="button"
          onClick={(event) => {
            event.stopPropagation();
            onOpen();
          }}
        >
          <ExternalLink size={12} />
        </button>
      ) : null}
    </span>
  );
}
