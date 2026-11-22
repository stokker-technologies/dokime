# Design

## Context

Vite builds the desktop assets and the standalone report. The desktop connects to its native host and the report reads a captured payload. Both compose the same document and plot packages. The migration changes their rendering and state bindings at that boundary.

## Decisions

1. Use Svelte 5 runes, TypeScript, and the Vite Svelte plugin. Components own local state, derived views, and DOM lifecycle. Use typed snippets for caller-provided markup and context for shared presentation state. Keep RPC clients, generated protocol bindings, models, projections, plot math, uPlot and d3, formatting, and theme CSS and tokens at their current ownership boundaries.
2. Use TanStack Svelte Query for server state. Preserve query identities, cache operations, mutation results, stream subscriptions, and teardown semantics. Use Bits UI for the accessible headless interactions and Lucide's maintained `@lucide/svelte` package for icons. The registry deprecates the original `lucide-svelte` name. Use semantic HTML tables with local sorting or TanStack Table Core where existing sorting warrants it. Remove the generic table search and pagination controls under the owner rule. Keep domain search in History and server-side evidence paging.
3. Use a small History API router in the desktop. The existing desktop serves a single Vite entry and the routes need client-side segment matching, typed parameters and search state, redirects, link navigation, and back and forward. SvelteKit would add a second application build and layout convention to supply those operations. Keep the existing URLs and load the heavy surfaces at their route boundaries. The frozen report has no independent route tree and uses captured execution selection within the report.
4. Keep the report Vite single-file build and `target/generated/report/report.html` output. Preserve the `hub-report-data` embedding and native decoder contract. Keep the desktop's existing report-generation task and Tauri asset entry.
5. Port in dependency order through plots, document, report, and desktop. Each producer has independent package proof at its boundary. Consumers still using React need their corresponding migration before they can consume the Svelte component exports. The final workspace delivers the framework replacement together, without compatibility exports.
6. Compare Light and Dark captures from the same browser viewport, fixtures, and interactions. Port behavior and DOM assertions to Svelte Testing Library. Keep unit and integration deadlines and shared native scenarios. Replace React-specific benchmark fixture and instrumentation code while retaining the measurements the system driver consumes.
7. Use TypeScript 6 for the package-local Svelte typechecks. The official Svelte checker requires its TypeScript 5 or 6 API even when enabling TypeScript 7 diagnostics. A single supported compiler per migrated package keeps the typecheck command and dependency graph smaller.

## Source of Truth

The requested `docs/architecture.md` and the architecture path named by `.claude/skills/dokime-architecture/SKILL.md` are absent in this checkout. The current OpenSpec requirements, package boundaries, host clients and tests establish the implementation baseline. Documentation under `docs/` is owner-managed.
