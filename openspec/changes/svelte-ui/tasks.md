# Tasks

## 1. React Baseline and Layout

- [x] 1.1 Capture the React desktop journeys and frozen report in Light and Dark under `target/svelte-parity/before/`.
- [x] 1.2 Separate Cartesian frame selection, range calculation and viewport reduction and verify palette and desktop plot integration.
- [x] 1.3 Move shared plots, document presentation, cycle views, projections, contracts, primitives, formatting and theme into the existing frontend by domain. Preserve the in-progress Svelte plots for the next step.
- [x] 1.4 Put the report HTML and entry in the GUI Vite build, preserving the report generation task, output path and decoder. Move organization generation and Tauri watch inputs.
- [x] 1.5 Remove package manifests, aliases and workspace entries. Consolidate package test jobs into the existing GUI lanes and identify any hosted workflow impact.
- [x] 1.6 Prove the React layout with the GUI typecheck, both Vite builds, the report import check, and focused frontend and tooling tests. Commit the layout before Svelte integration.

## 2. Plots

- [ ] 2.1 Port Cartesian, placeholder, histogram, and all catalog plots. Preserve viewport, canvas and SVG geometry, indicators, legends, hover, and resizing.
- [ ] 2.2 Port plot interactions, playback, settings, palette and summary window state. Keep plot math, projections, colors, axes, frames and formatting.
- [ ] 2.3 Port plot grouping, summaries, failure cards and shared controls. Preserve keyboard and pointer behavior and replace generic data table search and pagination with plain tables and existing column sorting.
- [ ] 2.4 Port the corresponding test fixtures and run focused tests and typecheck. Capture plots in both themes under `target/svelte-parity/after/plots/`.

## 3. Document and Section Presentation

- [ ] 3.1 Port document workspace and view, sections, notes, records, overview metrics and plan cards. Preserve models, query semantics, evidence assembly and report download.
- [ ] 3.2 Port artifact tree, media, markdown, value, clipboard and traceability presentation, notification and failure views, and the remaining shared UI controls.
- [ ] 3.3 Port the document tests, including the plot canvas and live document responsiveness fixtures. Preserve measurements consumed by the system driver. Run focused tests and typecheck and capture both themes under `target/svelte-parity/after/document/`.

## 4. Cycle Views

- [ ] 4.1 Port shared cycle evidence, cases, labels, parameters, sections, records, steps, prompt history, plot windows and snapshot source.
- [ ] 4.2 Port desktop cycle workspace and home, queries, mutations, active-cycle and document navigation, prompt and execution control, and live plot wiring.
- [ ] 4.3 Run focused cycle, execution and document tests and typecheck and capture both themes under `target/svelte-parity/after/cycle/`.

## 5. Frozen Report

- [ ] 5.1 Port the report app, navigation within the capture, shell and entry. Preserve appearance materialization, captured execution selection and the embedded decoder contract.
- [ ] 5.2 Extend the report import check to reject Tauri, routing, query clients and desktop-only modules in its transitive module graph.
- [ ] 5.3 Run the native report tests, typecheck and report generation. Verify one self-contained output, open the capture offline and capture both themes under `target/svelte-parity/after/report/`.

## 6. Desktop Shell and Routes

- [ ] 6.1 Port startup, native session, providers, runtime subscriptions, query client and connection screen. Keep desktop commands, host and RPC clients, generated bindings and Tauri contracts.
- [ ] 6.2 Replace the router and generated tree while preserving the root, cycles, history, comparison and execution evidence, library, notifications and settings URLs, parameters, redirects and back and forward.
- [ ] 6.3 Port the desktop shell, menu, sidebar, console, indicators, theme, focus, native appearance, UI and selection state, and runtime notifications.
- [ ] 6.4 Port History, Library, Notifications and Settings surfaces, including preferences, plugins, suite and parameter editing, file uploads and media. Run focused tests and typecheck and capture the desktop journeys in both themes.
- [ ] 6.5 Port React-specific system fixture instrumentation and complete the live frontend build.

## 7. Cleanup and Handoff

- [ ] 7.1 Remove superseded React components, hooks, fixtures, route generation and dependencies. Update Vite, TypeScript, Biome, CSS scanning, shadcn configuration and the pnpm lockfile.
- [ ] 7.2 Incorporate upstream changes at each migration boundary. Run focused frontend checks, both builds, format, repository check, the workspace test-target compile gate and strict OpenSpec validation.
- [ ] 7.3 Compare the final screenshots under `target/svelte-parity/after/` against the baseline and fix visible regressions. Verify offline report and final workspace dependencies.
