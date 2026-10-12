/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The console's live tail.
 *
 * One subscription, opened when the drawer is open and closed the moment it is
 * not: a console nobody is reading must not hold a stream. The scope is the
 * route's — the cycle, and the attempt when the operator picked one — so the
 * rows always belong to what is on screen.
 */

/* === External Imports === */

import { useEffect, useState } from "react";

/* === Local Imports === */

import { useNativeSessionOptional } from "@/app/native-session/NativeSessionProvider";
import { openNativeRecordStream } from "@/lib/api/native-evidence";
import {
  type InstanceLogLevel,
  instanceLogRecord,
  instanceLogsSupported,
  openInstanceLogStream,
} from "@/lib/api/native-logs";
import type { HubRecord } from "@/lib/types";

/* === Constants === */

/** The tail the drawer keeps in memory; older rows leave the window. */
const RETAINED_RECORDS = 5_000;

const NO_RECORDS: HubRecord[] = [];

/* === Hooks === */

export function useConsoleRecords({
  cycleId,
  enabled,
  executionId,
}: {
  cycleId: string | null;
  enabled: boolean;
  executionId: string | null;
}): HubRecord[] {
  const session = useNativeSessionOptional();
  const [records, setRecords] = useState<HubRecord[]>(NO_RECORDS);

  useEffect(() => {
    setRecords(NO_RECORDS);
    if (!enabled || !session || !cycleId) return undefined;

    let active = true;
    let retry: number | undefined;
    let stream: Awaited<ReturnType<typeof openNativeRecordStream>> | undefined;

    const connect = async () => {
      if (!active || !session.identity.isCurrent()) return;
      try {
        stream = await openNativeRecordStream(
          { ...session.identity, own: session.own },
          { test_cycle_id: cycleId, test_execution_id: executionId },
        );
        for await (const page of stream) {
          if (!active || !session.identity.isCurrent()) break;
          if (page.length === 0) continue;
          setRecords((current) => [...current, ...page].slice(-RETAINED_RECORDS));
        }
      } catch {
        /* A dropped subscription is not the operator's problem: reconnect. */
      } finally {
        stream = undefined;
        if (active && session.identity.isCurrent()) {
          retry = window.setTimeout(() => void connect(), 1_000);
        }
      }
    };

    void connect();

    return () => {
      active = false;
      if (retry !== undefined) window.clearTimeout(retry);
      void stream?.cancel().catch(() => undefined);
    };
  }, [cycleId, enabled, executionId, session]);

  return records;
}

/**
 * The backend's own log, as console rows.
 *
 * The level is part of the subscription, not a filter over it, so changing it
 * re-opens the stream; `dropped` is carried out rather than swallowed so the
 * drawer can say how much of the tail it never saw.
 */
export function useInstanceLogs({
  enabled,
  level,
}: {
  enabled: boolean;
  level: InstanceLogLevel;
}): { dropped: number; records: HubRecord[]; supported: boolean } {
  const session = useNativeSessionOptional();
  const [records, setRecords] = useState<HubRecord[]>(NO_RECORDS);
  const [dropped, setDropped] = useState(0);
  const supported = instanceLogsSupported();

  useEffect(() => {
    if (!enabled || !supported || !session) {
      setRecords(NO_RECORDS);
      setDropped(0);
      return undefined;
    }
    let cancelled = false;
    let sequence = 0;
    let close: (() => Promise<void>) | null = null;
    setRecords(NO_RECORDS);
    setDropped(0);
    void (async () => {
      try {
        const stream = await openInstanceLogStream(session.identity, { minLevel: level });
        close = stream.cancel;
        if (cancelled) {
          await stream.cancel().catch(() => undefined);
          return;
        }
        for await (const frame of stream) {
          if (cancelled) return;
          // `dropped` is cumulative, so it is the count and not an increment.
          setDropped(frame.dropped);
          const record = instanceLogRecord(frame, sequence++);
          setRecords((current) => [...current, record].slice(-RETAINED_RECORDS));
        }
      } catch {
        // An unavailable capability is reported by `supported`, not by a throw
        // the drawer would have to render as an error over an empty list.
      }
    })();
    return () => {
      cancelled = true;
      void close?.().catch(() => undefined);
    };
  }, [enabled, level, session, supported]);

  return { dropped, records, supported };
}
