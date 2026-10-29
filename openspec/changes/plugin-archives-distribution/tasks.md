# Implementation Tasks

## Source Inventory

- [ ] 1.1 Add shared v2 feed contracts and verified ZIP/tar.gz member extraction in domain/entry library; prove selection, corrupt hashes, unsafe/duplicate/link members and v1 refusal.
- [x] 1.2 Replace desktop feed v1 and per-plugin download with multi-plugin v2 discovery and archive/member verification; preserve generation fences and test installer behavior.
- [x] 2.1 Replace manifest release_url/sha256 with optional version, named plugins and external plugin tables; retain native/external validation and repeatable CLI flags.
- [x] 2.2 Resolve distribution and plugin archives from versioned/dev-channel release assets and SHA256SUMS; implement verified per-user digest cache and local served-file tests.
- [x] 3.1 Rename product payload code, exports, metadata, errors and tests to distribution; package declared sidecars archives and update authored stkr.toml declarations without generated release edits.
- [x] 4.1 Update README, product guide, plugins and organization-pack guidance; remove superseded v1 feed generator/tests and stale release instructions.
- [ ] 5.1 Run targeted dokime-packaging, cargo-dokime installer, desktop plugin and Python packaging proof; record remaining integration obligations in handoff.

## Integration Owner Follow-Up

- [ ] 6.1 Combine product work with Hub-rendered layout and other release work, then perform authorized final all-platform validation and delivery outside this worktree assignment.
