/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

import { Database } from "lucide-react";
import { DocumentCard } from "@/components/document/DocumentCard";
import { formatValue } from "@/lib/dashboard";
import type { DocumentCardLayout, TelemetryStream } from "@/lib/types";
import {
  latestStreamSample,
  telemetryContextGroups,
  telemetryFieldLabel,
  usefulTelemetryEntries,
} from "../document-overview-model";
import { LongValue } from "./LongValue";

export function TelemetryContextCard({
  streams,
  layout,
}: {
  streams: TelemetryStream[];
  layout?: DocumentCardLayout;
}) {
  const groups = telemetryContextGroups(streams);
  return (
    <DocumentCard
      contentClassName="p-0"
      eyebrow="Streams"
      icon={<Database size={20} />}
      layout={layout}
      title="Telemetry Context"
    >
      {groups.length ? (
        <div className="grid gap-0">
          {groups.map((group) => (
            <section className="border-b px-4 py-3 last:border-b-0" key={group.title}>
              <div className="mb-3 flex items-start justify-between gap-3">
                <div>
                  <strong>{group.title}</strong>
                  <p className="text-sm text-muted-foreground">{group.description}</p>
                </div>
                <span className="rounded-md border bg-muted/25 px-2 py-1 text-xs font-semibold">
                  {group.streams.reduce((total, stream) => total + stream.total_samples, 0)} samples
                </span>
              </div>
              <div className="grid gap-3">
                {group.streams.map((stream) => (
                  <TelemetryStreamSummary stream={stream} key={stream.stream} />
                ))}
              </div>
            </section>
          ))}
        </div>
      ) : (
        <p className="p-4 text-sm text-muted-foreground">No telemetry has been published yet.</p>
      )}
    </DocumentCard>
  );
}

export function TelemetryStreamSummary({ stream }: { stream: TelemetryStream }) {
  const latest = latestStreamSample(stream);
  const entries = usefulTelemetryEntries(stream, latest);
  const status = String(latest.probe_status ?? latest.scan_status ?? "").trim();

  return (
    <article className="grid gap-2 rounded-md border bg-muted/15 p-3">
      <div className="grid gap-1 md:grid-cols-[minmax(0,1fr)_auto] md:items-start">
        <div className="min-w-0">
          <strong className="block truncate text-sm">{stream.spec?.label ?? stream.stream}</strong>
          <span className="text-xs text-muted-foreground">{stream.stream}</span>
        </div>
        <div className="flex flex-wrap gap-2 text-xs text-muted-foreground md:justify-end">
          {status ? (
            <span className="rounded border bg-background px-1.5 py-0.5">{status}</span>
          ) : null}
          <span>{stream.total_samples} samples</span>
          {stream.dropped_samples ? <span>{stream.dropped_samples} dropped</span> : null}
        </div>
      </div>
      {entries.length ? (
        <dl className="grid gap-2 sm:grid-cols-2 xl:grid-cols-4">
          {entries.map(([key, value]) => (
            <div className="min-w-0 rounded border bg-background p-2" key={key}>
              <dt className="truncate text-xs font-semibold text-muted-foreground">
                {telemetryFieldLabel(stream, key)}
              </dt>
              <dd
                className="mt-1 truncate text-sm font-medium"
                title={formatValue(value, undefined, key)}
              >
                {formatValue(value, undefined, key)}
              </dd>
            </div>
          ))}
        </dl>
      ) : (
        <p className="text-sm text-muted-foreground">No useful retained fields for this stream.</p>
      )}
      <details className="text-xs text-muted-foreground">
        <summary className="cursor-pointer">Advanced Raw</summary>
        <LatestSampleGrid stream={stream} />
      </details>
    </article>
  );
}

export function LatestSampleGrid({ stream }: { stream: TelemetryStream }) {
  const latest = latestStreamSample(stream);
  const entries = Object.entries(latest)
    .filter(([, value]) => value !== null && value !== undefined)
    .slice(0, 8);
  const columnLabels = new Map((stream.spec?.columns ?? []).map((column) => [column.name, column]));

  if (!entries.length) {
    return (
      <p className="text-sm text-muted-foreground md:col-span-2">
        No retained samples for this stream.
      </p>
    );
  }

  return (
    <dl className="grid gap-2 md:col-span-2 sm:grid-cols-2 xl:grid-cols-4">
      {entries.map(([key, value]) => {
        const column = columnLabels.get(key);
        return (
          <div className="min-w-0 rounded border bg-muted/20 p-2" key={key}>
            <dt className="truncate text-xs font-semibold text-muted-foreground" title={key}>
              {column?.label ?? key}
            </dt>
            <dd className="mt-1 min-w-0 text-sm font-medium">
              <LongValue
                label={key}
                value={formatValue(value, column?.display_unit ?? column?.unit, key)}
              />
            </dd>
          </div>
        );
      })}
    </dl>
  );
}
