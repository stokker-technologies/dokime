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

## Native RPC Generation

- Public gRPC messages and explicit native RPC commands are generated from the
  canonical protocol descriptor by `mise run bindings:generate`; verify with
  `mise run bindings:check`. Do not maintain parallel domain DTOs or write
  invocation names in feature code. `src/lib/rpc/generated/client.ts` is the
  typed domain invocation surface. Keep this generation independent of
  compiling the GUI/runtime.
- The desktop control plane and its transport errors do not cross gRPC and are
  not generated. `src/lib/desktop/` holds the hand-written TypeScript for the
  commands declared in `src-tauri/src/bindings/mod.rs`; each module mirrors the
  Rust module that owns those types. Change both sides in one commit.
- `src/lib/types/` holds GUI-owned projection contracts, not wire types. The
  `src/lib/api/native-*.ts` adapters map generated protobuf messages into them;
  that mapping is the boundary, so do not replace a projection with a `_pb`
  type or let a protobuf shape leak into a component.
- Streams use one acknowledged pull at a time, with native frame, concurrency
  and idle limits. Never forward an unbounded gRPC reader into Tauri streams.
