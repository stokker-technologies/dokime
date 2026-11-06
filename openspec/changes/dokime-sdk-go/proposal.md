# Dokime SDK Go

## Why

Test engineers should write Dokime suites in Go, on top of Harmos's Go SDK.

## What Changes

- Add `sdk/go` with suites, templates, parameters, steps, criteria, prompts, records and
  documents, using the shared concept names.
- Bundle a Go sidecar as a plugin the desktop can install.
- Add a Nightly system test that runs a Go plugin end to end.

## Locked Decisions

- A thin layer over Harmos's SDK, pinned to a Harmos revision.
- Depends on `harmos-sdk-go`.
- Sidecars only.
