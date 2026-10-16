/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* One written report, as a payload the report tests can mount.
 *
 * The shape is taken from a report the backend actually wrote, trimmed to the
 * smallest evidence that still exercises every surface the frozen file puts on
 * screen: the cycle and its queue, one finished attempt, a console record and
 * — the part a hand-written fixture kept leaving out — an artifact link. A
 * report with no links renders the explorer's empty state and never mounts the
 * file browser or the text viewer behind it, which is how a required-session
 * hook reached the frozen path unnoticed.
 */

/* === Local Imports === */

import type { DocumentNode } from "@/lib/document-contracts";
import type { DocumentCapturePayload } from "@/lib/document-normalizer";
import type { ReportPayload } from "@/lib/report-types";
import type {
  ArtifactRecord,
  HubRecord,
  StepExecutionRecord,
  TestCyclePlan,
  TestExecution,
} from "@/lib/types";

/* === Constants === */

export const FIXTURE_CYCLE_ID = "cycle-1";

export const FIXTURE_CASE_ID = "case-1";

export const FIXTURE_CASE_LABEL = "Protection At 230 V";

export const FIXTURE_CYCLE_TITLE = "Mains Protection Sweep";

export const FIXTURE_EXECUTION_ID = "execution-1";

export const FIXTURE_RECORD_MESSAGE = "Case provider work settled";

/** A previewable text artifact, bytes and all, exactly as a report carries it. */
export const FIXTURE_ARTIFACT_NAME = "startup-summary.json";

/* === Public Functions === */

export function reportFixturePayload(): ReportPayload {
  return {
    case_runs: [
      {
        case_id: FIXTURE_CASE_ID,
        documents: { default: reportDocument(FIXTURE_EXECUTION_ID) },
        plots: {},
        queue_item_id: "queue-1",
        test_execution_id: FIXTURE_EXECUTION_ID,
      },
    ],
    cycle_summary: null,
    documents: { default: reportDocument(null) },
    generated_at: "2026-09-30T12:34:00.000Z",
    plots: {},
    schema_version: 1,
    snapshot: {
      artifacts: [artifact()],
      health: [],
      notes: [],
      pending_prompts: [],
      records: [record()],
      revision: 0,
      run: {
        metadata: {},
        session_id: "session-1",
        status: "completed",
        started_at: null,
        template_slug: "protection",
        title: FIXTURE_CYCLE_TITLE,
        updated_at: "2026-09-30T10:00:00.000Z",
      },
      session_id: "session-1",
      telemetry: {},
      title: FIXTURE_CYCLE_TITLE,
      updated_at: "2026-09-30T10:00:00.000Z",
    },
    test_cycle_plan: plan(),
    test_executions: [execution()],
    warnings: [],
  };
}

/* === Private Functions === */

function plan(): TestCyclePlan {
  return {
    cases: [
      {
        case_id: FIXTURE_CASE_ID,
        suite_id: "suite-1",
        generation_index: 0,
        label: FIXTURE_CASE_LABEL,
        metadata: {},
        parameters: {},
        template_id: "template-1",
      },
    ],
    generated_at: "2026-09-30T10:00:00.000Z",
    queue: [
      {
        attempt_count: 1,
        case_id: FIXTURE_CASE_ID,
        ordinal: 0,
        queue_item_id: "queue-1",
        repeat_index: 0,
        status: "passed",
        template_id: "template-1",
        test_cycle_id: FIXTURE_CYCLE_ID,
      },
    ],
    templates: [],
    test_cycle: {
      case_count: 1,
      continue_on_failure: false,
      generated_at: "2026-09-30T10:00:00.000Z",
      metadata: {},
      queue_count: 1,
      status: "completed",
      test_cycle_id: FIXTURE_CYCLE_ID,
      title: FIXTURE_CYCLE_TITLE,
    },
  };
}

function execution(): TestExecution {
  return {
    attempt_index: 0,
    case_id: FIXTURE_CASE_ID,
    metadata: {},
    queue_item_id: "queue-1",
    status: "passed",
    steps: [step("step"), step("criterion")],
    template_id: "template-1",
    test_cycle_id: FIXTURE_CYCLE_ID,
    test_execution_id: FIXTURE_EXECUTION_ID,
  };
}

