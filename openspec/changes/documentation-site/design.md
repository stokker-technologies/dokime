# Design

## Site Layout

```text
/                            redirects to /latest/, or /dev/ before the first release with an archive
/latest/                     redirects each page to the same page of the newest stable release line
/<line>/                     guides of that release line (MAJOR.MINOR), built from its latest patch
/<line>/reference/operations/
/<line>/reference/rust/
/<line>/reference/python/
/<line>/reference/csharp/
/<line>/reference/cpp/
/dev/                        the same tree, built from the dev branch
/versions.json               the entries the selector offers, newest first
```

The selector offers `latest`, each release line and `dev`. `latest` resolves to the newest stable
release line; a prerelease or Nightly never becomes `latest`.

An entry is built once with its own base path (`/<line>/` or `/dev/`) and never rebuilt, so
`/latest/` holds redirect pages instead of a second copy built for another base. The selector reads
`/versions.json` at view time, so an older entry also offers releases published after it; without
that file, as in a local preview, it shows only the current entry.

## Site Project

The Starlight project lives in `sites/docs/` and reads the maintained guides in `docs/` in place, without
copies. Its loader takes each page title from the guide's first heading and drops that heading
from the body; relative links between guides become site routes and links to other repository
files point to the repository browser at the built revision. The build fails on a broken page link
or heading anchor. The organization logo and typefaces are referenced from `organization/`.

## Shared Frontend Packages

The owner locked the extraction on 2026-11-21: pnpm workspace packages own the
existing components, with no compatibility paths or duplicate renderers.
`packages/plots` owns display-only plot components and data shaping, retaining
uPlot and the existing d3 modules. `packages/document` owns document rendering
and consumes plots. `packages/report` owns the standalone template, embeds host
protobuf responses and reuses the live decoder without importing RPC clients or
transports. Desktop consumes the packages; the documentation site consumes plots
and document rendering where shown. Nothing shared imports the desktop.
Formatting stays fixed without digit grouping, with a decimal point, and labels
carry units as `Mean [Mbps]`.

The owner resolved the remaining extraction decisions on 2026-11-21. There is
no packages/dokime and no fourth foundation package. Shared TypeScript, Biome and
Vitest settings live once at the workspace root and package configurations extend
them. Foundational UI primitives, formatting, presentation types and plot state
needed below the document boundary belong to plots; document-specific composition,
response decoding and shared report shell presentation belong to document.

The extraction inventory includes all plot components and their shaping, group
grid, histogram/statistics views, shared UI dependencies and deterministic tests;
the document renderers, native document assembler and host-free response projections;
the report entry, HTML, Vite build and captured-response materialization; desktop
imports and native recovery/save/palette providers; site demonstration components;
workspace manifests/lock, common tool configuration and frontend build/test tasks.
The native plot reads and settings dialog remain desktop-owned. No shared module
may import the desktop, including erased type imports.

ErrorCard presentation is host-free and accepts recovery behavior through a
presentation provider populated by the desktop. Traceability JSON saving has the
same boundary: desktop supplies its native save callback, while the frozen report
uses the browser download. This second import leak was discovered while tracing
the report graph; both are real boundary defects. Palette selection remains a
desktop concern and the shared plot provider receives its resolved colors.
The document-plots forwarder is deleted and consumers import grouping directly.
Generated host messages remain canonical; local presentation state must not duplicate
wire fields. Changes under docs/ and provider workflows remain owner-owned and
are reported with exact follow-up locations.

## Owner Integration Requirements

The benchmark consumer is explicitly outside this worker's ownership. Its clean
migration requires these import replacements in
`crates/dokime-gui/benches/system/surface.tsx`: line 9 to
`@dokime/document/lib/api/native-documents`, line 11 to
`@dokime/plots/components/document/PlotGroupGrid`, line 12 to
`@dokime/plots/lib/types`, line 14 to `@dokime/plots/lib/rpc/canonical`, line 16 to
`@dokime/plots/stores/summary-window-store`, and the stylesheet at line 17 to
`@dokime/document/styles.css`. The owner must update the benchmark's tsconfig
and Vite aliases if its standalone builder duplicates the old generated paths.
No compatibility module or compiler alias retains the old locations.

### Provider Workflow Edits Required

The owner must update `tools/python/src/dokime_tools/ci/matrices.py` with one
package-test row per package and configured platform, using the existing frontend
artifact/bootstrap contract and unique suite names. The command for each row is
`test:typescript:integration -- --project packages/<name> --select tests`, where
`<name>` is `plots`, `document` or `report`; retain the default dev/release profile
selected by the workflow and the matching `build-clients-{profile}-{platform}`
archive. The frontend task restores the generated messages before package tests.
`validate-test.yml` already accepts these commands and publishes each job's reports;
`ci.yml` and `nightly.yml` must retain those package rows when selecting TypeScript
validation. Package tests are separate jobs so their report directories cannot
overwrite the desktop's evidence. Keep tests off ordinary dev pushes.

