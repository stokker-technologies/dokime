## Implementation

- [x] 1.1 Own dev checks/build selection in the project workflows, with no dev test suites; remove impact planning and preserve native platforms and caches.
- [x] 1.2 Restrict release dry validation to release PRs; reject release git dependencies and make nightly report them as warnings.
- [x] 1.3 Add API validation of the target SHA before release tagging and publishing, including pending, missing and failed evidence.
- [x] 1.4 Draft dev publication before replacing assets, verify the complete upload inventory, move the tag once and publish; serialize publication.
- [x] 1.5 Update contributor/tooling ownership documentation and remove obsolete impact-planning guidance.

## Validation and Delivery

- [ ] 2.1 Run the requested mise check and tooling proof with UV_PROJECT_ENVIRONMENT removed; report every failure without editing product/test code.
- [ ] 2.2 Deliver only with authorization through the combined `feat/grpc-plugins` candidate; preserve remote SHA and actual hosted proof. This local specification task authorizes no push.
- [ ] 2.3 Retire the specification after the requested proof and handback obligations are fulfilled.
