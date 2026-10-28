/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Backend identity and connection state from the existing runtime subscription. */

import { useQuery } from "@tanstack/react-query";

import { useNativeSession } from "@/app/native-session/context";
import * as rpc from "@/lib/rpc/generated/client";
import { useHubStore } from "@/stores/hub-store";

export type ConnectionState = "connected" | "reconnecting" | "disconnected" | "unknown";

export type BackendStatus = {
  /** Operator-facing backend name, or null while the identity read is pending. */
  label: string | null;
  connection: ConnectionState;
  /** Connection failure reported by the runtime subscription. */
  detail: string | null;
};

export function useBackendStatus(): BackendStatus {
  const session = useNativeSession();
  const identity = useQuery({
    queryFn: () => rpc.backendGetIdentity(session.identity, {}),
    queryKey: ["shell", "backend-identity", session.identity.generation],
    staleTime: Number.POSITIVE_INFINITY,
  });
  const connection = useHubStore((state) => state.connection);
  const error = useHubStore((state) => state.error);

  return {
    connection: connection === "connecting" ? "unknown" : connection,
    detail: error,
    label: identity.data?.displayName || identity.data?.backendId || null,
  };
}
