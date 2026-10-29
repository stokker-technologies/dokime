# Tasks

## Implementation

- [x] 1.1 Triage existing runtime: archive assembly is already implemented at tools/python/src/dokime_tools/package/run.py:102; feed/upload producers at release/layout.py:109 and :133; credential-free rehearsal at release/run.py:454 and :560. Preserve them.
- [x] 1.2 Adapt product policy and package task interfaces, tool metadata and payload names; remove superseded release declarations (stkr.toml:599, mise.toml:476, crates/cargo-dokime/Cargo.toml:13, package/payload.py:90).
- [x] 1.3 Verify signed release downloads from latest.json without publishing the signature sidecar; align Windows installer/feed names and update deterministic installer tests (package/installer.py:252; installer/native.rs:395).
- [x] 1.4 Update README, product guide, example walkthrough and testing guidance with archive patterns and test-only OBC ownership (docs/product-guide.md:5; .agents/skills/dokime-testing/SKILL.md:10).
- [ ] 1.5 Declare the shipped host-plugin package and native bundle inputs once its producer owner supplies their exact identity and output filename. This checkout contains only OBC; do not implement prototype-streamline here or invent a placeholder.

- [x] 1.6 Collect release.plugins packages from the existing compiler record alongside OBC fixtures, without another build or hardcoded shipped-package list; prove selection and deduplication.

## Validation

- [x] 2.1 Run changed installer tests: targeted Python integration selection passes all 21 selected installer/plugin cases, including payload bytes/modes, embedded signatures and declared package collection.
- [x] 2.2 Run strict OpenSpec validation for all changes/specs; preserve the workflow files and all Check/Test holds byte-for-byte.
- [ ] 2.3 Prove native package/release dry and rehearsal using matching Product Build inputs; this checkout has no native product/target.txt and neither native release package. Task-graph package/rehearsal dry runs, Python candidate build/package, focused dokime-packaging compiler check and release:publish:dry pass; the latter staged no native target and proves registry publication only.
- [ ] 2.4 Resolve render diagnostics with the relevant owner: final stkr project check rejects only mise.toml; stkr project format identifies the locked measure:report-save task as having no skeleton section. Native workflow bytes and product policy formatting are unchanged/clean. Do not rename the parked change's task or alter Hub-owned code here.
- [ ] 2.5 Prove shipped host-plugin bundle and plugins.json against the integrated producer (blocked here: host plugin is absent; prototype-streamline owns its implementation).

## Workflow Follow-Up

- Hub adoption task 2.1 remains owned by the Hub integration checkout; report this worktree's commits and remaining proof without editing that checkout.
- Exact host package/bundle identity is pending owner input. All-target rehearsal also requires the Unix package artifact; local Windows proof cannot replace it.
- Full native validation and delivery remain with the integration owner; this worker never pushes. Source remains incomplete while 1.5 is open.
