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

At the assigned base revision there is no `packages/dokime` directory. Its intended
role and the location of shared presentation foundations require owner direction.
The report's current `ErrorCard` imports desktop session recovery and plugin RPC;
the extraction must preserve the existing import-boundary test's intent. Changes
under `docs/` and provider workflows belong to the owner and are reported only.

The landed independent slice moves palette resolution, point reduction,
histogram-bin shaping, viewport interaction and the React plot interaction
provider into `packages/plots/src/`, exported through explicit package subpaths.
Their old desktop locations are removed and desktop callers import the workspace
package. Native plot reads, the remaining plot components, documents and reports
still belong to the desktop pending the decisions above. Root `pnpm-lock.yaml`
replaces both application locks; cache keys and shell inventory read that lock.

One existing overlapping public path requires owner review: desktop
`src/lib/document-plots.ts` forwards plot shaping from `src/plots/grouping.ts`.
`PlotGroupGrid.tsx` uses the forwarding module and direct grouping imports, while
plot tests use grouping directly. Prefer removing the forwarding module and
giving the extracted shaping implementation one package entry when its move is
authorized; neither path is removed in the independent slice.

The existing document and plot presentation type families also require owner
review against the generated host messages before renderer extraction:
`src/lib/document-contracts.ts` and `src/lib/types/view-contracts.ts` describe
handwritten projection fields, while `target/generated/host/model.ts` exposes the
canonical messages. Native document/plot adapters consume the messages and
produce the projection families used by rendering components. Prefer composing
required local rendering state with generated fields and retiring duplicated
wire-shaped declarations; no family is removed by this independent move.

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
