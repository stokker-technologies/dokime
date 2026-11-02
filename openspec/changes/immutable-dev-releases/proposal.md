# Immutable Dev Pre-Releases

## Why

Downstream mise consumers need a permanent dev version matching their Dokime
git revision. The rolling channel alone cannot supply that pin.

## What Changes

- Publish a unique timestamped dev pre-release from the existing serialized publisher before updating the channel.
- Reuse one staged asset inventory and SHA256SUMS; draft, upload, verify, then publish.
- Retain the newest workflow-owned dev pre-releases using `release.dev_release_retention`, default 20.
- Exclude dev tags from stable Release events and release evidence classification.
- Document exact mise pins and retention; keep #199's stable gates and package requirements.

## Impact

Own `release/dev_channel.py`, `test_ci_gates.py`, `tools/project.toml`,
`dev-channel.yml`, `release.yml`, the review-release action, CONTRIBUTING,
README and the product guide. Product code, provider settings and #199's
unfinished inventory remain untouched. Delivery is one ready PR with one CI
run reported, no merge or auto-merge. The requested local proof is `mise run
check` and focused tooling tests with UV_PROJECT_ENVIRONMENT removed.
