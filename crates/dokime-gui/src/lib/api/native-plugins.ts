/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Native plugin and admitted-artifact adapters for the original settings UI. */

/* === Local Imports === */

import { create } from "@bufbuild/protobuf";
import type { RuntimeInstalledPluginRecord } from "@/features/settings/installed-plugins/installed-plugin-contracts";
import * as rpc from "@/lib/rpc/generated/client";
import {
  type AdmittedPlugin,
  type PluginInstallation,
  PluginInstallationState,
  StagePluginFrameSchema,
} from "@/lib/rpc/generated/dokime/v1/plugins_pb";
import type { SessionIdentity } from "@/lib/rpc/transport";
import type { PluginEntry } from "@/lib/types/view-contracts";
import type { NativeContentSession } from "./native-content";
import {
  NATIVE_MAX_PAGES,
  NATIVE_PAGE_SIZE,
  NativeApiLimitError,
  safeNumber,
  sha256Hex,
  timestampIso,
} from "./native-values";

/* === Plugin installations === */

export async function listNativePlugins(session: SessionIdentity): Promise<PluginEntry[]> {
  const installations: PluginInstallation[] = [];
  let after: { manifestSha256: string; afterInstallId: string } | undefined;

  for (let page = 0; page < NATIVE_MAX_PAGES; page += 1) {
    const response = await rpc.pluginsListInstallations(session, {
      pageSize: NATIVE_PAGE_SIZE,
      ...(after ? { after } : {}),
    });
    installations.push(...response.installations);
    if (!response.next) return installations.map(mapPluginInstallation);

    const next = {
      manifestSha256: response.next.manifestSha256,
      afterInstallId: response.next.afterInstallId,
    };
    if (
      after &&
      after.manifestSha256 === next.manifestSha256 &&
      after.afterInstallId === next.afterInstallId
    ) {
      throw new NativeApiLimitError("Plugins.ListInstallations", "cursor did not advance");
    }
    after = next;
  }

  throw new NativeApiLimitError("Plugins.ListInstallations", `more than ${NATIVE_MAX_PAGES} pages`);
}

export async function getNativePlugin(
  session: SessionIdentity,
  installId: string,
): Promise<PluginEntry> {
  return mapPluginInstallation(await rpc.pluginsGetInstallation(session, { installId }));
}

export async function consentNativePlugin(
  session: SessionIdentity,
  installId: string,
): Promise<PluginEntry> {
  const current = await rpc.pluginsGetInstallation(session, { installId });
  const installation = await rpc.pluginsConsent(session, {
    installId,
    expectedSha256: current.sha256,
  });
  return mapPluginInstallation(installation);
}

export async function removeNativePlugin(
  session: SessionIdentity,
  installId: string,
): Promise<void> {
  await rpc.pluginsRemove(session, { installId });
}

export async function setNativePluginEnabled(
  session: SessionIdentity,
  installId: string,
  enabled: boolean,
): Promise<PluginEntry> {
  return mapPluginInstallation(await rpc.pluginsSetEnabled(session, { installId, enabled }));
}

/* === Plugin staging progress === */

/** Progress phases {@link stageNativePlugin} reports through `onProgress`, in order. */
export type StagePluginProgress =
  | { phase: "reading" }
  | { phase: "hashing" }
  | { phase: "opening" }
  | { phase: "uploading"; sent: number; total: number }
  | { phase: "finishing" };

export interface StagePluginOptions {
  onProgress?(progress: StagePluginProgress): void;
  /** Test-only override for the invoke stall bounds below; production never sets this. */
  stallBoundsMs?: Partial<StageStallBoundsMs>;
}

interface StageStallBoundsMs {
  open: number;
  write: number;
  finish: number;
}

