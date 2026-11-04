# Generic CI And Release Layout Adoption

## Why

Adopt the locked Hub `generic-ci-and-release-layout` design, product adoption 2.1, on Dokime's freshly rendered branch. The owner authorizes product-owned changes and targeted proof only; no push or release.

## What Changes

- Declare tool archives, shipped host-plugin bundles and feed, portable payload ZIPs and native installers as the release inventory.
- Remove OBC, Python, examples and standalone signatures from release downloads; keep Python registry publication and local candidate examples.
- Align cargo-binstall metadata, mise archive examples, package task arguments and manifest signature verification.
- Preserve all owner Check/Test holds and enabled Release / Dry.

## Impact

Product policy, mise tasks, cargo-dokime metadata, package tooling, installer tests, README and product guide. Hub-rendered tools already implement archives, feeds, upload manifests and credential-free rehearsal and remain untouched.
