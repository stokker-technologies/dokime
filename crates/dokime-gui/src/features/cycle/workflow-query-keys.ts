// Dokime — SPDX-License-Identifier: Apache-2.0
import type { EvidenceScope } from "@generated/host/model";
import { NATIVE_PAGE_SIZE } from "../../lib/api/native-values";
import type { Wire } from "../../lib/rpc/canonical";

export const workflowQueryKeys = {
  all: ["workflow"] as const,
  artifacts: () => [...workflowQueryKeys.all, "artifacts"] as const,
  cycleDetails: () => [...workflowQueryKeys.all, "cycles", "detail"] as const,
  cycleDetail: (cycleId: string | null = null) => [...workflowQueryKeys.all, "cycles", "detail", cycleId ?? "current"] as const,
  cycleHistory: () => [...workflowQueryKeys.all, "cycles", "history"] as const,
  records: (limit = NATIVE_PAGE_SIZE) => [...workflowQueryKeys.all, "records", { limit }] as const,
  executionDocument: ({ cycleId, executionId }: { cycleId: string | null; executionId: string | null }) =>
    [...workflowQueryKeys.all, "documents", "execution", cycleId ?? "", executionId ?? ""] as const,
  executionDetails: () => [...workflowQueryKeys.all, "execution-detail"] as const,
  executionDetail: (executionId: string | null = null) => [...workflowQueryKeys.all, "execution-detail", executionId ?? ""] as const,
  executionFiles: (executionId: string | null) => [...workflowQueryKeys.all, "execution-files", executionId ?? ""] as const,
  health: () => [...workflowQueryKeys.all, "health"] as const,
  notes: () => [...workflowQueryKeys.all, "notes"] as const,
  prompts: () => [...workflowQueryKeys.all, "prompts"] as const,
  runtimeShell: () => [...workflowQueryKeys.all, "runtime-shell"] as const,
  traceability: (scope?: Wire<EvidenceScope> | null) =>
    scope ? ([...workflowQueryKeys.all, "traceability", scope] as const) : ([...workflowQueryKeys.all, "traceability"] as const),
};