/**
 * Matches the native side's own bounds: the bridge's 30 s idle timeout on
 * each `client_stream_write`/open frame (`client_streams.rs::IDLE_TIMEOUT`),
 * and the server's 300 s deadline on the whole `Plugins.Stage` call
 * (`grpc/deadline.rs::Deadlines::upload`). Nothing on the client enforces
 * either today, so an invoke that never resolves — the failure this module
 * exists to diagnose — spins "Inspecting …" forever.
 */
const DEFAULT_STAGE_STALL_MS: StageStallBoundsMs = {
  open: 30_000,
  write: 30_000,
  finish: 300_000,
};

/**
 * Bound on the cancel invoke itself. `cancel` is what every stall path above
 * falls back on; without its own bound, a stalled `client_stream_cancel`
 * would let the cleanup this module runs on the way out re-create the exact
 * hang it exists to end.
 */
const STAGE_CANCEL_STALL_MS = 10_000;

/** Render a byte count the way staging progress and its failure messages do: decimal megabytes. */
export function formatStagePluginBytes(bytes: number): string {
  const megabytes = Math.round((bytes / 1_000_000) * 10) / 10;
  return Number.isInteger(megabytes) ? `${megabytes} MB` : `${megabytes.toFixed(1)} MB`;
}

/** Stage an inert plugin archive; activation remains a separate consent command. */
export async function stageNativePlugin(
  session: NativeContentSession,
  file: File,
  options: StagePluginOptions = {},
): Promise<PluginEntry> {
  const onProgress = options.onProgress;
  const bounds = { ...DEFAULT_STAGE_STALL_MS, ...options.stallBoundsMs };

  onProgress?.({ phase: "reading" });
  const bytes = await withPhase("reading", () => readPluginBytes(file));

  onProgress?.({ phase: "hashing" });
  const digest = await withPhase("hashing", () => sha256Hex(bytes));

  let stream: Awaited<ReturnType<typeof rpc.pluginsStage>> | undefined;
  let release: (() => void) | undefined;
  let cancelPromise: Promise<void> | undefined;
  const cancel = async (): Promise<void> => {
    if (!stream) return;
    cancelPromise ??= withTimeout(stream.cancel(), STAGE_CANCEL_STALL_MS);
    await cancelPromise;
  };
  try {
    onProgress?.({ phase: "opening" });
    // The open request is the first frame the bridge forwards to the
    // server, which requires that first frame to be the header. Writing
    // the header as a separate first `stream.write()` call instead would
    // forward an empty frame first, which the server rejects.
    stream = await withInvoke("opening", bounds.open, cancel, () =>
      rpc.pluginsStage(session, {
        frame: { case: "header", value: { sizeBytes: BigInt(bytes.byteLength), sha256: digest } },
      }),
    );
    release = session.own(() => void cancel().catch(() => undefined));

    const total = bytes.byteLength;
    const chunkCount = Math.ceil(total / NATIVE_PLUGIN_CHUNK_BYTES);
    // 0-based: chunk N starts at byte offset N * NATIVE_PLUGIN_CHUNK_BYTES,
    // so a chunk's number and its "sent" byte offset always agree, in both
    // the progress event just before it and the failure message if it
    // stalls or errors.
    let chunkIndex = 0;
    for (let offset = 0; offset < total; offset += NATIVE_PLUGIN_CHUNK_BYTES) {
      const data = bytes.slice(offset, offset + NATIVE_PLUGIN_CHUNK_BYTES);
      onProgress?.({ phase: "uploading", sent: offset, total });
      const label = `uploading chunk ${chunkIndex} of ${chunkCount} (${formatStagePluginBytes(offset)} of ${formatStagePluginBytes(total)})`;
      const openedStream = stream;
      await withInvoke(label, bounds.write, cancel, () =>
        openedStream.write(
          create(StagePluginFrameSchema, {
            frame: { case: "chunk", value: { offset: BigInt(offset), data } },
          }),
        ),
      );
      chunkIndex += 1;
    }
    onProgress?.({ phase: "uploading", sent: total, total });

    onProgress?.({ phase: "finishing" });
    const openedStream = stream;
    return mapPluginInstallation(
      await withInvoke("finishing", bounds.finish, cancel, () => openedStream.finish()),
    );
  } finally {
    await cancel().catch(() => undefined);
    release?.();
  }
}

