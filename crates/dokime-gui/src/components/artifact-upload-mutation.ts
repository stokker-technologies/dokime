/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { useMutation, useQueryClient } from "@tanstack/react-query";

/* === Local Imports === */

import { useNativeSession } from "@/app/native-session/context";
import { invalidateWorkflowDocuments, invalidateWorkflowResourceQueries } from "@/features/cycle/server-state";
import { nativeContentSession, uploadNativeArtifact } from "@/lib/api/native-content";
import type { NativeArtifactUploadOptions } from "../lib/api/native-content-projection";

/* === Hooks === */

export function useOperatorArtifactUploadMutation() {
  const session = useNativeSession();
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (options: NativeArtifactUploadOptions) =>
      uploadNativeArtifact(nativeContentSession(session.identity, session.own), options),
    onSuccess: async () => {
      await Promise.all([invalidateWorkflowResourceQueries(queryClient), invalidateWorkflowDocuments(queryClient)]);
    },
  });
}
