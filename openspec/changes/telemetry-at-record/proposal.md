# Telemetry at Record

Status: proposed on 2026-11-09; semantics locked by the owner on 2026-11-09;
deferred until the owner schedules it.

## Why

Documents can plot a stream (`telemetry(stream, field)`) and a record series
(`record(type)` with `time_between` and `detail`), but nothing relates the two.
The question "what was the tracking distance when each disconnection
happened?" cannot be asked: the value has to be copied into the record by the
plugin that publishes it, so every such question costs a plugin change and a
new record field, and historical executions never gain the answer.

## What Changes

- Add the `at` transform to data expressions:
  `telemetry(stream, field).at(record_type)` yields one value per record of
  that type, the stream row with the greatest timestamp at or before the
  record's timestamp. `.within_s(tolerance)` switches to the nearest row on
  either side within the tolerance. A record with no qualifying row yields no
  value.
- The result is a per-record series: histograms, aggregates, `detail` of the
  record and `per_case` compose with it like they do with `time_between`.
- Like `time_between`, `at` is row-local: a repeated section whose value,
  order or filter uses it is refused at validation, naming the section.
- Evaluation merges the two time-ordered inputs in one pass, so live and
  sealed evidence behave identically and the linear record evaluation
  requirement holds.
- Wire: one new `DataTransform` variant (`at`, with `record_type` and optional
  `within_s`), carried by the document definition like the others. Data shape
  versions derive; existing documents are unaffected.

## Impact

- Affected: `dokime-domain` expressions, the runtime document evaluator
  (`document/query.rs`, `frame.rs`, `binding.rs`), document validation, the
  authoring skill and product guide, the Python SDK's expression builders.
- Not affected: record publication, stream storage, plot rendering.
