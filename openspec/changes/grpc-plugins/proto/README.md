# Wire Format Draft

Status: draft for owner review on 2026-11-10; not agreed and not implemented.

| File | Contents |
| --- | --- |
| `dokime/v1/execution.proto` | The test case, the operations a plugin serves for an execution, and the host operations a plugin calls through `ctx` |
| `dokime/v1/plugins.proto` | Plugin entries and the operations that add and remove a remote plugin |

The transport is `harmos.v1`, drafted in the Harmos change of the same name.

## What Is Not Drafted by Hand

About 130 operations already exist with request and response types defined in
Rust, and their `.proto` definitions are generated from those types. They
move into the `dokime.v1` package with their shapes unchanged. Before the
first release the generated files are checked in, a check fails when the Rust
types and the files differ, and from then on the compatibility rules apply to
them as well.

## Compatibility Rules

The same rules as `harmos.v1`:

- Changes are additive only. A field's number, name and type never change.
- A removed field or enum value is `reserved` and never reused.
- Both sides ignore fields and enum values they do not know.
- Every enum has an unspecified zero value.
- A change that cannot follow these rules is `dokime.v2`, served beside `v1`
  for one release.

## Decisions Taken in the Draft

Each is open to review.

| Decision | Reason |
| --- | --- |
| Parameters are `google.protobuf.Value` | They are authored as loosely typed values today |
| `StepRequest` carries only the execution and step identifiers | The suite instance already holds the test case from `initialize` |
| `InitializeRequest` carries the whole `TestCase` | The plugin needs the resolved parameters and nothing else arrives later |
| `CleanupRequest.termination` | Cleanup often differs after a failure or a cancel |
| `Step.repeatable` | Kept from today's phases for interactive executions |
| `StatisticsRequest.field` | A stream can have several columns; statistics are per column |
| `Records.records` is bounded by `limit` | A long execution can hold thousands of records |
| A record and a stream row are bytes of a message named in the plugin's catalog | The host stores them without knowing the plugin's types at build time |

## Settled in Review

- A step returns no values; `StepResponse` stays empty.
- A criterion returns pass or fail and nothing else.
- Statistics are per column, so the request names a field.
- Statistics cover streams only in the first version. A numeric series a
  criterion judges belongs in a stream; a record field can be added later
  as one new request field.
- Visibility has two levels, and the host owns the clock (see the Harmos
  draft).
