# Dokime Desktop Validation

Use the actual native WebView for desktop IPC and process ownership. Browser mocks and component tests do not prove native transport. Report precisely which mounted workflows and platforms were exercised; an unavailable platform remains unverified.

Global guidance owns cadence. During implementation run relevant local gates and tests proving changed behavior, respecting explicit holds. After source completion select acceptance and native system lanes from affected behavior. Local full validation is reserved for committed changes with project-wide impact; before ready review, complete the required final-candidate local gates and satisfy the complete provider validation graph, including configured native platforms. Discover tasks in `mise.toml`; `mise run test:system` runs the desktop walkthrough (`test:system:typescript`) after the server walkthrough when its prerequisites are available. Preserve repository environment and preparation for filtered diagnosis.

## Live Native Desktop

Run `mise run dev`. The frontend dev server defaults to `http://127.0.0.1:5173`; set `DOKIME_DEV_PORT` for an isolated frontend. The owned gRPC listener binds a dynamic loopback port. There is no embedded desktop HTTP/WS server or `DOKIME_API_PORT` pair. The task builds the OBC bundle into `target/debug/plugins/` and sets `DOKIME_PLUGINS_DIR` there, so the owned backend loads it for that process without installing it.

Use a separate `DOKIME_APPLICATION_ROOT` for validation settings, plugins and data. For explicit attachment, start the shell with its `--endpoint http://127.0.0.1:50051` argument. HTTPS verifies the server certificate. Explicit HTTP selects plaintext native gRPC for an isolated lab and exposes keys and data; authentication remains the receiving listener's independent policy. Credentials belong in the native keyring, not URLs, arguments, logs, or browser storage. A remote target never acquires native listener trust from its scheme or loopback spelling.

Inspect the real workspace: suite and template selection, typed parameters, plan/create/run, pending prompts, lifecycle completion, and visible history/evidence supported by mounted views. Test failed endpoint/authentication, cancel, disconnect, reconnect, and backend switching. Old generations must not update the new cache, streams or appearance.

Closing a local-owned desktop must drain client, listener, then runtime. Closing an attached desktop must leave the remote host alive. An unavailable secondary operation must not secretly invoke HTTP.

## Visual And Diagnostic Pass

- Exercise changed controls and adjacent mounted workflow states: loading, empty, failed, permission denied, running, completed, and disconnected.
- Check desktop and narrow widths, keyboard labels/focus, text overflow and stable controls during updates.
- Inspect WebView errors and network resources; the WebView makes no network requests. Domain operations cross the opaque IPC bridge (`rpc`, and `rpc_stream_*` with one Tauri channel per stream); native IPC also owns session lifecycle and the file-backed upload and recording adapter. Do not use `/api` fetches, WebSockets or gRPC-Web from the desktop.
- Measure stream transport changes rather than assuming them: `node crates/dokime-gui/e2e/stream-throughput.mjs` compares plot stream rows per second, frame latency and unary round trips through the bridge against direct gRPC-Web in the same WebView.
- Capture screenshots when useful, keep temporary evidence outside committed source, and stop only the validation processes you own.
- Do not repeatedly create runs to diagnose a stale view; inspect its authoritative native state and generation first.
- Keep performance assertions scoped to measured phases and actual optimized/debug profiles.