In `.github/workflows/validate-build-docs.yml`, append `rust` to the
`MISE_ENABLE_TOOLS` allowlist and add
`mise run typecheck:typescript -- --project sites/docs` after the canonical host
message generation and before building the site. The shared examples consume
canonical generated messages, so docs build/serve now use the existing
`generate:host-frontend` task; its Cargo metadata query locates the exact pinned
library descriptor but compiles no Rust. This existing generator requirement is
an additional tool cost for documentation-only jobs, requiring owner review.
Do not introduce a second descriptor lookup or generated message family.

`tools/python/src/dokime_tools/ci/impact.py` should classify `packages/` and the
root frontend configuration/lock inputs as both `gui` and `docs`, rather than
falling back to every language. Its existing `sites/` rule continues to select
documentation. Workspace install caches already use the root pnpm lockfile.

### Owner Documentation Edits Required

- `docs/gui.md:4`: presentation now lives in the three workspace packages; the
  desktop retains native reads, recovery, plot editing and application composition.
- `docs/gui.md:10`: organization presentation assets are generated in
  `packages/document/src/generated/organization`; the report template is built at
  `target/generated/report/report.html` from `packages/report/report.html`.
- `docs/architecture.md:647`: report capture embeds canonical host protobuf
  responses and uses the same host-free decoder as live document rendering;
  report -> document -> plots is the dependency direction.
- `docs/tooling.md:279`: report building belongs to the report package; the GUI
  retains the generate:report orchestration task, not a build:report npm script.
- `docs/tooling.md:308`: generate:report writes target/generated/report and the
  package tests run with the --project selector; docs build/serve prepare messages.

### Remaining Overlap Requiring Owner Decision

`crates/dokime-gui/tools/host-bindings.mjs:61` (`generate`) and line 139
(`generateFrontend`) both run Cargo metadata and read the exact pinned library's
protocol descriptor. The generator entry invokes `generate()` unconditionally and
then `generateFrontend()` for `--frontend`, so the frontend generation task performs
both lookups. The first supplies native dispatch/digest generation; the second
supplies the TypeScript message/client generation. Keep both output owners, but
prefer deleting the second lookup and passing one immutable descriptor input from
the entrypoint. Neither lookup has been removed without the owner's decision.

A second retained parallel view family is `ExecutionMetadata` in
`packages/plots/src/lib/types/browse.ts:98` and `TestExecution` in
`packages/plots/src/lib/types/test.ts:296`. The former is populated by
`crates/dokime-gui/src/lib/api/native-checkpoints.ts:174` for Explore; the latter
is populated by `packages/document/src/lib/api/native-execution-projection.ts`
for the live workspace and frozen report. Both repeat identity, lifecycle and
timestamp presentation, with different names and state vocabularies. Prefer
retiring the browse metadata family after composing its browse-only state with
the generated execution detail. Neither parallel family is deleted without the
owner deciding the shared view shape; serialized plot duplicates are retired.

### Recovery and Test Fixture Boundaries

The migration preserves recovery and fixture behavior with the assertions intact.
The desktop's `failureRecovery` owns settings navigation and plugin restart;
ErrorCard receives the capability through its presentation provider. Thus
`Backend Recovery Actions Take Precedence Over Generic Retries` must supply the
real desktop recovery factory, while report rendering supplies no native recovery.
`Repeated Remote Settings Loads and Failures Stay Inline`, `Export Endpoint Preview`
and `Unified Desktop Settings Supply Remembered Parameters` mock the environment
predicate at its new shared owner, not the native open-path adapter.
`Application Theme Selects the Plot Palette Without a Browser Theme Provider`
exercises the desktop's appearance selection; shared plots receive that palette.
`Live Document Responsiveness` retains its native fixture resolver alongside the
report template builder. These were real migration defects in test wiring, not
superseded behaviors; none of these tests or their bounds is removed.

The mandatory bindings gate currently reports missing library/starter imports and
SDK type diagnostics outside this worker's ownership. It remains a repair obligation
for the SDK owner; this extraction does not replace that missing capability.

## Archives

| Channel | Asset | Source | Site Entry |
| --- | --- | --- | --- |
| Stable release | `dokime-docs-<version>.zip` | the release review of the tag | `/<line>/`, and `/latest/` when newest |
| Nightly | `dokime-docs-nightly.zip` | the `dev` branch | `/dev/` |

