/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { z } from "zod";

/* === Constants === */

const WORKFLOW_ROUTES = ["plan", "execution", "cycle"] as const;

const workflowRoutePaths = {
  cycle: "/cycle",
  execution: "/execution",
  plan: "/plan",
} as const satisfies Record<WorkflowRoute, `/${string}`>;

export const workflowRouteSchema = z.enum(WORKFLOW_ROUTES);

const optionalSearchStringSchema = z
  .preprocess((value) => {
    const candidate = Array.isArray(value) ? value.at(-1) : value;
    if (typeof candidate !== "string") {
      return undefined;
    }
    const trimmed = candidate.trim();
    return trimmed.length > 0 ? trimmed : undefined;
  }, z.string().optional())
  .catch(undefined);

const followSearchSchema = z
  .preprocess((value) => {
    const candidate = Array.isArray(value) ? value.at(-1) : value;
    if (candidate === "0" || candidate === 0 || candidate === false) {
      return "0";
    }
    if (candidate === "1" || candidate === 1 || candidate === true) {
      return "1";
    }
    return undefined;
  }, z.union([z.literal("0"), z.literal("1")]).optional())
  .catch(undefined);

export const workflowSearchSchema = z.object({
  cycleId: optionalSearchStringSchema,
  executionId: optionalSearchStringSchema,
  follow: followSearchSchema,
  queueItemId: optionalSearchStringSchema,
});

/* === Types === */

export type WorkflowRoute = z.infer<typeof workflowRouteSchema>;

export type WorkflowRoutePath = (typeof workflowRoutePaths)[WorkflowRoute];

export type WorkflowSearch = z.infer<typeof workflowSearchSchema>;

export type SelectionRouteState = {
  selectedExecutionId: string | null;
  followActive: boolean;
  selectedQueueItemId: string | null;
};

/* === Public Functions === */

export function workflowRoutePath(view: WorkflowRoute): WorkflowRoutePath {
  return workflowRoutePaths[view];
}

export function parseWorkflowRoute(pathname: string): WorkflowRoute {
  const candidate = pathname.split("/").filter(Boolean)[0] ?? "plan";
  const result = workflowRouteSchema.safeParse(candidate);
  return result.success ? result.data : "plan";
}

export function parseWorkflowSearch(value: unknown): WorkflowSearch {
  return workflowSearchSchema.parse(searchObject(value));
}

export function parseSelectionRouteState(value: unknown): SelectionRouteState {
  const search = parseWorkflowSearch(value);
  return {
    selectedExecutionId: search.executionId ?? null,
    followActive: search.follow !== "0",
    selectedQueueItemId: search.queueItemId ?? null,
  };
}

export function serializeSelectionRouteState(state: SelectionRouteState): WorkflowSearch {
  return {
    executionId: state.selectedExecutionId ?? undefined,
    follow: state.followActive ? undefined : "0",
    queueItemId: state.selectedQueueItemId ?? undefined,
  };
}

export function workflowSearchKey(value: WorkflowSearch): string {
  return workflowSearchParams(value).toString();
}

export function workflowSearchParams(value: WorkflowSearch): URLSearchParams {
  const searchParams = new URLSearchParams();
  if (value.cycleId) {
    searchParams.set("cycleId", value.cycleId);
  }
  if (value.follow) {
    searchParams.set("follow", value.follow);
  }
  if (value.executionId) {
    searchParams.set("executionId", value.executionId);
  }
  if (value.queueItemId) {
    searchParams.set("queueItemId", value.queueItemId);
  }
  return searchParams;
}

/* === Private Functions === */

function searchObject(value: unknown): Record<string, unknown> {
  if (value instanceof URLSearchParams) {
    return {
      cycleId: value.get("cycleId") ?? undefined,
      executionId: value.get("executionId") ?? undefined,
      follow: value.get("follow") ?? undefined,
      queueItemId: value.get("queueItemId") ?? undefined,
    };
  }
  return value && typeof value === "object" && !Array.isArray(value) ? (value as Record<string, unknown>) : {};
}
