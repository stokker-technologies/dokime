# Tasks

## 1. Inventory and Baseline

- [x] 1.1 Lock the producer, consumer, route, state, test, build and dependency inventory below and record the presentation contracts in the proposal, design and requirement delta.
- [x] 1.2 Prepare the React worktree with the existing tasks. Capture connection, Cycles, Library, History, execution or cycle evidence with plots, Settings, Notifications and a saved report at matching viewports in Light and Dark under `target/svelte-parity/before/` with headless Playwright. Record reproducible fixture and route selection under the ignored evidence directory.
- [x] 1.3 Validate and commit the OpenSpec artifacts before porting source.

## 2. Plots

- [ ] 2.1 Port `CartesianPlot`, the placeholder, HistogramChart, and all catalog entries (Bar, Box, Contour, Frequency Series, Grid Chart, Heat Strip, Histogram, Pie, Time Series, Trajectory, XY and XY Interval). Preserve canvas and SVG geometry, viewport, legends, pointer and resizing behavior. Verify the existing presentation, heat strip and viewport suites.
- [ ] 2.2 Port PlotGroupGrid, AbbreviatedValue, FailureDetails, ErrorCard, StatsSummaryTable and the DataTable group. Preserve existing column sorting, remove generic search, filtering and pagination presentation, and verify semantic tables and current summary calculations.
- [ ] 2.3 Port the badge, button, checkbox, dialog, dropdown, input, popover, select, slider, table and tooltip primitives. Preserve focus, keyboard and pointer interaction and existing classes and slots.
- [ ] 2.4 Port interaction, playback, plot settings, plot palette, summary window and notifications presentation state. Preserve the axis, category, colors, expressions, grouping, frames, markers, bins, reduction, viewport, statistics, format and projection modules. Verify teardown and existing palette, point color and color suites.
- [ ] 2.5 Update plots exports, package manifest, tsconfig, Vite/Vitest and formatter configuration. Port component assertions to Testing Library. Run package typecheck and focused unit/integration tasks and capture matching plot states in both themes under `target/svelte-parity/after/plots/`.

## 3. Document

- [ ] 3.1 Port cycle documents, evidence, sidebar cases, labels, parameters, sections, records, steps, node summary, plot windows, prompt history and snapshot components. Preserve document layout, assembly, query models, evidence projection and report download behavior. Run the document presentation and cycle unit suites.
- [ ] 3.2 Port artifact tree, gallery, markdown, metric, path, records, value, notification and failure components, plus cards, progress, tabs, textarea, toggle and toggle group primitives. Verify media, record and notification interaction.
- [ ] 3.3 Port AppShellLayout, BrandMark, ConsoleDrawerView, PageShell, SidebarFrame and page chrome. Port appearance, application theme, brand, snapshot source, frozen plots, elapsed time, row windowing, media gallery, route scope, focus mode, shell density, sidebar/navigation state, console buffer, plot-window state and UI/selection stores. Keep pure models, CSS, theme tokens and host projections. Verify subscriptions and view restoration.
- [ ] 3.4 Update document exports, manifest, tsconfig, Vite/Vitest and formatter configuration. Run package typecheck and focused unit/integration tasks and capture matching document states in both themes under `target/svelte-parity/after/document/`.

## 4. Frozen Report

- [ ] 4.1 Port ReportApp, ReportCycleWorkspace, ReportShell and the entrypoint. Preserve appearance materialization, frozen evidence selection, and decoder and payload compatibility. Run the native-report integration suite.
- [ ] 4.2 Update package exports, manifest, tsconfig, Vite/Vitest, formatter configuration and `report.html`. Use `mise -C crates/dokime-gui run generate:report`, verify one standalone output and run the package typecheck. Open the same saved capture without external requests and capture both themes under `target/svelte-parity/after/report/`.

## 5. Desktop

- [ ] 5.1 Port startup, provider composition, native session and connection screen, query client, runtime stream and execution progress. Keep desktop commands, API, host and RPC clients, generated bindings and Tauri host contracts. Verify startup and shell status tests.
- [ ] 5.2 Port root shell and routes `/`, `/cycles`, `/cycles/:cycleId`, `/history`, `/history/:cycleId`, `/history/compare`, `/history/executions/:executionId`, `/library`, `/notifications`, `/settings`, `/settings/:section` and `/settings/plugins/:pluginId`. Preserve path and search semantics, redirects, focus and navigation history. Delete the generated React route tree and generator configuration.
- [ ] 5.3 Port app shell, menu, sidebar, console, status bar, run indicator, theme and appearance, backend status, console records, hub store, rerun notice and parameter overrides. Verify shell navigation, sidebar cycles, organization, run focus, console and notifications tests.
- [ ] 5.4 Port cycle and cycles-home, History/explore and execution evidence, Library, system Notifications and all Settings sections, plugins and installed plugins. Port queries and mutations for cycle detail, runtime, label, pending prompts, execution actions/files/records, notes, health, traceability, documents, History queries, settings frame, plugins, app preferences and appearance. Port routing and control hooks for active cycle, document cards/navigation, prompt values, drop, autosave, touch density and desktop build.
- [ ] 5.5 Port shared components and primitives, document and plot adapters, live frames and palette, suite picker state/mutations, parameter editing, artifact upload and media URLs. Verify settings, execution, storage, Library, cycle and History unit/integration suites, including document, evidence queries, saved report and control.
- [ ] 5.6 Port execution responsiveness and plot canvas/document fixtures and component tests to Testing Library. Preserve structural, performance-invariant and behavior assertions. Verify integration setup, generation/surface checks, smoke build, native e2e, report browser and save contracts. Port the React-specific system benchmark surface and build instrumentation without running the benchmark.
- [ ] 5.7 Update desktop `package.json`, `index.html`, `vite.config.ts`, `tsconfig.json`, `biome.json`, shadcn `components.json`, CSS scanning, and Tauri build references where needed. Verify the live frontend build and native asset paths.

## 6. Workspace and Handoff

- [ ] 6.1 Update package manifests and pnpm workspace lock. Remove React, React DOM and types, React Query/Router/Table, React router generator/plugin, React Vite plugin, Radix, lucide-react, next-themes, React notifications and unused React-bound state adapters. Preserve shared pure dependencies and remove stale React references in scoped tools.
- [ ] 6.2 Update shared Vite/TypeScript/test and format tooling, React-specific dev/build or organization generation if present, and applicable README wording. Identify any hosted CI and owner-managed documentation impact without changing either. Verify repository checks.
- [ ] 6.3 Incorporate `origin/dev` at each package boundary. Run focused typechecks and corresponding unit/integration suites for every affected frontend package, report generation, desktop frontend build, `mise run format`, `mise run check:repository`, `mise run test -- --changed Cargo.toml --no-run` with desktop testing as applicable, and strict OpenSpec validation.
- [ ] 6.4 Capture the final baseline journey in both themes under `target/svelte-parity/after/`, compare with the before captures and fix visible regressions. Verify the offline report and workspace dependency inventory.