const NATIVE_PLUGIN_BYTES = 272 * 1024 * 1024;
const NATIVE_PLUGIN_CHUNK_BYTES = 256 * 1024;

async function readPluginBytes(file: File): Promise<Uint8Array<ArrayBuffer>> {
  if (!Number.isSafeInteger(file.size) || file.size <= 0 || file.size > NATIVE_PLUGIN_BYTES) {
    throw new NativeApiLimitError(
      "Plugins.Stage",
      `plugin archive must be between 1 and ${NATIVE_PLUGIN_BYTES} bytes`,
    );
  }
  return new Uint8Array(await file.arrayBuffer());
}

/** Marks a {@link withTimeout} rejection as the timeout winning, not `promise` itself failing. */
class StallTimeout extends Error {}

/**
 * Race `promise` against a timer, without ever letting the loser become an
 * unhandled rejection: if `promise` loses (the timeout fires first) and
 * later rejects anyway — once native cleanup unblocks whatever it was
 * waiting on — nothing else in this module is still awaiting it directly.
 */
function withTimeout<T>(promise: Promise<T>, ms: number): Promise<T> {
  promise.catch(() => undefined);
  let timer!: ReturnType<typeof setTimeout>;
  const timeout = new Promise<never>((_resolve, reject) => {
    timer = setTimeout(() => reject(new StallTimeout()), ms);
  });
  return Promise.race([promise, timeout]).finally(() => clearTimeout(timer));
}

/** Wrap a local (non-RPC) staging step so its failure names the phase it happened in. */
async function withPhase<T>(label: string, run: () => Promise<T>): Promise<T> {
  try {
    return await run();
  } catch (error) {
    throw stagingFailure(label, error);
  }
}

/**
 * Wrap a single bridge invoke (open, one write, or finish) with the stall
 * watchdog: past `timeoutMs` with no response, cancel the stream and fail
 * with a phase-tagged "stalled after N s" message instead of leaving the
 * caller's "Inspecting …" spinning forever.
 */
async function withInvoke<T>(
  label: string,
  timeoutMs: number,
  cancel: () => Promise<void>,
  run: () => Promise<T>,
): Promise<T> {
  try {
    return await withTimeout(run(), timeoutMs);
  } catch (error) {
    await cancel().catch(() => undefined);
    if (error instanceof StallTimeout) {
      throw new Error(`plugin staging failed while ${label} stalled after ${timeoutMs / 1_000} s`);
    }
    throw stagingFailure(label, error);
  }
}

/** Keep the size-limit precondition's own message untouched; wrap everything else with its phase. */
function stagingFailure(label: string, error: unknown): Error {
  if (error instanceof NativeApiLimitError) return error;
  const message = error instanceof Error ? error.message : String(error);
  const failure = new Error(`plugin staging failed while ${label}: ${message}`, { cause: error });
  failure.name = "NativeStagePluginError";
  return failure;
}

/* === Admitted Plugins === */

export async function listNativeAdmittedPlugins(
  session: SessionIdentity,
): Promise<RuntimeInstalledPluginRecord[]> {
  const artifacts: AdmittedPlugin[] = [];
  let after: { epoch: string; revision: bigint; afterPluginId: string } | undefined;

  for (let page = 0; page < NATIVE_MAX_PAGES; page += 1) {
    const response = await rpc.pluginsListAdmittedPlugins(session, {
      pageSize: NATIVE_PAGE_SIZE,
      ...(after ? { after } : {}),
    });
    artifacts.push(...response.plugins);
    if (!response.next) return artifacts.map(mapAdmittedPlugin);

    const next = {
      epoch: response.next.epoch,
      revision: response.next.revision,
      afterPluginId: response.next.afterPluginId,
    };
    if (
      after &&
      after.epoch === next.epoch &&
      after.revision === next.revision &&
      after.afterPluginId === next.afterPluginId
    ) {
      throw new NativeApiLimitError("Plugins.ListAdmittedPlugins", "cursor did not advance");
    }
    after = next;
  }

  throw new NativeApiLimitError(
    "Plugins.ListAdmittedPlugins",
    `more than ${NATIVE_MAX_PAGES} pages`,
  );
}

