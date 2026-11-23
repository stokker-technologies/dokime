// Dokime — SPDX-License-Identifier: Apache-2.0

/* === Imports === */

import { createQuery, type QueryClient } from "@tanstack/svelte-query";
import type { ConnectedSession } from "../../../app/native-session/session";
import { putSecret, readSecrets, removeSecret, type SecretMetadata, type SecretSnapshot } from "../../../lib/api/native-secrets";

/* === Public Functions === */

export function createSecretStorage(session: () => ConnectedSession | null, client: () => QueryClient) {
  const queryKey = $derived(["settings", "secrets", session()?.identity.generation]);
  const query = createQuery(() => {
    const identity = session()?.identity;
    return {
      queryKey,
      enabled: Boolean(identity),
      queryFn: () => {
        if (!identity) {
          throw new Error("Connect to manage Secret Storage.");
        }
        return readSecrets(identity);
      },
    };
  }, client);
  let busy = $state(false);
  let error = $state.raw<unknown>();

  async function change(work: () => Promise<SecretSnapshot>, onSuccess: () => void) {
    const identity = session()?.identity;
    if (!identity) {
      return;
    }
    const key = queryKey;
    const active = client();
    busy = true;
    error = undefined;
    try {
      await active.cancelQueries({ queryKey: key });
      const next = await work();
      if (identity.isCurrent()) {
        active.setQueryData(key, next);
        void active.invalidateQueries({ queryKey: ["settings", "installed-plugins"] });
        onSuccess();
      }
    } catch (failure) {
      if (identity.isCurrent()) {
        error = failure;
      }
    } finally {
      if (identity.isCurrent()) {
        busy = false;
      }
    }
  }

  return {
    query,
    get busy() {
      return busy;
    },
    get error() {
      return error;
    },
    save(metadata: SecretMetadata, value: string | undefined, generation: bigint, onSuccess: () => void) {
      const identity = session()?.identity;
      return identity ? change(() => putSecret(identity, metadata, value, generation), onSuccess) : Promise.resolve();
    },
    remove(identifier: string, generation: bigint, onSuccess: () => void) {
      const identity = session()?.identity;
      return identity ? change(() => removeSecret(identity, identifier, generation), onSuccess) : Promise.resolve();
    },
  };
}
