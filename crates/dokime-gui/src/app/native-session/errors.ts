// Dokime — SPDX-License-Identifier: Apache-2.0
/** Never stringify arbitrary request objects: they may contain credentials. */
export function connectionError(error: unknown): string {
  if (typeof error === "string") return error;
  if (error instanceof Error) return error.message;
  if (!error || typeof error !== "object" || !("kind" in error))
    return "The backend could not be reached.";
  if (error.kind === "connection" && "error" in error) return connectionError(error.error);
  if ("message" in error && typeof error.message === "string") return error.message;
  switch (error.kind) {
    case "local_backend_unavailable":
      return "This desktop session attaches to a backend another process started. Enter its endpoint below.";
    case "authentication_required":
    case "unauthenticated":
      return "The backend requires a valid access token.";
    case "credentials_unavailable":
      return "No saved access token is available. Enter a token to connect.";
    case "permission_denied":
      return "This account does not have permission to use the backend.";
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
      return "The backend could not be reached. Check the endpoint and credentials.";
  }
}
