/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import { Badge } from "@/components/ui/badge";
import { useBrand } from "@/lib/brand";
import type { TestExecution } from "@/lib/types";

/** An invalidated attempt keeps its evidence but cannot produce a result. */
export function ExecutionInvalidationNotice({ execution }: { execution: TestExecution | null }) {
  const brand = useBrand();
  if (execution?.status !== "invalidated") return null;
  const reason = execution.invalidation;
  return (
    <div
      role="status"
      className="grid gap-2 rounded-md border border-warning/30 bg-warning/10 p-3 text-sm"
    >
      <div>
        <Badge variant="warning">Invalidated</Badge>
      </div>
      <p>
        {reason
          ? reason.current
            ? `Plugin ${reason.previous.source} changed after this execution was created.`
            : `Plugin ${reason.previous.source} was not available when ${brand.label} restarted. It may have been removed, disabled, or failed to load.`
          : "The plugin required by this execution changed or is no longer available."}
      </p>
      <p>This execution ended without a result. Its recorded evidence is preserved.</p>
      <p className="text-muted-foreground">
        Run this case again in a new execution when the required plugin is available.
      </p>
    </div>
  );
}
