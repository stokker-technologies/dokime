/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Backend identity and readiness for the status bar. Backend.GetIdentity and
   Backend.GetHealth have no `lib/api/native-*.ts` adapter yet, so the shell
   calls the generated client directly and keeps the mapping here. */

/* === External Imports === */

import { useQuery } from "@tanstack/react-query";

/* === Local Imports === */

import { useNativeSession } from "@/app/native-session/NativeSessionProvider";
import * as rpc from "@/lib/rpc/generated/client";
import { Readiness } from "@/lib/rpc/generated/dokime/v1/backend_pb";

/* === Types === */

export type ConnectionState = "connected" | "reconnecting" | "disconnected" | "unknown";

export type BackendStatus = {
  /** Operator-facing backend name, or null while the identity read is pending. */
  label: string | null;
  connection: ConnectionState;
  /** Readiness detail worth showing beside the state, when the backend sends one. */
  detail: string | null;
};

/* === Constants === */

const HEALTH_POLL_MS = 15_000;

/* === Hooks === */

export function useBackendStatus(): BackendStatus {
  const session = useNativeSession();
  const identity = useQuery({
    queryFn: () => rpc.backendGetIdentity(session.identity, {}),
    queryKey: ["shell", "backend-identity", session.identity.generation],
    staleTime: Number.POSITIVE_INFINITY,
  });
  const health = useQuery({
    queryFn: () => rpc.backendGetHealth(session.identity, {}),
    queryKey: ["shell", "backend-health", session.identity.generation],
    refetchInterval: HEALTH_POLL_MS,
  });

  return {
    connection: connectionState(health.data?.readiness, health.isError),
    detail: health.data?.message?.trim() || null,
    label: identity.data?.displayName || identity.data?.backendId || null,
  };
}

/* === Private Functions === */

function connectionState(readiness: Readiness | undefined, failed: boolean): ConnectionState {
  if (failed) {
    return "disconnected";
  }
  switch (readiness) {
    case Readiness.READY:
      return "connected";
    case Readiness.STARTING:
    case Readiness.DEGRADED:
      return "reconnecting";
    case Readiness.STOPPING:
      return "disconnected";
    default:
      return "unknown";
  }
}
