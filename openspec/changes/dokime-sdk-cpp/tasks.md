# Tasks

## 1. Implementation

- [ ] 1.1 Add the SDK over Harmos's C++ SDK with the shared concept names.
- [ ] 1.2 Add `cargo dokime` support for building and bundling a C++ plugin.
- [ ] 1.3 Add a sample plugin, a starter template and documentation.
- [ ] 1.4 Add the Nightly system test and update the README comparison table.
- [ ] 1.5 Add the `dokime new --lang cpp` template, and teach `sidecar dev` and `sidecar package` the C++ build through its collected-build file.
- [ ] 1.6 Verify vcpkg gRPC on Linux and Windows and mise's CMake and Ninja, then publish the source archive and hash on releases.

## 2. Final Validation

- [ ] 2.1 Run targeted tests and the sample plugin on Linux and Windows.

## Workflow Follow-Up

- Delivery uses targeted validation and merges to dev without waiting for CI; breakage is fixed in follow-ups.
- SDK system tests run in Nightly only, not on pull requests.