An archive holds the built entry at its root and a `docs.json` stamp naming the entry, version and
built revision. A release line links repository files at its tag; the development entry links them
at its revision. `docs:build -- --archive` packs it as `target/docs/dokime-docs.zip`; release
staging copies it under its channel's asset name after checking the stamp against the release, and
the release index classifies it as `docs`. It holds the guides and the references available for its
source, generated with the tools that source pins.

## Assembly

`docs:image` lists the published releases (it needs `GH_TOKEN`), downloads the newest stable
archive of each release line and combines them with `target/docs/dokime-docs.zip` built from the current checkout, unpacks each under its entry, writes
`versions.json`, the `/latest/` redirects and the root redirect, and packs the image as
`target/docs/dokime-docs-site.zip`. `--archive <path>` assembles local archives instead. It installs
no language toolchain and builds no old release. Every release line that published an archive
appears; only releases that published an archive can appear.

The default assembly requires a development archive stamped with the current commit; it never
uses the published Nightly archive as the development entry. Local `--archive` inputs retain the
same assembly path. Nightly downloads its documentation build artifact from the same workflow run,
assembles it with published stable lines through `docs:image`, and builds `sites/docs/Dockerfile`
from `target/docs/image/`. The digest-pinned static server image carries the source revision label.
It publishes only `ghcr.io/stokker-technologies/dokime-docs:nightly`, using the workflow token and
job-local `packages: write`; `latest` is reserved for stable image publication. Registry visibility
is an owner setting, and a new package inherits the registry default.

## Local Authoring

`mise run docs:serve` serves `http://127.0.0.1:4321/dev/`. Astro's native glob loader watches the
whole `docs/` root, including additions, and rerenders changed Markdown and D2 fences. No guide
copy or parallel file watcher is maintained.

## References

The operations reference is generated by repository tooling from the protocol definitions and
their comments (a `buf` descriptor set with source information) as site pages, one per protocol
package, so it shares the site's navigation and search. Each SDK generator writes its own static
pages under `reference/<name>/`; the guides link to them. They are not converted into site pages.

Generator currency was verified on 2026-11-16 against the upstream release records; whether each
generator handles its SDK as built is verified when the SDK references are generated.

| Reference | Source | Generator | Latest Release |
| --- | --- | --- | --- |
| Operations and messages | protocol files and their comments | repository tooling over `buf build` | pinned `buf` 1.75.0 |
| Rust SDK | doc comments | Rustdoc | ships with the pinned Rust toolchain |
| Python SDK | docstrings and type stubs | Sphinx AutoAPI (candidate, static analysis) | 3.9.0, 2026-10-31 |
| C# SDK | XML documentation comments | DocFX | 2.81.0, 2026-09-25 |
| C++ SDK | header comments | Doxygen | 1.18.0, 2026-08-13 |

`protoc-gen-doc` was not adopted: its latest release (1.5.1) dates from 2022-02-18. For Python,
Sphinx AutoAPI is the candidate because it analyses sources and stubs statically; `pdoc` imports the
module and its latest release (16.0.0) dates from 2025-10-27. Stub handling is confirmed with 2.2.

The Python SDK is a pure package. Its reference must cover authored APIs and the public generated message/stub surface; a native module is not required.

## Tasks

```text
docs:build    guides and references for the current checkout, into target/docs/<entry>/
              --release builds the release line of the current version; --archive packs it
docs:serve    local preview of the development entry
docs:image    assemble the current development archive and published stable lines into the site image
```

## Pipelines

- Pull requests and `dev` pushes run the documentation build check (`Build / Documentation`), a
  required job.
- A release review builds its release line; the release review requires that artifact, the
  rehearsal stages it and stable publication refuses to publish without it.
- Nightly builds the archive in a parallel job that may fail; publication depends only on build and
  package, and the final report attaches the archive when it was built and says whether it was
  attached.

Documentation-only pushes and reviews select only the documentation job: `docs/`, `sites/`,
Markdown anywhere and `openspec/`. Documentation Python tooling and impact-rule changes also use
that job, which runs tooling checks and focused tests; they select no product jobs. Mixed changes
union their product selections.

Nightly freshness supplies the published `nightly` tag revision to the existing impact planner.
Both scheduled and manual runs skip when it equals the current revision. A documentation-only
diff runs the documentation build and its independent image consumer; product validation,
packaging, publication and release reporting are skipped. The product comparison base remains
recorded in `refs/tags/nightly`: documentation-only runs never move that tag or edit its release,
so the next product run still compares against the commit that produced the published binaries.
With no tag, or an unavailable comparison, planning selects the complete eligible product work.
The image job runs only on `dev` and is absent from publication dependencies, so its failure
cannot block release publication.

## Search

Search is static (Pagefind, built with each entry) and needs no external service, so the site
works on a private deployment.
