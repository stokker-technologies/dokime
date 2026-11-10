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

## Distribution

- A versioned source archive with generated protocol code on each GitHub release, with its hash.
- Projects pull it through CMake `FetchContent`, pinned by URL and hash. No binary SDK, because
  C++ binaries depend on the compiler and its settings.
- gRPC and protobuf come through a vcpkg manifest in the starter.
- mise pins CMake and Ninja; the compiler comes from the system.
- A vcpkg port or Conan package can follow once stable releases exist.
- Authors ship a `.dokime` bundle per platform, statically linked where possible.

## Alignment With gRPC Plugins

Recorded on 2026-11-10. This change follows `grpc-plugins`:

- The SDK has two halves: a plugin half over the Harmos SDK for the language
  and a client half over `dokime.v1`.
- A plugin is a gRPC server. A step, criterion, initialize and cleanup are
  operations the host calls; there is no run loop in the SDK.
- `dokime proto export` provides the `.proto` files; there is no
  `dokime bindings` command.
- C++ is part of the pilot with Rust, Python and C#.
