# AGENTS.md

Agent context for the Dokime GUI. This is a Tauri 2 desktop application with a
Vite, React 19, and Tailwind CSS v4 frontend. The same frontend code supports the
live operator shell and the self-contained report renderer.

## Brand Assets

- Brand values live in `../../branding/brand.toml` and are compiled into
  `dokime-theme`. Preserve committed frontend assets under
  `../../branding/generated/`; runtime appearance owns the compiled stylesheet.
- Treat `../dokime-app/dist-report/report.html` as generated output. The report
  build owns it.

## Project Shape

- `src/app` and `src/routes` compose the live application shell and route state.
- `src/features` owns product workflows; keep workflow behavior with its feature.
- `src/components/ui` contains reusable primitives. Shared product components
  belong under `src/components` only when their contract is genuinely reusable.
- `src/lib` owns typed cross-feature infrastructure such as appearance, API, and
  theme handling. Keep transport and browser side effects out of primitives.
- `src/report` owns static-report boot and appearance behavior. Changes to shared
  theme or document rendering must account for both live and report entrypoints.

## Validation

- Install synchronized dependencies with `mise run sync`.
- After authored brand changes, rebuild the compiled theme and report with
  `mise run prepare:report`, then run the repository checks.
- Run `mise run check:types` and `mise run test:integration:javascript` for frontend changes.
- Run `mise run check` before committing. Use `mise run test:system:javascript` when a
  running-stack workflow or browser interaction changes.

## Canonical Bindings And Transport

- `mise run bindings:generate` projects standard ES bindings and the typed facade
  from one explicitly selected canonical Cargo export. `bindings:check` compares
  that same export. Rust `dokime-contract::operations` declarations own the wire
  shape; do not maintain parallel domain DTOs or a second codec. Generation does
  not compile the GUI or runtime.
- Calls and subscriptions in `src/lib/rpc/generated/client.ts` go through the
  seam in `src/lib/rpc/transport.ts`. The desktop binds it to `ipc.ts`, the opaque
  bridge in `src-tauri/src/rpc/calls.rs`: path and protobuf bytes in, bytes or a
  typed status out, one Tauri channel per stream, fenced by the connection
  generation. The bridge never decodes or dispatches per operation; keep it so.
  `grpc-web.ts` serves only a browser build (`DOKIME_TRANSPORT=grpc-web`). The
  WebView holds no endpoint or credential; they never belong in status, logs,
  profiles, persisted browser storage or report output.
- `src-tauri/src/rpc/artifact_upload.rs` is the native file-backed adapter for
  artifact and recording bytes. Do not add ordinary calls to it.
- Desktop controls and transport errors are hand-written in `src/lib/desktop/`,
  matching their native owner modules. Change both sides together.
- `src/lib/types/` holds GUI projection contracts. API adapters map generated
  message shapes into them; preserve that boundary in components.
- Streams read one bounded frame at a time, require final OK status, and cancel
  with their session or subscription owner. Canceling a subscription does not stop
  the backend producer. WebView TLS requires platform-trusted certificates.