/* === Mapping === */

function mapPluginInstallation(installation: PluginInstallation): PluginEntry {
  const stagedAt = timestampIso(installation.stagedAt);
  if (!stagedAt) {
    throw new Error(`Native plugin ${installation.installId} has no valid staged timestamp.`);
  }

  return {
    install_id: installation.installId,
    sha256: installation.sha256,
    staged_at: stagedAt,
    consented_at: timestampIso(installation.consentedAt),
    state: mapPluginState(installation.state),
    last_boot: mapPluginLastBoot(installation.lastBoot),
    metadata: installation.metadata
      ? {
          identity: installation.metadata.identity
            ? {
                name: installation.metadata.identity.name,
                version: installation.metadata.identity.version,
                description: installation.metadata.identity.description,
              }
            : (() => {
                throw new Error(
                  `Native plugin ${installation.installId} metadata has no identity.`,
                );
              })(),
          owners: installation.metadata.owners,
          size_bytes: safeNumber(installation.metadata.sizeBytes, "Plugins plugin size"),
          environment_references: installation.metadata.environmentReferences.map((reference) => ({
            owner: reference.owner,
            setting: reference.setting,
            variable: reference.variable,
          })),
        }
      : null,
    error: installation.error ?? null,
    active: installation.active,
    restart_required: installation.restartRequired,
    boot_error: installation.bootError ?? null,
  };
}

function mapPluginState(state: PluginInstallationState): PluginEntry["state"] {
  switch (state) {
    case PluginInstallationState.STAGED:
      return "staged";
    case PluginInstallationState.INSTALLED:
      return "installed";
    case PluginInstallationState.DISABLED:
      return "disabled";
    case PluginInstallationState.REMOVAL_PENDING:
      return "removal_pending";
    default:
      throw new Error("Native plugin returned an unknown installation state.");
  }
}

function mapPluginLastBoot(lastBoot: PluginInstallation["lastBoot"]): PluginEntry["last_boot"] {
  if (!lastBoot) return null;
  const at = timestampIso(lastBoot.at);
  if (!at) throw new Error("Native plugin returned a last-boot result without a timestamp.");
  switch (lastBoot.result.case) {
    case "loaded":
      return { state: "loaded", at };
    case "error":
      return { state: "failed", error: lastBoot.result.value, at };
    default:
      throw new Error("Native plugin returned a last-boot result without an outcome.");
  }
}

function mapAdmittedPlugin(artifact: AdmittedPlugin): RuntimeInstalledPluginRecord {
  return {
    plugin_id: artifact.pluginId,
    owner:
      artifact.owner.case === "host"
        ? { kind: "host" }
        : artifact.owner.case === "bundle"
          ? {
              kind: "bundle",
              package: artifact.owner.value.package,
              installation: artifact.owner.value.installationId ?? null,
              content_digest: artifact.owner.value.contentDigest,
            }
          : artifact.owner.case === "embedded"
            ? { kind: "embedded", generation: artifact.owner.value.generation }
            : null,
    label: artifact.label,
    version: artifact.version,
    enabled: artifact.enabled,
    // The original view model only has a code-defined runtime kind. Native
    // admitted artifacts are the code catalog exposed by Plugins, so preserve
    // that existing badge contract while retaining the owner discriminator.
    runtime: "code",
    status: artifact.enabled ? "ready" : "disabled",
    error: null,
  };
}
