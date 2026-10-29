# Tasks

## Implementation

- [x] 1.1 Triage existing runtime: archive assembly is already implemented at tools/python/src/dokime_tools/package/run.py:102; feed/upload producers at release/layout.py:109 and :133; credential-free rehearsal at release/run.py:454 and :560. Preserve them.
- [x] 1.2 Adapt product policy and package task interfaces, tool metadata and payload names; remove superseded release declarations (stkr.toml:599, mise.toml:476, crates/cargo-dokime/Cargo.toml:13, package/payload.py:90).
- [x] 1.3 Verify signed release downloads from latest.json without publishing the signature sidecar; align Windows installer/feed names and update deterministic installer tests (package/installer.py:252; installer/native.rs:395).
- [x] 1.4 Update README, product guide, example walkthrough and testing guidance with archive patterns and test-only OBC ownership (docs/product-guide.md:5; .agents/skills/dokime-testing/SKILL.md:10).
- [x] 1.5 Declare the shipped host-plugin package and native bundle inputs once its producer owner supplies their exact identity and output filename. The owner-selected next-release branch supplies dokime-host and host.dokime; integrate its producer before declaring the matching release input.

- [x] 1.6 Collect release.plugins shipping packages from the existing compiler record; keep OBC in explicit test/bench fixture exports, without another build or hardcoded shipped-package list; prove selection and deduplication.

- [x] 1.7 Merge the owner-selected next-release branch into this render worktree; reconcile product-owned tasks, host collection, package metadata, docs and skills, re-render managed outputs, preserve managed workflow deletions and restore every Check/Test hold with Release / Dry enabled. Normalize imported OpenSpec capability size and revision references without changing requirements.

- [ ] 1.8 Normalize the imported Rust/Cargo skeletons reported by project check with the canonical formatter; preserve source behavior and re-prove affected native builds.

## Validation

- [x] 2.1 Run changed installer tests: targeted Python integration selection passes all 25 selected installer/plugin cases, including payload bytes/modes, embedded signatures and declared package collection.
- [x] 2.2 Run strict OpenSpec validation for all changes/specs; re-render workflows, restore all 15 Check/Test conditions with a (false) prefix, keep Release / Dry enabled and retired managed workflows absent. Actionlint passes.
- [ ] 2.3 Prove native package/release dry and rehearsal using matching Product Build inputs; this checkout has no native product/target.txt and neither native release package. Task-graph package/rehearsal dry runs, Python candidate build/package, focused dokime-packaging compiler check and release:publish:dry pass; the latter staged no native target and proves registry publication only.
- [ ] 2.4 Resolve project diagnostics after the merge: normalize imported source skeletons with the canonical formatter, retain the locked measure:report-save task and report its Hub task-category conflict; only the owner-held CI digest may remain as an intentional render difference.
- [ ] 2.5 Prove shipped host-plugin bundle and plugins.json against the integrated producer (producer integrated from the owner-selected next-release branch; verify the actual native bundle and staged feed).

## Workflow Follow-Up

- Hub adoption task 2.1 remains owned by the Hub integration checkout; report this worktree's commits and remaining proof without editing that checkout.
- The integrated producer owns dokime-host and plugins/host.dokime. All-target rehearsal also requires the Unix package artifact; local Windows proof cannot replace it.
- Full native validation and delivery remain with the integration owner; this worker never pushes. Native package/feed proof and the existing Hub skeleton conflict remain open; no final all-platform proof or delivery is authorized.
