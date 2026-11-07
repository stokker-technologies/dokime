## Implementation

- [x] 1.1 Update impact action, Python planner and simulation graph for dev package scope; remove redundant release compilation and preserve caches.
- [x] 1.2 Restrict release dry validation to release PRs; reject release git dependencies and make nightly report them as warnings.
- [x] 1.3 Add API validation of the target SHA before release tagging and publishing, including pending, missing and failed evidence.
- [x] 1.4 Draft dev publication before replacing assets, verify the complete upload inventory, move the tag once and publish; serialize publication.
- [x] 1.5 Update CONTRIBUTING and tooling ownership wording; review README impact and verify changed behavior with focused tooling tests and impact evaluation.

## Validation and Delivery

- [ ] 2.1 Run the requested mise check and tooling proof with UV_PROJECT_ENVIRONMENT removed; report every failure without editing product/test code.
- [ ] 2.2 Push and verify the remote SHA, open one ready PR into dev and report its single CI run; do not merge or enable auto-merge.
- [ ] 2.3 Retire the specification after the requested proof and handback obligations are fulfilled.
