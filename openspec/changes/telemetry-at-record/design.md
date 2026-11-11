# Telemetry at Record Design

Status: proposed on 2026-11-09; semantics locked; deferred until scheduled.

## Semantics

For an expression `telemetry(S, F).at(R)`:

- Input A: the rows of stream `S` with field `F`, ordered by timestamp.
- Input B: the records of type `R`, ordered by timestamp.
- Without `within_s`: for each record `b`, the value is `F` of the last row
  `a` with `a.timestamp <= b.timestamp`; no row means no value for `b`.
- With `within_s(t)`: the row minimizing `|a.timestamp - b.timestamp|` among
  rows with `|a.timestamp - b.timestamp| <= t`; ties resolve to the earlier
  row; none within `t` means no value.
- Output: a series keyed by the record (its timestamp and identity), carrying
  the value; `detail(name)` applied after `at` reads the record, aggregates
  apply to the values, `per_case` groups by case as elsewhere.

## Evaluation

One merge over the two ordered inputs: advance A while its next timestamp is
at or before the current record, keep the last row (and, with a tolerance, the
first row after the record for the nearest decision). Memory is bounded by
one row plus the current record; the pass is linear in rows plus records and
fits the shared frozen evaluation used for sealed documents.

## Validation

`at` on a repeat selector (value, order or filter of a repeated section) is
refused with the same diagnostic shape as `time_between`. `at` requires a
telemetry source; applying it to a record, parameter, dimension or artifact
source is a validation error naming the expression.

## Wire

```json
{"type": "at", "record_type": "...", "within_s": 0.5}
```

`within_s` is optional and omitted when absent.

## Open Questions

- Whether `record(R).at(S, F)` should exist as the mirrored spelling; the
  proposal ships only the telemetry-first form.
- Whether a pairwise form over two record types (`distance_between`) is wanted
  once `at` exists, or whether `at` on position fields covers it.
