import type { RuntimeAppLogo } from "@/lib/types/view-contracts";
/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Decode Embedded Native Report Responses
 */

/* === Imports === */

import type { CycleSnapshotSource } from "@/features/cycle/snapshot-source";
import { mapNativeHealth, mapNativeNote } from "@/lib/api/native-annotation-projection";
import { mapArtifactMetadata, mapExecutionFile } from "@/lib/api/native-content-projection";
import { assembleDocument, mapNativeDocument } from "@/lib/api/native-documents";
import { mapNativeExecution } from "@/lib/api/native-execution-projection";
import { mapNativeCyclePlan } from "@/lib/api/native-planning-projection";
import { mapNativePlotEnvelope } from "@/lib/api/native-plot-projection";
import { decodeFrozenRecords } from "@/lib/api/native-record-projection";
import { mapRuntimeShell } from "@/lib/api/native-runtime-projection";
import { mapNativeTraceability } from "@/lib/api/native-traceability-projection";
import { timestampIso } from "@/lib/api/native-values";
import { frozenRegistry } from "@/lib/host/schema-registry";
import { mapSuite } from "@/lib/host/suites";
import { appearanceFrameFromNative, logoVariant } from "@/lib/native-appearance";
import { cutKey, required, type Wire } from "@/lib/rpc/canonical";
import type { ArtifactRecord, DocumentViewPayload, PlotPayload } from "@/lib/types";
import { create, fromBinary, toBinary } from "@bufbuild/protobuf";
import { FileDescriptorSetSchema } from "@bufbuild/protobuf/wkt";
import { ReportCaptureSchema, type DocumentsGetResponse, type FileReference, type ReportCapture } from "@host/model";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Public Functions === */

export function readReportCapture(): ReportCapture {
  const data = document.getElementById("hub-report-data")?.textContent?.trim();
  if (!data) throw new Error("Open Report: Create a Report with Reports/Create First");
  const bytes = Uint8Array.from(atob(data), (character) => character.charCodeAt(0));
  return fromBinary(ReportCaptureSchema, bytes);
}

export async function materializeReport(capture: ReportCapture) {
  const urls = await fileUrls(capture);
  const source = await reportSource(capture, urls);
  const appearance = reportAppearance(capture, urls);
  return { source, appearance };
}

async function reportSource(capture: ReportCapture, urls: Map<string, string>): Promise<CycleSnapshotSource> {
  const metadata = required(capture.report, "Open Report Metadata");
  const cycle = required(capture.cycle?.cycle, "Open Report Cycle");
  const plan = mapNativeCyclePlan(
    cycle,
    capture.cases.map((entry) => {
      const detail = required(entry.detail?.value, "Open Report Case");
      return {
        detail,
        item: required(detail.summary, "Open Report Case Summary"),
        parameters: required(entry.parameters, "Open Report Case Parameters").parameters,
      };
    }),
    capture.suites.map((reply) => mapSuite(required(reply.value, "Open Report Suite"))),
  );
  const documents = new Map<string | null, DocumentViewPayload>();
  if (capture.document) documents.set(null, await reportDocument(capture.document));
  const plots = new Map<string, Record<string, PlotPayload>>();
  const traceability = new Map(
    capture.executions.map((entry) => {
      const detail = required(entry.detail?.execution, "Open Report Execution");
      const id = required(detail.summary, "Open Report Execution Summary").executionId;
      return [
        id,
        mapNativeTraceability(
          entry.traceability.flatMap((reply) => reply.snapshots),
          { cycleId: cycle.cycleId, executionId: id },
          "Open Report Traceability",
        ),
      ];
    }),
  );
  const executions = capture.executions.map((entry) =>
    mapNativeExecution(
      {
        detail: required(entry.detail?.execution, "Open Report Execution"),
        inputs: required(entry.inputs, "Open Report Inputs").parameters,
        outputs: required(entry.outputs, "Open Report Outputs").parameters,
        steps: required(entry.steps, "Open Report Steps").steps.map((step) => ({ step, criteria: step.criteria })),
      },
      plan.cases,
      cycle,
    ),
  );
  const records = [];
  for (const entry of capture.executions) {
    const detail = required(entry.detail?.execution, "Open Report Execution");
    const id = required(detail.summary, "Open Report Execution Summary").executionId;
    documents.set(id, await reportDocument(required(entry.document, "Open Report Document")));
    plots.set(
      id,
      Object.fromEntries(
        required(entry.plots, "Open Report Plots").envelopes.map((envelope) => {
          const mapped = mapNativePlotEnvelope(envelope);
          return [mapped.plot_id, mapped.payload];
        }),
      ),
    );
    const types = create(FileDescriptorSetSchema, {
      file: entry.schemas.flatMap((reply) => required(reply.types, "Open Report Schemas").file),
    });
    const digest = new Uint8Array(await crypto.subtle.digest("SHA-256", toBinary(FileDescriptorSetSchema, types)));
    if (cutKey(digest) !== cutKey(required(detail.schemas, "Open Report Schema Digest").digest))
      throw new Error("Open Report: Restore Its Digest-Matched Frozen Schemas");
    const registry = frozenRegistry(types);
    records.push(
      ...decodeFrozenRecords(
        capture.records.flatMap((reply) => reply.records).filter((record) => record.scope?.executionId === id),
        registry,
        detail.schemas?.records,
      ),
    );
  }
  const artifacts = capture.artifacts.map((reply) =>
    attachFile(mapArtifactMetadata(required(reply.metadata, "Open Report Artifact")), reply.file, urls),
  );
  const files = new Map<string, ArtifactRecord[]>();
  for (const entry of capture.executions) {
    const id = required(entry.detail?.execution?.summary, "Open Report File Execution").executionId;
    files.set(
      id,
      entry.files.map((reply) => attachFile(mapExecutionFile(id, required(reply.metadata, "Open Report File")), reply.file, urls)),
    );
  }
  const cycleTraceability = mapNativeTraceability(
    capture.executions.flatMap((entry) => entry.traceability.flatMap((reply) => reply.snapshots)),
    { cycleId: cycle.cycleId },
    "Open Report Traceability",
  );
  const notes = capture.notes.flatMap((reply) => reply.notes).map(mapNativeNote);
  const health = capture.health.flatMap((reply) => reply.events).map(mapNativeHealth);
  const overview = required(capture.overview?.overview, "Open Report Overview");
  const runtimeShell = mapRuntimeShell([{ ...overview, cycleSummaries: [cycle] }], cycle.cycleId);
  return {
    generatedAt: timestampIso(metadata.generatedAt),
    plan,
    executions,
    runtimeShell: { ...runtimeShell, records, notes, health, artifacts },
    records,
    notes,
    health,
    artifacts,
    document: (id) => documents.get(id) ?? null,
    executionFiles: (id) => (id === null ? [] : (files.get(id) ?? [])),
    plotPayloads: (id) => ({ plots: id === null ? {} : (plots.get(id) ?? {}) }),
    traceability: (id) => (id === null ? cycleTraceability : (traceability.get(id) ?? null)),
    warnings: metadata.warnings,
  };
}

