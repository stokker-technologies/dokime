# Dokime Protocol Contracts

`dokime/v1/` contains the checked-in execution, plugin, context-operation, catalog
and document contracts. `harmos/v1/plugin.proto` is a read-only Harmos copy.
Existing client gRPC services remain generated from `crates/dokime-operations`
and are not relocated by these definitions.

Generate Rust with `mise run generate -- --protocol`; verify drift with
`mise run test -- --package dokime-protocol --target integration` (or
`mise run generate -- --protocol --check`). The pinned vendored compiler needs
no installed `protoc`. Normal builds consume checked-in generated Rust.
`dokime-protocol` exposes `dokime_protocol::v1` and its local Harmos types under
`dokime_protocol::harmos::v1`; it has no dependency on the host or a Harmos crate.

Export these sources with `dokime proto export --out <directory>`. The output
contains only `.proto` files and preserves their package directories. Standard
`google/protobuf` imports come from your protobuf toolchain. Plugin-specific
export is not available yet; obtain those sources from the plugin author.

Within `v1`, evolve additively: preserve field names, numbers, types and meanings;
reserve removed names and numbers; ignore unknown fields and enum values. Every
enum has an unspecified zero. Incompatible changes require `v2` alongside `v1`
for one release.
