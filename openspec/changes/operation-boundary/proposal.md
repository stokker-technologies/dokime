# Operation Boundary

## Why

Dokime still persists most product data through its own `Store` (direct SQL, Arrow reads,
receipts, cuts and notifications in `crates/dokime-storage`), re-implements statistics,
ranks, binning and payload codecs that Harmos already provides, constructs Harmos managers
directly and depends on six Harmos crates. Every Harmos host operation is also served to
clients and plugins unless `protocol/composition.toml` narrows it. The owner locked one
boundary on 2026-11-16 so that Dokime owns product policy and its backend, and nothing else.

## What Changes

Owner decisions locked on 2026-11-16. Status: **proposed** until the owning tasks land;
**landed** only where stated.

| Decision | Direction |
| --- | --- |
| Storage only through Harmos operations | Dokime implements and registers its storage backend (landed: `crates/dokime-runtime/src/services/composition.rs`). Executions, cases, cycles, evidence, records, samples, documents, prompts, reports, content and files, watches, administration, queries and aggregates compose Harmos storage operations. Duplicated kernels are deleted. |
| Statistics in Harmos | Aggregation and statistics come from Harmos `list_rows` transforms. Floating-point RMS and standard deviation are accepted. |
| Product algorithms stay | Sigma, contour, grid and FFT, document and plot presentation, planning and seal policy stay in Dokime and source data through Harmos operations. Zoom coarsening stays a Dokime product operation that only calls Harmos operations. |
| Envelope stays stored | Decided by the owner on 2026-11-16: Dokime keeps its persisted zoom envelope, stored through Harmos object or file operations, with its merge kernel in Harmos. Sealing an execution clears its stored envelope and previews. |
| Idiomatic Harmos surface | **BREAKING.** Dokime builds the runtime through the Harmos builder, calls operations, implements its backend through the backend authoring module, and drops direct `harmos-domain`, `harmos-query`, `harmos-storage` and `harmos-protocol` dependencies. |
| Product wrappers | **BREAKING.** Harmos host operations become in-process only. Dokime declares its own wrapper operations for every Harmos host operation its clients use. |
| Composition removed | **BREAKING.** `protocol/composition.toml` and its readers are deleted; generators derive from the embedded descriptors and Dokime's own wrappers. |

The upstream Harmos change `operation-boundary` closes the operation gaps first. Dokime
adopts the revision that landed each Harmos group in dependency order; development storage
policy allows clean breaks: no migrations, no legacy compatibility, fresh roots when stored
shapes change, never silent deletion of existing roots.

Supersedes in `docs/architecture.md`: invariant 7's "Compose library services directly,
without redeclaring or forwarding them", invariant 1's composition reference, invariant 7's
"Product seals, artifact metadata and transaction ownership remain in the product Store",
and the Final Library Assembly and Storage Contract's manager construction and composition
file. It also supersedes the `storage-interface` change's statement that independent
backend writes are not one cross-method transaction (compound mutation is now an operation).

## Capabilities

### New Capabilities

- `storage-through-operations`: Dokime persistence reaches storage only through Harmos
  storage operations, with its backend as the only physical storage owner.
- `statistics-delegation`: aggregation through Harmos transforms, product algorithms sourcing
  data through operations, and the stored envelope lifecycle including clearing at seal.
- `library-dependency-fence`: Dokime depends on the Harmos facade and SDK only and builds
  its runtime through the Harmos builder.
- `remote-operation-surface`: Dokime's product wrappers for Harmos host operations and
  descriptor-derived SDK generation without a composition file.

### Modified Capabilities

None. Product query, report and plot behavior is preserved; `report-previews` and
`telemetry-query` keep their requirements on the new sources.

## Impact

`crates/dokime-storage` (reduces to the backend implementation and product persisted
shapes), `crates/dokime-operations`, `crates/dokime-runtime`, `crates/dokime-protocol`,
`crates/dokime-gui/src-tauri`, `crates/dokime-cli`, `proto/dokime/host/v1`, the four
product SDKs and their generators, `tools/python`, `Cargo.toml` Harmos dependencies, stories,
benchmarks, `docs/architecture.md`, `docs/execution-host.md` and the product guide.