function reportAppearance(capture: ReportCapture, urls: Map<string, string>) {
  const appearance = required(capture.appearance?.appearance, "Open Report Appearance");
  const logos: RuntimeAppLogo[] = appearance.logos.map((logo) => {
    const asset = required(logo.asset, "Open Report Logo");
    const reference = required(asset.reference, "Open Report Logo Identity");
    return {
      variant: logoVariant(logo),
      url: fileUrl(required(asset.file, "Open Report Logo File"), urls, asset.contentType),
      plugin_id: reference.pluginId,
      asset_id: reference.assetId,
    };
  });
  return appearanceFrameFromNative({ appearance, logos, dispose: () => undefined });
}

/* === Private Functions === */

async function reportDocument(reply: DocumentsGetResponse): Promise<DocumentViewPayload> {
  const first = reply.frames[0]?.frame?.kind;
  if (first?.case !== "header") throw new Error("Open Report: Restore Its Document Header");
  const header = first.value;
  const scope = required(header.scope, "Open Report Document Scope");
  const frames = (async function* () {
    yield* reply.frames;
  })();
  const document = await assembleDocument(
    frames,
    {
      scope: { cycleId: required(scope.cycleId, "Open Report Cycle Scope"), executionId: scope.executionId ?? "" },
      documentId: header.documentId,
      epoch: required(header.frontiers, "Open Report Document Frontiers").processEpoch,
    },
    () => true,
  );
  return mapNativeDocument(document);
}

function fileKey(file: Wire<FileReference>): string {
  return `${file.length}/${[...file.sha256].map((byte) => byte.toString(16).padStart(2, "0")).join("")}`;
}

async function fileUrls(capture: ReportCapture): Promise<Map<string, string>> {
  const urls = new Map<string, string>();
  for (const entry of capture.files) {
    const file = required(entry.file, "Open Report File Reference");
    const bytes = new Uint8Array(entry.reads.reduce((length, reply) => length + reply.data.length, 0));
    let offset = 0;
    for (const reply of entry.reads) {
      bytes.set(reply.data, offset);
      offset += reply.data.length;
    }
    const digest = new Uint8Array(await crypto.subtle.digest("SHA-256", bytes));
    if (BigInt(bytes.length) !== file.length || cutKey(digest) !== cutKey(file.sha256))
      throw new Error("Open Report: Restore Its Digest-Matched File Bytes");
    let binary = "";
    for (let offset = 0; offset < bytes.length; offset += 32768) binary += String.fromCharCode(...bytes.subarray(offset, offset + 32768));
    urls.set(fileKey(file), `data:${file.metadata?.mediaType || "application/octet-stream"};base64,${btoa(binary)}`);
  }
  return urls;
}

function attachFile(artifact: ArtifactRecord, file: Wire<FileReference> | undefined, urls: Map<string, string>): ArtifactRecord {
  const url = fileUrl(required(file, "Open Report Artifact File"), urls, artifact.media_type);
  return { ...artifact, href: url, downloadHref: url };
}

function fileUrl(file: Wire<FileReference>, urls: Map<string, string>, mediaType: string | null | undefined): string {
  const url = required(urls.get(fileKey(file)), "Open Report File Bytes");
  return mediaType ? url.replace(/^data:[^;]*;/, `data:${mediaType};`) : url;
}

/* === Entry Point === */
