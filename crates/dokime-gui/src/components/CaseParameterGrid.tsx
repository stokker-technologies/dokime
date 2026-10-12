/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Local Imports === */

import { ValueDisplay } from "@/components/ValueDisplay";
import { testCaseParameterEntries } from "@/lib/test-cases";
import type { JsonObject, TestCase } from "@/lib/types";

/* === Types === */

type CaseParameterGridProps = {
  testCase?: TestCase;
  environment?: JsonObject | null;
};

/* === Components === */

export function CaseParameterGrid({ testCase, environment }: CaseParameterGridProps) {
  const parameters = testCaseParameterEntries(testCase);
  const environmentEntries = environmentDisplayEntries(environment);
  if (!parameters.length && !environmentEntries.length) {
    return null;
  }
  return (
    <div className="grid gap-4">
      <ParameterSection entries={parameters} title="Case Parameters" />
      <ParameterSection entries={environmentEntries} title="Environment" />
    </div>
  );
}

function ParameterSection({
  entries,
  title,
}: {
  entries: Array<[string, unknown]>;
  title: string;
}) {
  if (!entries.length) {
    return null;
  }
  return (
    <section className="grid gap-2">
      <h4 className="text-xs font-bold uppercase text-muted-foreground">{title}</h4>
      <dl className="grid min-w-0 gap-x-5 gap-y-3 text-xs sm:grid-cols-2 2xl:grid-cols-3">
        {entries.map(([key, value]) => (
          <div className="grid min-w-0 gap-1.5" key={key}>
            <dt
              className="truncate text-[0.68rem] font-semibold uppercase leading-4 text-muted-foreground"
              title={key}
            >
              {key}
            </dt>
            <dd className="min-w-0 font-mono text-xs leading-5 text-foreground">
              <ValueDisplay compactPath label={key} value={value} />
            </dd>
          </div>
        ))}
      </dl>
    </section>
  );
}

function objectEntries(value: Record<string, unknown> | null | undefined) {
  if (!value) {
    return [];
  }
  return Object.entries(value).filter(([, item]) => item !== null && item !== "");
}

function environmentDisplayEntries(
  environment: Record<string, unknown> | null | undefined,
): Array<[string, unknown]> {
  if (!environment) {
    return [];
  }
  const location = asRecord(environment.location);
  const reading = asRecord(environment.reading);
  return objectEntries({
    Location: location?.label,
    Latitude: location?.latitude,
    Longitude: location?.longitude,
    Source: location?.source,
    Temperature: reading?.temperature_c,
    Humidity: reading?.relative_humidity_percent,
    Pressure: reading?.pressure_hpa,
    Conditions: reading?.condition,
  });
}

function asRecord(value: unknown): Record<string, unknown> | null {
  return value && typeof value === "object" && !Array.isArray(value)
    ? (value as Record<string, unknown>)
    : null;
}
