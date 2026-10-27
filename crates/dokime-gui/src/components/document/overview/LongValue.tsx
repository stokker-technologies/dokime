/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

import { Clipboard } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { ValueDisplay } from "@/components/ValueDisplay";
import { isPathLike } from "@/lib/path-display";

export function LongValue({ alwaysCopy = false, label = "", value }: { alwaysCopy?: boolean; label?: string; value: string }) {
  const valueRef = useRef<HTMLSpanElement>(null);
  const [isTruncated, setIsTruncated] = useState(false);

  useEffect(() => {
    const element = valueRef.current;
    if (!element) {
      return;
    }

    function updateTruncation() {
      const current = valueRef.current;
      setIsTruncated(current === null ? false : current.scrollWidth > current.clientWidth);
    }

    updateTruncation();
    const observer = typeof ResizeObserver === "undefined" ? null : new ResizeObserver(updateTruncation);
    observer?.observe(element);
    window.addEventListener("resize", updateTruncation);
    return () => {
      observer?.disconnect();
      window.removeEventListener("resize", updateTruncation);
    };
  }, []);

  async function copyValue() {
    await navigator.clipboard?.writeText(value);
  }

  const normalized = value.trim().toLowerCase();
  if (normalized === "true" || normalized === "false") {
    const enabled = normalized === "true";
    return (
      <span
        className={`inline-flex rounded-full px-2 py-1 text-xs font-semibold ${enabled ? "bg-success/10 text-success ring-1 ring-success/20" : "bg-destructive/10 text-destructive ring-1 ring-destructive/20"}`}
        title={value}
      >
        {enabled ? "True" : "False"}
      </span>
    );
  }
  if (isPathLike(value, label)) {
    return <ValueDisplay align="right" compactPath label={label} value={value} />;
  }

  return (
    <span className="flex min-w-0 max-w-full items-center justify-end gap-1 overflow-hidden">
      <span ref={valueRef} className="block min-w-0 flex-1 truncate" title={value}>
        {value}
      </span>
      {alwaysCopy || isTruncated || value.length > 24 ? (
        <button
          aria-label="Copy full value"
          className="grid size-5 shrink-0 place-items-center rounded border text-muted-foreground hover:bg-muted"
          title="Copy full value"
          type="button"
          onClick={() => void copyValue()}
        >
          <Clipboard size={12} />
        </button>
      ) : null}
    </span>
  );
}
