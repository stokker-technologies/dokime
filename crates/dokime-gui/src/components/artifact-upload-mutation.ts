/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

/* === Local Imports === */

import type { ConnectedSession } from "@/app/native-session/session";
import { invalidateWorkflowDocuments, invalidateWorkflowResourceQueries } from "@/features/cycle/server-state";
import { nativeContentSession, uploadNativeArtifact } from "@/lib/api/native-content";
import type { NativeArtifactUploadOptions } from "@/lib/api/native-content-projection";

/* === Hooks === */

export function operatorArtifactUploadOptions(session: ConnectedSession) {
  const { queryClient } = session;
  return {
    mutationFn: (options: NativeArtifactUploadOptions) =>
      uploadNativeArtifact(nativeContentSession(session.identity, session.own), options),
    onSuccess: async () => {
      await Promise.all([invalidateWorkflowResourceQueries(queryClient), invalidateWorkflowDocuments(queryClient)]);
    },
  };
}
