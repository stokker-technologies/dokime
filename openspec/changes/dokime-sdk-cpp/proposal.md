# Dokime SDK C++

## Why

Test engineers should write Dokime suites in C++, on top of Harmos's C++ SDK.

## What Changes

- Add `sdk/cpp` with suites, templates, parameters, steps, criteria, prompts, records and
  documents, using the shared concept names.
- Bundle a C++ sidecar as a plugin the desktop can install.
- Add a Nightly system test that runs a C++ plugin end to end.

## Locked Decisions

- A thin layer over Harmos's SDK, pinned to a Harmos revision.
- Depends on `harmos-sdk-cpp`.
- Sidecars only.