/* Step and criterion rows arrive under the execution that recorded them. */
function step(recordType: "step" | "criterion"): StepExecutionRecord {
  return {
    case_id: FIXTURE_CASE_ID,
    duration_s: recordType === "step" ? 1 : 0,
    finished_at: "2026-09-30T10:00:01.000Z",
    metadata: {},
    method_name: recordType === "step" ? "protection.trip" : "protection.trip_time",
    order: 0,
    queue_item_id: "queue-1",
    record_type: recordType,
    result: { details: {}, status: "passed" },
    started_at: "2026-09-30T10:00:00.000Z",
    status: "passed",
    step_id: "step-1",
    template_id: "template-1",
    test_cycle_id: FIXTURE_CYCLE_ID,
    test_execution_id: FIXTURE_EXECUTION_ID,
  };
}

function record(): HubRecord {
  return {
    details: {},
    display: { icon: "activity", label: "info", tone: "neutral" },
    level: "info",
    message: FIXTURE_RECORD_MESSAGE,
    record_id: "record-1",
    record_type: "log",
    scope: scope(),
    source: "runtime",
    timestamp: "2026-09-30T10:00:00.000Z",
  } as HubRecord;
}

function artifact(): ArtifactRecord {
  const href = "data:application/json;base64,eyJhY2NlcHRlZCI6IHRydWV9";
  return {
    artifact_id: "artifact-1",
    description: "Execution artifact",
    group_id: "execution-artifacts",
    group_label: "Execution Artifacts",
    /* The generator inlines the bytes, so the row is reachable from the file
       itself and needs no backend to open. */
    downloadHref: href,
    href,
    label: FIXTURE_ARTIFACT_NAME,
    media_type: "application/json",
    path: FIXTURE_ARTIFACT_NAME,
    scope: scope(),
    size_bytes: 19,
  } as ArtifactRecord;
}

function scope() {
  return {
    test_case_id: FIXTURE_CASE_ID,
    template_id: "template-1",
    test_cycle_id: FIXTURE_CYCLE_ID,
    test_execution_id: FIXTURE_EXECUTION_ID,
  };
}

/* The smallest document that still puts the artifact explorer on screen: the
   records timeline the console reads, and the artifacts card that mounts the
   file browser. Both sections are copied from a written report's layout. */
function reportDocument(executionId: string | null): DocumentCapturePayload {
  const records = {
    metadata: documentMetadata("cycle.events.timeline", "Records"),
    type: "records",
  } as const;
  const artifacts = {
    data: {
      descriptor: {
        artifact_count: 1,
        completed_count: 1,
        scope: documentScope(executionId),
      },
    },
    metadata: documentMetadata("cycle.traceability.artifacts", "Artifacts"),
    type: "artifacts",
  } as const;

  return {
    definition: {
      cards: [records, { metadata: artifacts.metadata, type: "artifacts" }],
      document_id: "default",
      layout: {
        nodes: [
          documentSectionNode("cycle.events", "Records", "cycle.events.timeline"),
          documentSectionNode(
            "cycle.traceability.artifacts",
            "Artifacts",
            "cycle.traceability.artifacts",
          ),
        ],
      },
      metadata: documentMetadata("default", FIXTURE_CYCLE_TITLE),
    },
    selection: { scope: documentScope(executionId) },
    cards: [records, artifacts].map((card) => ({
      source_id: card.metadata.identifier,
      metadata: card.metadata,
      result: { type: "static" },
    })),
    repeats: {},
    sections: {
      "cycle.events": { metadata: documentMetadata("cycle.events", "Records") },
      "cycle.traceability.artifacts": {
        metadata: documentMetadata("cycle.traceability.artifacts", "Artifacts"),
      },
    },
    warnings: [],
  } satisfies DocumentCapturePayload;
}

function documentSectionNode(identifier: string, label: string, cardId: string): DocumentNode {
  return {
    data: {
      children: [
        {
          data: {
            card_id: cardId,
            layout: {
              col_span: { base: 12, lg: null, md: null, sm: null, xl: null },
              density: "normal",
              order: 10,
              rail: "main",
              row_span: 1,
            },
          },
          type: "card",
        },
      ],
      metadata: documentMetadata(identifier, label),
      node_id: identifier,
    },
    type: "section",
  };
}

function documentMetadata(identifier: string, label: string) {
  return {
    description: null,
    identifier,
    label,
    tags: [],
  };
}

/** The document's own scope shape, which names the case as `test_case_id`. */
function documentScope(executionId: string | null) {
  return {
    criterion_id: null,
    session_id: null,
    step_id: null,
    suite_id: null,
    template_id: null,
    test_case_id: executionId ? FIXTURE_CASE_ID : null,
    test_cycle_id: FIXTURE_CYCLE_ID,
    test_execution_id: executionId,
  };
}
