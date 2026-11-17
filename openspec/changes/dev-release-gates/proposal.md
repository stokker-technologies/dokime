# Dokime-Owned CI Gates

## Why

Dokime owns its CI after the Hub baseline is rendered. Dev merges need fresh
packages without repeating PR suites, while releases need explicit pin and
commit validation gates.

## What Changes

- Dev pushes run quick checks and builds without packages, publication or suites; PRs use the project-owned impact planner, with safe full fallback.
- Git dependencies are allowed in development and rejected for releases; nightly warns.
- Release tagging and publication verify the target commit's green CI through the API.
- Nightly assets are replaced only while drafted, then published together after both platforms.
- Document Dokime ownership and the owner-managed main ruleset without changing settings.

## Impact

Workflows, the setup action, Python CI/release tooling, tooling
tests, task/policy declarations and CONTRIBUTING are owned by this branch.
Product/test code and repository settings are excluded. Delivery is part of the combined candidate; this specification-only assignment grants no remote authority.
