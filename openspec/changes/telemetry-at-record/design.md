# Telemetry at Record Design

Status: scheduled on 2026-11-22; semantics locked.

## Delivery Decisions

The pinned library exposes `TimeMergeMode::AtOrBefore` but no nearest-within
mode. Ship the at-or-before binding; retain the tolerance wire field and builders
but reject its evaluation with an actionable diagnostic. Nearest-within tasks
and the tolerance scenario remain open until the library supplies that mode.

The existing storage expression evaluator owns the binding. Its record source
is the primary input of the library merge, preserving record position and time.
Existing plot filters evaluate on those same rows: `at(...).detail("event")`
reads that primary record, without a second filter concept. Histograms for
connected and disconnected values share the one status record type.

The worker leaves `docs/` unchanged. The requested architecture document is
absent from this checkout; authoring examples ship with the SDKs instead.

## Ownership Follow-Up

`packages/plots/src/expression.ts::validatedExpression` validates expression
shape for `crates/dokime-gui/src/plots/live-data.ts`. The protocol's
`src/model/expressions.rs::DataExpression::validate` validates transform inputs
and capabilities for document expansion and storage evaluation. Both recurse
through expression inputs and guard nesting. Keep both pending the owner's
decision; prefer removing the frontend semantic rules and using the host's
validation result, while retaining transport decoding at the frontend boundary.

In `crates/dokime-storage/src/expressions.rs`, `collect` adds a `time_between`
predecessor to the generic attachment loop in `Store::evaluate`, while
`collect_intervals` drives a second ordered pairing of the current and previous
record tables in the same evaluator. Both serve document values and projected
plots; interval-only expressions therefore reread and pair sources unnecessarily.
Keep this inherited overlap pending the owner's decision. Prefer removing the
generic attachment for dependencies used only by interval computation, retaining
the interval pairing and attachments actually needed for selected record fields.

The inherited `time_between` repeat rejection was specified but absent from the
current implementation. The document validator now refuses both row-local
transforms with the same section diagnostic. `SectionRepeat` has no filter
field; the check covers the repeated section's card filters as well as its
value and ordering. Plot filters remain row-aligned inside plot evaluation.

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

The product binds the expression to Harmos time-merge operations under
operation-boundary; it does not add a private merge kernel. Semantically, one
merge over the two ordered inputs: advance A while its next timestamp is
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
