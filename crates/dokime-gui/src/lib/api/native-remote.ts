// Dokime — SPDX-License-Identifier: Apache-2.0

// Access-Owned Listener Projection for Desktop Settings.
import type { ApplicationSettings, PreferencesSnapshot } from "@generated/application-settings";
import * as rpc from "@/lib/host/client";
import type { Wire } from "@/lib/rpc/canonical";
import type { SessionIdentity } from "@/lib/rpc/transport";

export type ListenerSettings = {
  address: string;
  enabled: boolean;
  certificateNames: string[];
  tlsEnabled: boolean;
  authenticationEnabled: boolean;
};
export type ListenerStatus = {
  settings: ListenerSettings;
  revision?: Uint8Array;
  listeningAddress?: string;
  certificatePem?: string;
  error?: string;
  keyConfigured?: boolean;
  endpoints: { address: string; adapterName?: string }[];
};
export type ScopedPreferencesSnapshot = PreferencesSnapshot & { listener?: ListenerStatus };

export async function readNativeListener(session: SessionIdentity): Promise<ListenerStatus> {
  return projectListener(await rpc.accessListenerGet(session, {}));
}
export async function updateNativeListener(
  session: SessionIdentity,
  status: ListenerStatus,
  settings: ListenerSettings,
): Promise<ListenerStatus> {
  if (!status.revision?.byteLength) throw new Error("Save Remote Access: Refresh Its Listener Revision Before Saving");
  if (!settings.tlsEnabled || !settings.authenticationEnabled)
    throw new Error("Save Remote Access: This Host Requires TLS and Pairing Authentication");
  return projectListener(
    await rpc.accessListenerSet(session, {
      expectedRevision: status.revision,
      value: { enabled: settings.enabled, endpoint: settings.address, certificateNames: settings.certificateNames },
    }),
  );
}
export async function regenerateNativeCertificate(session: SessionIdentity): Promise<ListenerStatus> {
  const certificate = await rpc.accessCertificatesCreate(session, {});
  return { ...(await readNativeListener(session)), certificatePem: certificate.certificatePem };
}

export async function withNativeListener(session: SessionIdentity, snapshot: PreferencesSnapshot): Promise<ScopedPreferencesSnapshot> {
  const listener = await readNativeListener(session);
  return { ...snapshot, listener, settings: { ...snapshot.settings, listener: listenerPreferences(listener.settings) } };
}
export async function saveNativeListenerPreferences(
  session: SessionIdentity,
  previous: ScopedPreferencesSnapshot,
  settings: ApplicationSettings,
): Promise<ScopedPreferencesSnapshot> {
  if (!previous.listener) throw new Error("Save Remote Access: Refresh the Listener Before Saving");
  const value = settings.listener;
  const listener = await updateNativeListener(session, previous.listener, {
    enabled: value.enabled,
    address: value.address,
    certificateNames: value.certificate_names,
    tlsEnabled: value.tls_enabled,
    authenticationEnabled: value.authentication_enabled,
  });
  return { ...previous, listener, settings: { ...previous.settings, listener: listenerPreferences(listener.settings) } };
}
function listenerPreferences(value: ListenerSettings): ApplicationSettings["listener"] {
  return {
    address: value.address,
    enabled: value.enabled,
    certificate_names: value.certificateNames,
    tls_enabled: value.tlsEnabled,
    authentication_enabled: value.authenticationEnabled,
  };
}
function projectListener(reply: Wire<Awaited<ReturnType<typeof rpc.accessListenerGet>>>): ListenerStatus {
  const value = reply.value;
  if (!value || !reply.revision.byteLength) throw new Error("Read Remote Access: Host Omitted Its Applied Listener or Revision");
  const address = reply.boundEndpoint || undefined;
  return {
    settings: {
      address: value.endpoint,
      enabled: value.enabled,
      certificateNames: value.certificateNames,
      tlsEnabled: true,
      authenticationEnabled: true,
    },
    revision: reply.revision,
    listeningAddress: address,
    endpoints: address ? [{ address }] : [],
  };
}
