# Native Desktop Test Parity

## Why

The local Windows validation currently skips desktop system journeys that run on Linux CI. This allows UI failures to reach the PR without local proof. The current Linux run also exposed a cancelled provider group being resubmitted during shutdown.

## What Changes

- Require the same ordinary desktop system journeys on Windows and Linux, locally and in declared CI jobs.
- Remove OS-based story skips while retaining native driver and installer adaptations.
- Resolve and verify the matching Windows WebView2 driver during authored bootstrap, without depending on manual runner configuration.
- Make native clicks target a stable visible element and retain actionable failure evidence without extending deadlines or relaxing assertions.
- Prevent provider-group advancement from resubmitting cancelled pending work and cover the stopping transaction with a regression test.

## Impact

Dokime's mise tasks, stkr CI declarations, desktop bootstrap/harness, runtime provider completion, tests, and maintained testing guidance are affected. Hub owns any canonical graph correction; generated consumers are refreshed from Hub, never edited directly. Existing signed PR and branch remain the integration path. No release or branch protection changes are introduced.
