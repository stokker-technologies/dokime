# Documentation Site

## Why

The guides under `docs/` are plain Markdown read on the repository host. There is no website, no
API reference for the SDKs or the operations, and nothing shows a reader which release a page
describes. With four SDK languages the reference has to be generated from source, per release.

## What Changes

- One documentation website per product, served from one deployed image that contains the pages of
  every supported release line, a `latest` entry for the newest stable release and a `dev` entry
  built from the `dev` branch.
- Each stable release and each Nightly publishes one documentation archive: the built guides and
  the references for exactly that source.
- The site image is assembled from those archives; it never rebuilds an old release.
- References are generated for the operations (from the protocol files) and for the Rust, Python,
  C# and C++ SDKs.
- `docs:build`, `docs:serve` and `docs:image` tasks.

## Decisions

Agreed with the owner on 2026-11-13; the open points below were decided on 2026-11-14 and
2026-11-16, and phase one is implemented. The SDK references wait for the SDK restructures.

- Old releases are never copied into the repository. A release's documentation is the documentation
  at its tag. Maintained Markdown keeps stating current contracts only.
- One entry per release line showing its latest patch, plus `latest` and `dev` (2026-11-14).
- The site generator is Starlight (2026-11-16).
- Installers do not bundle the documentation. The website is the way documentation is read.
- The documentation archive is a required asset of a stable release. In Nightly it is built by a
  parallel job that may fail; Nightly publication does not depend on it.

## Open

- Whether each SDK generator handles its SDK as built; verified when the SDK references are
  generated after the restructures. Their currency is recorded in design.md.
- The deployment target of the site image.

## Impact

- `docs/`, a new site project, the release and Nightly workflows, the release asset layout and its
  index, mise tasks.
- Doc comments become a deliverable in every SDK.
- Follows the SDK restructures and the examples, because the references are generated from them.
