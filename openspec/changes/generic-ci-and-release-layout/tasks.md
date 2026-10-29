# Tasks

## Implementation

- [x] 1.1 Triage existing runtime: archive assembly is already implemented at tools/python/src/dokime_tools/package/run.py:102; feed/upload producers at release/layout.py:109 and :133; credential-free rehearsal at release/run.py:454 and :560. Preserve them.
- [x] 1.2 Adapt product policy and package task interfaces, tool metadata and payload names; remove superseded release declarations (stkr.toml:599, mise.toml:476, crates/cargo-dokime/Cargo.toml:13, package/payload.py:90).
- [x] 1.3 Verify signed release downloads from latest.json without publishing the signature sidecar; align Windows installer/feed names and update deterministic installer tests (package/installer.py:252; installer/native.rs:395).
- [x] 1.4 Update README, product guide, example walkthrough and testing guidance with archive patterns and test-only OBC ownership (docs/product-guide.md:5; .agents/skills/dokime-testing/SKILL.md:10).
- [x] 1.5 Declare the shipped host-plugin package and native bundle inputs once its producer owner supplies their exact identity and output filename. The owner-selected next-release branch supplies dokime-host and host.dokime; integrate its producer before declaring the matching release input.

- [x] 1.6 Collect release.plugins shipping packages from the existing compiler record; keep OBC in explicit test/bench fixture exports, without another build or hardcoded shipped-package list; prove selection and deduplication.

- [x] 1.7 Merge the owner-selected next-release branch into this render worktree; reconcile product-owned tasks, host collection, package metadata, docs and skills, re-render managed outputs, preserve managed workflow deletions and restore every Check/Test hold with Release / Dry enabled. Normalize imported OpenSpec capability size and revision references without changing requirements.

- [x] 1.8 Normalize the imported Rust/Cargo skeletons reported by project check with the canonical formatter; preserve source behavior and re-prove affected native builds.

## Validation

- [x] 2.1 Run changed installer tests: targeted Python integration selection passes all 25 selected installer/plugin cases, including payload bytes/modes, embedded signatures and declared package collection.
- [x] 2.2 Run strict OpenSpec validation for all changes/specs; re-render workflows, restore all 15 Check/Test conditions with a (false) prefix, keep Release / Dry enabled and retired managed workflows absent. Actionlint passes.
- [x] 2.3 Complete the requested local-platform release dry task against matching packaged Windows Product exports: package:rust --profile dev --target native and release:publish:dry pass, including archive/installer/payload assembly, eight staged downloads and registry dry runs without uploads. All-target release:rehearse reports missing release-profile packages; keep that rehearsal and required Unix validation with the combined integration owner.
- [ ] 2.4 Resolve the remaining Hub task-category conflict: project check now rejects only mise.toml because the locked measure:report-save task has no canonical section. Imported source skeletons and OpenSpec metadata are repaired; the owner-held CI digest remains intentional.
- [x] 2.5 Prove the Windows shipped host bundle and staged plugins.json: actual bundle inspection reports dokime-host version 0.1.0 and the Windows MSVC ABI; target/release-assets/any/plugins.json and upload.json contain its matching public asset, digest and size, with no OBC, examples, Python artifacts or standalone signatures.

## Workflow Follow-Up

- Hub adoption task 2.1 remains owned by the Hub integration checkout; report this worktree's commits and remaining proof without editing that checkout.
- The integrated producer owns dokime-host and plugins/host.dokime. All-target rehearsal also requires the Unix package artifact; local Windows proof cannot replace it.
- Full native validation and delivery remain with the integration owner; this worker never pushes. Local Windows package/feed/dry-run proof passes; the Hub skeleton conflict remains open. No final all-platform proof or delivery is authorized.
