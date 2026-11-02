## Implementation

- [x] 1.1 Extend the serialized GitHub publisher with UTC dev tag formatting, collision refusal, draft upload verification and immutable publication before channel replacement.
- [x] 1.2 Add named retention policy and workflow provenance filtering; delete only eligible older pre-releases and their exact tags.
- [x] 1.3 Restrict stable workflow tag events and reject nonstable release evidence without changing #199's gates.
- [x] 1.4 Update CONTRIBUTING, README and product guide with exact mise dev pins, matching revisions and retention.
- [x] 1.5 Prove formatting, retention, publication failure/order/collision behavior and stable exclusion with focused tooling tests; repair and locally validate callback fixture bindings reported by the first hosted Python lint check.

## Validation And Delivery

- [x] 2.1 Run the requested mise check with UV_PROJECT_ENVIRONMENT removed and report actual results; the repaired local candidate passes.
- [x] 2.2 Push the combined candidate without force and verify the remote SHA; open one ready PR into dev with the requested title, leaving auto-merge disabled.
- [ ] 2.3 Report the single hosted CI run and retire this specification after handback obligations are satisfied.
- [ ] 2.4 Publish the locally validated test callback repair only after the owner permits the resulting additional CI run.

## Workflow Follow-Up

- The active Stable Release Tags ruleset includes all `refs/tags/v*` without exclusions and blocks the native workflow token's dev-tag creation/deletion. A dev-only exclusion requires owner authorization; no provider setting has been changed.
- The single-run instruction prevents pushing the validated callback repair without direction. Full local checking and the first hosted run remain the requested proof; do not manually rerun CI or enable auto-merge.
