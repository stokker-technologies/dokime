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

## Distribution

A Go module under `sdk/go`, installed with `go get` and released with subfolder tags. mise pins Go.

## Alignment With gRPC Plugins

Recorded on 2026-11-10. This change follows `grpc-plugins`:

- The SDK has two halves: a plugin half over the Harmos SDK for the language
  and a client half over `dokime.v1`.
- A plugin is a gRPC server. A step, criterion, initialize and cleanup are
  operations the host calls; there is no run loop in the SDK.
- `dokime proto export` provides the `.proto` files; there is no
  `dokime bindings` command.
- Go is outside the pilot and follows it.
