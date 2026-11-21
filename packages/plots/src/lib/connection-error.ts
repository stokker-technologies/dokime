// Dokime — SPDX-License-Identifier: Apache-2.0

/** Never stringify arbitrary request objects: they may contain secrets. */
export function connectionError(error: unknown): string {
  if (typeof error === "string") {
    const kind = error.replace(/([a-z])([A-Z])/g, "$1_$2").toLowerCase();
    if (/^[a-z]+(?:_[a-z]+)+$/.test(kind)) return connectionError({ kind });
    return error;
  }
  if (error instanceof Error) return connectionError(error.message);
  if (!error || typeof error !== "object" || !("kind" in error)) return "The backend could not be reached.";
  if (error.kind === "connection" && "error" in error) return connectionError(error.error);
  if (error.kind === "archived" && "error" in error) return connectionError(error.error);
  if ("message" in error && typeof error.message === "string") return connectionError(error.message);
  switch (error.kind) {
    case "backend_unavailable":
      return "The backend is unavailable. Check that it is running and try again.";
    case "not_connected":
      return "Connect to a backend to continue.";
    case "generation_exhausted":
      return "Restart the desktop to establish a new connection.";
    case "local_backend_unavailable":
      return "This desktop session attaches to a backend another process started. Import its pairing file below.";
    case "authentication_required":
    case "unauthenticated":
      return "The backend requires a valid pairing file.";
    case "permission_denied":
      return "The backend refused this operation.";
    case "identity_changed":
      return "The backend identity does not match the expected identity.";
    case "incompatible_api":
      return "The backend API is incompatible with this desktop version.";
    case "wait_timed_out":
      return "The request wait deadline elapsed. Execution may still be running.";
    case "shutting_down":
      return "The desktop is shutting down.";
    case "transport":
      return "The endpoint could not be reached or its TLS certificate could not be verified.";
    case "superseded":
      return "The connection request was replaced.";
    default:
      return "The backend could not be reached. Check the endpoint and secrets.";
  }
}
