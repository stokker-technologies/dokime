# Port Desktop Presentation to Svelte

## Why

The desktop and frozen report share plots and document views. Port that presentation stack to Svelte so both products continue to use the same components, and remove the React toolchain from the frontend workspace.

## What Changes

- Port plots, document views, the frozen report and the desktop to Svelte 5 with TypeScript and Vite.
- Replace React Router, Query, Table and Radix integrations with a small desktop router, TanStack Svelte Query, native table markup and Bits UI. Preserve existing sort interactions.
- Keep the current desktop workflows, URLs, report payload, chart behavior, appearance, and accessibility.
- Port component tests, fixtures and frontend instrumentation. Capture the same views in Light and Dark from the baseline and each migration stage.

## Capabilities

### New Capabilities

- `svelte-presentation` covers the desktop, shared components, and standalone report framework and presentation parity.

### Modified Capabilities

Existing desktop, plot, and report capabilities retain their requirements.

## Impact

This changes `packages/plots`, `packages/document`, `packages/report`, `crates/dokime-gui`, their frontend tooling, and the pnpm workspace dependency graph. The checkout has no `packages/dokime` package. Host contracts, Rust domain crates, plugins, SDKs, documentation content, and hosted CI are outside the implementation boundary.
