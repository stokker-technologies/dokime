<!-- Rendered by stkr project update from .agents/skills/dokime-testing/references/desktop.md; sha256:7019341ba4f2a02a53362a18b4e283c56d6eb8a088eb9d4fe7eeca9eb0495038 -->

# Dokime Desktop Validation

Use the actual native WebView for desktop IPC and process ownership. Browser mocks and component tests do not prove native transport. Report precisely which mounted workflows and platforms were exercised; an unavailable platform remains unverified.

Global guidance owns cadence. Run narrow scenarios only for concrete blockers during editing; run required checks, the full test aggregate and explicitly required native system tasks after cohesive implementation. Discover tasks in `mise.toml`; `mise run test:system:typescript` exercises the native desktop when its prerequisites are available. Preserve repository environment and preparation for filtered diagnosis.

## Live Native Desktop

Run `mise run dev:desktop:dev`. The frontend dev server defaults to `http://127.0.0.1:5173`; set `DOKIME_DEV_PORT` for an isolated frontend. The owned gRPC listener binds a dynamic loopback port. There is no embedded desktop HTTP/WS server or `DOKIME_API_PORT` pair. The task builds the OBC and suite fixture bundles into `target/debug/plugins/` and sets `DOKIME_PLUGINS_DIR` there, so the owned backend loads them for that process without installing them.

Use a separate `DOKIME_APPLICATION_ROOT` for validation settings, plugins and data. For explicit attachment, forward native arguments: `mise run dev:desktop:dev -- -- --endpoint http://127.0.0.1:50051`. Remote endpoints require HTTPS and verified certificates; credentials belong in the native keyring, not URLs, arguments, logs, or browser storage.

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
