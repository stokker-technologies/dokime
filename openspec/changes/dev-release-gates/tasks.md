## Implementation

- [x] 1.1 Own dev checks/build selection in the project workflows, with no dev test suites; remove impact planning and preserve native platforms and caches.
- [x] 1.2 Restrict release dry validation to release PRs; reject release git dependencies and make nightly report them as warnings.
- [x] 1.3 Add API validation of the target SHA before release tagging and publishing, including pending, missing and failed evidence.
- [x] 1.4 Draft dev publication before replacing assets, verify the complete upload inventory, move the tag once and publish; serialize publication.
- [x] 1.5 Update contributor/tooling ownership documentation and remove obsolete impact-planning guidance.

## Validation and Delivery

- [ ] 2.1 Run the requested mise check and tooling proof with UV_PROJECT_ENVIRONMENT removed; report every failure without editing product/test code.
- [ ] 2.2 Deliver only with authorization through the combined feature candidate; preserve remote SHA and actual hosted proof. This local specification task authorizes no push.
- [ ] 2.3 Retire the specification after the requested proof and handback obligations are fulfilled.

## Stable Release Prerequisites

- [ ] 3.1 Verify live GitHub protected environments, release identity/token, signed preparation commits and crates.io/PyPI publishing identities; workflow declarations alone are not provider proof. Obtain administrator action for any unavailable setting rather than inventing a reporting or publishing identity.
- [ ] 3.2 Verify exact-release-commit green required checks and artifact reuse before stable tagging/publication; preserve the existing ungated dev policy and mandatory hooks.
- [ ] 3.3 Establish a working private vulnerability reporting channel: have the repository administrator enable the designated GitHub channel or have the owner choose a reachable alternative; verify it accepts private reports and update docs/SECURITY.md only against that evidence.
- [ ] 3.4 Before stable publication verify packaged license texts/notices and the reviewed advisory exception register at the release date, including the November 22–23 expiries; resolve expired entries without relaxing the supply-chain gate.
