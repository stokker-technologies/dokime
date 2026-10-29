# Tasks

Locked 2026-10-29; scheduled by the owner. Implementation in `feat/distribution-identity`.

## Source Inventory

Installer identity and native assembly: `dokime-packaging/src/installer/{manifest,payload,native,inspect}.rs`, identity-only hooks and packaging tests; integrate the bundled-plugin startup inventory reader and its fixtures with `installer.json`. Plugin selection/staging remains with the host-plugin-features owner.

Shared identity validation: `dokime-domain/src/model/identity/distribution.rs`. Shared resolver and consumers: `dokime-runtime/src/paths.rs`, configuration snapshots and consumers (runtime settings, server diagnostics/log streams), desktop bootstrap/organization/settings/logging/local backend and native-session paths and cache consumers; server logging, CLI server startup and `dokime/src/installation.rs` GUI discovery. Preserve compiled protocol and updater identities.

Proof and guidance: resolver tests, installer tests, native desktop lifecycle identity coverage, product guide and README. Implement the shared identity contract before packaging and consumer integration. Native Windows and Unix walkthroughs remain separate validation obligations; this slice runs targeted proof only and never pushes.

Triage: `ApplicationPaths` already owns basic root/data/plugins paths (`crates/dokime-runtime/src/paths.rs:34`); extend it. Existing `distribution.json` is the inspection inventory (`crates/dokime-packaging/src/installer/inspect.rs:138`); replace its role with a separate inventory so the startup identity file itself is verified. None of 1.1–1.5 is already complete.

## 1. Implementation

- [x] 1.1 Installer manifest `[installer] identifier` with validation; payload identity checks accept the distribution identifier.
- [ ] 1.2 `cargo dokime installer package` writes `distribution.json` into the payload inventory and uses the identifier for the NSIS install folder, uninstall registry key and DEB/RPM package names.
- [x] 1.3 One path resolver (desktop, CLI, server) reading `distribution.json` with the compiled identifier as fallback; replace `app_data_dir()` and `app_log_dir(identifier)` uses; set the WebView data directory.
- [x] 1.4 Product guide: Native Installers and Organization sections; README claims.
- [ ] 1.5 Tests: resolver unit tests, packaging tests for the written file and names, a native lifecycle check that a branded and a plain installation keep separate directories.

## 2. Validation

- [ ] 2.1 Targeted proof, then the native desktop walkthrough on Windows and Unix with a branded distribution.

## Workflow Follow-Up

- Integration preserves the distribution slug in native filenames and the release layout's Rust target in NSIS filenames. Verified organization-folder and explicit plugin staging are preserved; Linux plugin destinations follow the distribution directory. Bundled-plugin startup reads and fingerprints `installer.json`; `distribution.json` retains only the startup identity. First-start installation, upgrade-only reconciliation and preservation of disabled state/settings remain with the plugin store.
- Keep 1.2 and 1.5 open for the NSIS decoding repair. A fixed, valid LZMA stream reproduces the header failure without packaging, files, environment variables or external processes. The full 8,196-byte stream decodes; its 4,100-byte prefix fails with `end reached but not decoder finished`, and `decompress_header` returns `UnsupportedCompression` (the public installer parser reports `none of the supported compression methods succeeded`). Two focused diagnostic runs reproduce it. This establishes a dependency defect, not environmental proof; the original failed installer was not retained, so its exact cause cannot be proven retrospectively.
- `nsis` 0.6.0's `decompress_lzma` supplies a known final size for `DecodeLimit::Exact`, but a solid header is a prefix of a continuing stream. `lzma-rust2` 0.21.0 rejects a pending match at that artificial end. The compressed fixture below contains a little-endian 4,096-byte header length followed by 8,192 zero bytes; the header boundary cuts a match.
- Reproduce with this temporary packaging test target (`crates/dokime-packaging/tests/nsis_diagnostic.rs`), using `mise run build:rust:focused -- test --locked -p dokime-packaging --test nsis_diagnostic -- --exact solid_header_prefix --nocapture`. The diagnostic intentionally fails at the final unwrap; it is not part of the committed test suite.

```rust
#[test]
fn solid_header_prefix() {
    use nsis::decompress::{DecodeLimit, decompress_header, lzma::decompress_lzma};
    let stream = &[
        93, 0, 0, 128, 0, 0, 0, 4, 134, 51, 209, 233, 7, 130, 254, 100, 93, 106,
        22, 213, 216, 220, 249, 7, 192, 205, 39, 6, 162, 106, 225, 181, 38, 215, 246, 172,
        215, 140, 70, 137, 71, 222, 4, 13, 152, 193, 25, 255, 255, 148, 210, 0, 0,
    ];
    let full = decompress_lzma(stream, DecodeLimit::Capped(8196)).unwrap();
    assert_eq!(full.data.len(), 8196);
    assert_eq!(&full.data[..4], &4096_u32.to_le_bytes());
    eprintln!("{:?}", decompress_lzma(stream, DecodeLimit::Exact(4100)));
    decompress_header(stream, 4096).unwrap();
}
```

- Owner question: authorize a bounded-prefix decoding repair in the dependency or assign it to the decoder/installer owner. No dependency replacement, compression workaround or weakened inspection was introduced here. Native Windows and Unix walkthroughs remain open under 2.1.
