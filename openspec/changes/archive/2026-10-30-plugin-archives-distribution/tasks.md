# Implementation Tasks

## Source Inventory

- [x] 1.1 Add shared v2 feed contracts and verified ZIP/tar.gz member extraction in domain/entry library; prove selection, corrupt hashes, unsafe/duplicate/link members and v1 refusal.
- [x] 1.2 Replace desktop feed v1 and per-plugin download with multi-plugin v2 discovery and archive/member verification; preserve generation fences and test installer behavior.
- [x] 2.1 Replace manifest release_url/sha256 and superseded plugin keys with optional version and top-level [[plugin]] entries; reject both/neither name/path with one-based entry indices in manifest.rs and retain repeatable CLI flags.
- [x] 2.2 Resolve distribution and plugin archives from versioned/dev-channel release assets and SHA256SUMS; implement verified per-user digest cache and local served-file tests.
- [x] 3.1 Rename product payload code, exports, metadata, errors and tests to distribution; package declared sidecars archives and update authored .stkr/project.toml declarations without generated release edits.
- [x] 4.1 Update README, product guide, plugins and organization-pack guidance; remove superseded v1 feed generator/tests and stale release instructions.
- [x] 5.1 Run targeted dokime-packaging, cargo-dokime installer, desktop plugin and Python packaging proof; record remaining integration obligations in handoff.
- [x] 7.1 Resolve explicit path entries against the selected organization pack with confinement checks in installer/manifest.rs and mod.rs; plugins.rs no longer discovers folders or overlays filenames, and retains bundle integrity, target and compatibility checks.
- [x] 7.2 Replace implicit-bundling tests with explicit selection, unlisted-folder exclusion, indexed invalid-entry and lexical path-escape proof in packaging unit/integration tests; retain Unix link-escape proof for native Linux validation.
- [x] 7.3 Update proposal/design/spec, README, product guide organization/installer examples, docs/plugins.md and authored/generated architecture guidance; product Python declarations already use explicit --plugin inputs (package/installer.py:83), so no conversion is needed. Sibling changes and Hub-rendered release files remain outside ownership.
- [x] 7.4 Run targeted mise packaging and cargo-dokime proof, strict OpenSpec validation and record specification retirement eligibility before committing the amendment once without pushing.

## Integration Owner Follow-Up

- [x] 6.1 Combine product work with Hub-rendered layout and other release work, reconcile superseded implicit plugin-folder requirements in the host-plugin-features change, then perform authorized final all-platform validation and delivery outside this worktree assignment.
