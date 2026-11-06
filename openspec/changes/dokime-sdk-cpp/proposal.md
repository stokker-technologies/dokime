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
