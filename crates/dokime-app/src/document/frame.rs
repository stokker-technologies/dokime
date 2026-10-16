/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::borrow::Cow;
use std::cmp::Ordering;
use std::num::NonZeroU128;

// Third Party Imports
use harmos::Aggregate;
use harmos::storage::query::scalar::{Agg, ExactNumber, ScalarValue};

// Local Imports
use super::DocumentFailure;
use super::binding::{Evidence, Row};
use super::context::Entry;
use super::fold::Accumulator;
use super::input::RecordView;
use crate::core::{EvidenceOrigin, RecordScope, Value};
use crate::expressions::{DataExpression, DataSource, DataTransform};
use crate::telemetry::TelemetrySample;

/* === Constants === */
const NANOS_PER_SECOND: u128 = 1_000_000_000;
const NONNUMERIC: &str = "numeric aggregate contains nonnumeric or nonfinite evidence";

/* === Macros === */

/* === Type Aliases === */
type Result<T> = dokime::Result<T>;

/* === Structs === */
/// One compiled data expression: which columns each side projects, how they
/// correlate and how the measured value folds, per case or overall.
pub(super) struct Plan {
    left: Vec<Column>,
    right: Vec<Column>,
    join: Join,
    kind: Kind,
    shape: Shape,
    cases: Option<usize>,
    inner: Agg,
    measured: usize,
    outer: Option<Agg>,
}

// One Projected Row: Its Capture Order And Typed Column Values.
#[derive(Clone)]
struct Projected {
    capture: u64,
    values: Vec<ScalarValue>,
}

// The Fold Of Every Group: Per-Group Values And The Optional Outer Fold.
struct Collected {
    groups: Vec<Vec<ScalarValue>>,
    outer: Option<Vec<ScalarValue>>,
}

struct Numeric {
    value: ScalarValue,
    signed: bool,
    float: bool,
}

/* === Enums === */
#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Ordinal,
    Presence,
    Numeric,
    Occurrence,
}
#[derive(Clone, PartialEq, Eq)]
enum Join {
    None,
    Previous(String),
    AtOrBefore { stream: String, field: String },
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Column {
    Value,
    Signed,
    Float,
    Case,
    Execution,
    Attempt,
    Time,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Shape {
    Original,
    Count,
    Numeric { flags: bool },
    Elapsed,
}
enum Output {
    Value(Option<Value>),
    Left(Option<u64>),
    Right(Option<u64>),
}

/* === Traits === */

/* === Implementations === */
impl Plan {
    pub(super) fn compile(
        expression: &DataExpression,
        _rows: usize,
        cases: Option<usize>,
    ) -> Result<Self> {
        let transforms = &expression.transforms;
        let split = transforms
            .iter()
            .position(|transform| matches!(transform, DataTransform::PerCase));
        let inner = &transforms[..split.unwrap_or(transforms.len())];
        let outer = split.and_then(|index| transforms.get(index + 1)).map(agg);
        let fold = inner.last().and_then(fold_of);
        let selector = &inner[..inner.len() - usize::from(fold.is_some())];
        let join = match (&expression.source, selector.first()) {
            (
                DataSource::Record { .. },
                Some(DataTransform::TimeBetween {
                    previous_record_type,
                }),
            ) => Join::Previous(previous_record_type.clone()),
            (
                DataSource::Record { .. },
                Some(DataTransform::TelemetryAtOrBefore { stream, field }),
            ) => Join::AtOrBefore {
                stream: stream.clone(),
                field: field.clone(),
            },
            _ => Join::None,
        };
        let occurrence = matches!(
            expression.source,
            DataSource::Record { .. } | DataSource::Artifact { .. }
        ) && selector.is_empty()
            && fold == Some(Agg::Of(Aggregate::Count));
        let kind = match (fold, outer) {
            _ if occurrence => Kind::Occurrence,
            (Some(Agg::Of(Aggregate::Count)), _) | (None, Some(Agg::Of(Aggregate::Count))) => {
                Kind::Presence
            }
            (Some(_), _) | (None, Some(_)) => Kind::Numeric,
            (None, None) => Kind::Ordinal,
        };
        if cases.is_some() != split.is_some() {
            return Err(failure(
                "per-case bindings require the original execution context",
            ));
        }
        if cases.is_some_and(|cases| cases > 4096) {
            return Err(DocumentFailure::Limit.into());
        }
        let mut left = Vec::new();
        let mut right = Vec::new();
        let elapsed = matches!(join, Join::Previous(_));

        // The Left Scan Measures Its Own Value Unless A Correlation Supplies It.
        let numeric = 1 + 2 * usize::from(kind == Kind::Numeric);
        let measured = match &join {
            Join::None => {
                left.push(Column::Value);
                if kind == Kind::Numeric {
                    left.extend([Column::Signed, Column::Float]);
                }
                numeric
            }
            Join::Previous(_) => {
                left.extend([Column::Execution, Column::Attempt, Column::Time]);
                right.extend([Column::Execution, Column::Attempt, Column::Time]);
                1
            }
            Join::AtOrBefore { .. } => {
                left.extend([Column::Execution, Column::Attempt, Column::Time]);
                right.push(Column::Value);
                if kind == Kind::Numeric {
                    right.extend([Column::Signed, Column::Float]);
                }
                right.extend([Column::Execution, Column::Attempt, Column::Time]);
                numeric
            }
        };
        if cases.is_some() {
            left.push(Column::Case);
        }
        // Inner Folds Measure Each Case; Flags Keep Their Original Representation.
        let inner = match (kind, fold) {
            (Kind::Occurrence, _) => Agg::Len,
            (_, Some(fold)) => fold,
            (_, None) => Agg::Last,
        };
        let outer = match cases {
            Some(_) => Some(outer.ok_or_else(|| {
                failure("per-case scalar summaries require one final outer fold")
            })?),
            None => None,
        };
        let shape = match (cases.is_some(), outer, kind) {
            (true, Some(Agg::Of(Aggregate::Count)), _)
            | (false, _, Kind::Presence | Kind::Occurrence) => Shape::Count,
            (true, _, Kind::Presence | Kind::Occurrence) => Shape::Numeric { flags: false },
            _ if elapsed => Shape::Elapsed,
            (_, _, Kind::Ordinal) => Shape::Original,
            _ => Shape::Numeric { flags: true },
        };
        Ok(Self {
            left,
            right,
            join,
            kind,
            shape,
            cases,
            inner,
            measured,
            outer,
        })
    }

    // Projected Rows Of Both Sides And One Sorted Copy Of The Correlated Side.
    pub(super) fn memory(&self, rows: usize) -> usize {
        let width = self.left.len().max(self.right.len());
        rows.saturating_mul(
            width
                .saturating_mul(std::mem::size_of::<ScalarValue>())
                .saturating_add(std::mem::size_of::<Projected>()),
        )
        .saturating_mul(2 * (1 + usize::from(!self.right.is_empty())))
    }

    pub(super) fn work(&self, rows: usize) -> usize {
        let scans = 1 + usize::from(!self.right.is_empty());
        let sort = if self.right.is_empty() {
            1
        } else {
            rows.max(2).ilog2() as usize + 1
        };
        rows.saturating_mul(scans).saturating_mul(sort)
    }

    pub(super) fn evaluate<'a>(
        &self,
        expression: &DataExpression,
        evidence: Evidence<'a>,
        cases: &[Entry<'a>],
        selected: impl Fn(Row<'a>) -> bool,
    ) -> Result<Option<Value>> {
        let mut left = Vec::new();
        let mut right = Vec::new();
        match &self.join {
            Join::None if self.cases.is_some() => {
                // A Case's Own Context Decides Whether Samples Or Context Supply It.
                let mut ordinal = 0usize;
                for (case, entry) in cases.iter().enumerate() {
                    for row in evidence.rows_for_case(expression, Some(*entry)) {
                        if selected(row) {
                            let value = super::binding::row(row, expression, evidence);
                            left.push(Selected::new(ordinal, value, None, Some(case)));
                        }
                        ordinal += 1;
                    }
                }
            }
            Join::None => {
                for (ordinal, row) in evidence.rows(expression).enumerate() {
                    if selected(row) {
                        let value = super::binding::row(row, expression, evidence);
                        left.push(Selected::new(ordinal, value, None, None));
                    }
                }
            }
            Join::Previous(_) | Join::AtOrBefore { .. } => {
                let DataSource::Record { record_type } = &expression.source else {
                    return Err(failure("record correlations require a record source"));
                };
                for (ordinal, record) in evidence.records.iter().enumerate() {
                    let case = case_of(cases, record.scope);
                    let row = Row::Record {
                        record,
                        context: case
                            .map(|case| cases[case])
                            .or_else(|| evidence.context.for_scope(record.scope)),
                    };
                    if record.entry.record_type.as_str() == record_type
                        && evidence
                            .selection
                            .is_none_or(|selection| selection.matches_scope(record.scope))
                        && (self.cases.is_none() || case.is_some())
                        && selected(row)
                    {
                        left.push(Selected::new(
                            ordinal,
                            None,
                            Some(Keys::record(record)),
                            case,
                        ));
                    }
                }
                self.right_rows(expression, evidence, &mut right);
            }
        }
        let collected = self.collect(left, right)?;
        Ok(match self.output(&collected)? {
            Output::Value(value) => value,
            Output::Left(ordinal) => ordinal.and_then(|ordinal| {
                let row = evidence.rows(expression).nth(ordinal as usize)?;
                super::binding::row(row, expression, evidence).map(Cow::into_owned)
            }),
            Output::Right(ordinal) => ordinal.and_then(|ordinal| {
                let Join::AtOrBefore { field, .. } = &self.join else {
                    return None;
                };
                let sample = evidence.telemetry.get(ordinal as usize)?;
                detail(telemetry_field(sample, field)?, &expression.transforms[1..])
                    .map(Cow::into_owned)
            }),
        })
    }

    pub(super) fn correlate<'a>(
        &self,
        record: RecordView<'a>,
        expression: &DataExpression,
        evidence: Evidence<'a>,
    ) -> Result<Option<Cow<'a, Value>>> {
        // Elapsed Time Pairs By Capture Order Inside The Window; A Sample Lookup
        // Needs Only The Record's Own Time And Scope.
        let ordinal = match evidence
            .records
            .iter()
            .position(|candidate| std::ptr::eq(candidate.entry, record.entry))
        {
            Some(ordinal) => ordinal,
            None if matches!(self.join, Join::AtOrBefore { .. }) => 0,
            None => return Ok(None),
        };
        let left = vec![Selected::new(
            ordinal,
            None,
            Some(Keys::record(record)),
            None,
        )];
        let mut right = Vec::new();
        self.right_rows(expression, evidence, &mut right);
        let collected = self.collect(left, right)?;
        Ok(match self.output(&collected)? {
            Output::Value(value) => value.map(Cow::Owned),
            Output::Right(ordinal) => ordinal.and_then(|ordinal| {
                let Join::AtOrBefore { field, .. } = &self.join else {
                    return None;
                };
                telemetry_field(evidence.telemetry.get(ordinal as usize)?, field)
            }),
            Output::Left(_) => None,
        })
    }

    fn right_rows<'a>(
        &self,
        expression: &DataExpression,
        evidence: Evidence<'a>,
        right: &mut Vec<Selected<'a>>,
    ) {
        match &self.join {
            Join::Previous(previous) => {
                for (ordinal, record) in evidence.records.iter().enumerate() {
                    if record.entry.record_type.as_str() == previous
                        && evidence
                            .selection
                            .is_none_or(|selection| selection.matches_scope(record.scope))
                    {
                        right.push(Selected::new(
                            ordinal,
                            None,
                            Some(Keys::record(record)),
                            None,
                        ));
                    }
                }
            }
            Join::AtOrBefore { stream, field } => {
                for (ordinal, sample) in evidence.telemetry.iter().enumerate() {
                    if evidence.selected(sample)
                        && super::plot_inputs::stream_matches_alias(sample.stream.as_str(), stream)
                    {
                        let value = telemetry_field(sample, field)
                            .and_then(|value| detail(value, &expression.transforms[1..]));
                        right.push(Selected::new(
                            ordinal,
                            value,
                            Some(Keys::sample(sample)),
                            None,
                        ));
                    }
                }
            }
            Join::None => {}
        }
    }

    fn project(&self, columns: &[Column], row: &Selected<'_>, float: bool) -> Result<Projected> {
        let value = row.value.as_deref();
        let mut values = Vec::with_capacity(columns.len());
        for column in columns {
            let projected = match column {
                Column::Value => match (self.kind, value) {
                    (Kind::Occurrence, _) => ScalarValue::Unsigned(row.ordinal as u64),
                    (_, None) => ScalarValue::Missing,
                    (Kind::Ordinal, Some(_)) => ScalarValue::Unsigned(row.ordinal as u64),
                    (_, Some(Value::Null)) => ScalarValue::Null,
                    (Kind::Presence, Some(_)) => ScalarValue::Bool(true),
                    (Kind::Numeric, Some(value)) if float => floating(value)?,
                    (Kind::Numeric, Some(value)) => exact(value)?,
                },
                Column::Signed | Column::Float => match value {
                    None => ScalarValue::Missing,
                    Some(Value::Null) => ScalarValue::Null,
                    Some(value) => ScalarValue::Bool(if *column == Column::Signed {
                        matches!(value, Value::Integer(_))
                    } else {
                        matches!(value, Value::Number(_))
                    }),
                },
                Column::Case => row.case.map_or(ScalarValue::Missing, |case| {
                    ScalarValue::Unsigned(case as u64)
                }),
                // Unowned Evidence Correlates Only With Other Unowned Evidence.
                Column::Execution => {
                    let id = row
                        .keys
                        .as_ref()
                        .and_then(|keys| keys.scope)
                        .and_then(|scope| scope.test_execution_id.as_ref())
                        .map_or("", |id| id.as_str());
                    ScalarValue::Text(id.to_owned())
                }
                Column::Attempt => {
                    let attempt = match row.keys.as_ref().and_then(|keys| keys.scope) {
                        Some(RecordScope {
                            origin: EvidenceOrigin::Step { attempt_id },
                            ..
                        }) => attempt_id.as_str(),
                        _ => "",
                    };
                    ScalarValue::Text(attempt.to_owned())
                }
                Column::Time => row
                    .keys
                    .as_ref()
                    .and_then(|keys| keys.time)
                    .map_or(ScalarValue::Missing, ScalarValue::Datetime),
            };
            values.push(projected);
        }
        Ok(Projected {
            capture: row.ordinal as u64,
            values,
        })
    }

    fn collect(&self, left: Vec<Selected<'_>>, right: Vec<Selected<'_>>) -> Result<Collected> {
        let floats = |rows: &[Selected<'_>]| {
            self.kind == Kind::Numeric
                && rows
                    .iter()
                    .any(|row| matches!(row.value.as_deref(), Some(Value::Number(_))))
        };
        let (left_float, right_float) = (floats(&left), floats(&right));
        let mut left = left
            .iter()
            .map(|row| self.project(&self.left, row, left_float))
            .collect::<Result<Vec<_>>>()?;
        let right = right
            .iter()
            .map(|row| self.project(&self.right, row, right_float))
            .collect::<Result<Vec<_>>>()?;
        left.sort_by_key(|row| row.capture);

        // Each Left Row Measures Its Own Columns Or Its Correlated Right Row.
        let correlation = match &self.join {
            Join::None => None,
            Join::Previous(_) => Some((Correlation::new(right, 0, Strategy::Previous), 0)),
            Join::AtOrBefore { .. } => {
                let keys = self.right.len() - 3;
                Some((Correlation::new(right, keys, Strategy::AtOrBefore), keys))
            }
        };
        let case = self.left.iter().position(|column| *column == Column::Case);
        let groups = self.cases.unwrap_or(1);
        let mut folds = (0..groups)
            .map(|_| {
                (0..self.measured)
                    .map(|index| {
                        Accumulator::new(if index == 0 {
                            self.inner
                        } else {
                            flag(self.inner)
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        for row in &left {
            let group = match case {
                Some(column) => match row.values[column] {
                    ScalarValue::Unsigned(case) if (case as usize) < groups => case as usize,
                    _ => continue,
                },
                None => 0,
            };
            let measured = match &correlation {
                None => row.values[..self.measured].to_vec(),
                Some((correlation, keys)) => {
                    let matched = correlation.find(&row.values[..3], row.capture);
                    match (&self.join, matched) {
                        (Join::Previous(_), Some(matched)) => {
                            vec![elapsed(&matched.values[2], &row.values[2])?]
                        }
                        (Join::Previous(_), None) => vec![ScalarValue::Missing],
                        (_, Some(matched)) => matched.values[..*keys].to_vec(),
                        (_, None) => vec![ScalarValue::Missing; *keys],
                    }
                }
            };
            for (accumulator, value) in folds[group].iter_mut().zip(&measured) {
                accumulator.push(row.capture, value).map_err(failure)?;
            }
        }
        let groups = folds
            .into_iter()
            .map(|fold| {
                fold.into_iter()
                    .map(|accumulator| accumulator.finish().map_err(failure))
                    .collect::<Result<Vec<_>>>()
            })
            .collect::<Result<Vec<_>>>()?;
        let outer = match self.outer {
            None => None,
            Some(outer) => {
                let mut values = Vec::with_capacity(self.measured);
                for column in 0..self.measured {
                    let mut accumulator = Accumulator::new(if column == 0 {
                        outer
                    } else {
                        Agg::Of(Aggregate::Max)
                    });
                    for (position, group) in groups.iter().enumerate() {
                        accumulator
                            .push(position as u64, &group[column])
                            .map_err(failure)?;
                    }
                    values.push(accumulator.finish().map_err(failure)?);
                }
                Some(values)
            }
        };
        Ok(Collected { groups, outer })
    }

    fn output(&self, collected: &Collected) -> Result<Output> {
        let values = match self.cases {
            Some(_) => collected
                .outer
                .as_deref()
                .ok_or_else(|| failure("document fold produced no outer value"))?,
            None => collected
                .groups
                .first()
                .ok_or_else(|| failure("document fold produced no group"))?,
        };
        let first = values.first().cloned().unwrap_or(ScalarValue::Missing);
        if matches!(first, ScalarValue::Missing) {
            return Ok(Output::Value(None));
        }
        Ok(match self.shape {
            Shape::Original => match (&self.join, first) {
                (Join::AtOrBefore { .. }, ScalarValue::Unsigned(ordinal)) => {
                    Output::Right(Some(ordinal))
                }
                (_, ScalarValue::Unsigned(ordinal)) => Output::Left(Some(ordinal)),
                (_, ScalarValue::Null) => Output::Value(Some(Value::Null)),
                _ => return Err(failure("bounded frame ordinal is invalid")),
            },
            Shape::Count => match first {
                ScalarValue::Unsigned(count) => Output::Value(Some(Value::Unsigned(count))),
                _ => return Err(failure("bounded frame count is invalid")),
            },
            Shape::Elapsed => Output::Value(seconds(first)?),
            Shape::Numeric { flags } => {
                let flag = |index: usize| {
                    flags && matches!(values.get(index), Some(ScalarValue::Bool(true)))
                };
                Output::Value(
                    Numeric {
                        value: first,
                        signed: flag(1),
                        float: flag(2),
                    }
                    .value()?,
                )
            }
        })
    }
}

impl Numeric {
    fn value(self) -> Result<Option<Value>> {
        let exact = match self.value {
            ScalarValue::Null => return Ok(Some(Value::Null)),
            ScalarValue::Missing => return Ok(None),
            ScalarValue::Unsigned(value) => {
                ExactNumber::new(false, u128::from(value), NonZeroU128::MIN)
            }
            ScalarValue::Signed(value) => ExactNumber::new(
                value < 0,
                u128::from(value.unsigned_abs()),
                NonZeroU128::MIN,
            ),
            ScalarValue::Exact(value) => value,
            ScalarValue::Float(value) if value.is_finite() => {
                return Ok(Some(Value::Number(value)));
            }
            _ => return Err(failure(NONNUMERIC)),
        };
        if self.float || exact.denominator.get() != 1 {
            let value = float(exact);
            if !value.is_finite() {
                return Err(failure("aggregate is outside the finite number range"));
            }
            return Ok(Some(Value::Number(value)));
        }
        if exact.negative || self.signed {
            let magnitude = i128::try_from(exact.numerator)
                .map_err(|_| failure("aggregate exceeds signed integer range"))?;
            let value = if exact.negative {
                -magnitude
            } else {
                magnitude
            };
            if let Ok(value) = i64::try_from(value) {
                return Ok(Some(Value::Integer(value)));
            }
        }
        Ok(Some(Value::Unsigned(
            u64::try_from(exact.numerator)
                .map_err(|_| failure("aggregate exceeds unsigned integer range"))?,
        )))
    }
}

struct Selected<'a> {
    ordinal: usize,
    value: Option<Cow<'a, Value>>,
    keys: Option<Keys<'a>>,
    case: Option<usize>,
}

impl<'a> Selected<'a> {
    fn new(
        ordinal: usize,
        value: Option<Cow<'a, Value>>,
        keys: Option<Keys<'a>>,
        case: Option<usize>,
    ) -> Self {
        Self {
            ordinal,
            value,
            keys,
            case,
        }
    }
}

struct Keys<'a> {
    scope: Option<&'a RecordScope>,
    time: Option<i128>,
}

impl<'a> Keys<'a> {
    fn record(record: RecordView<'a>) -> Self {
        Self {
            scope: record.scope,
            time: nanos(&record.entry.timestamp),
        }
    }
    fn sample(sample: &'a TelemetrySample) -> Self {
        Self {
            scope: sample.scope.as_ref(),
            time: nanos(&sample.timestamp),
        }
    }
}

// As-Of Correlation: Right Rows Grouped By (Execution, Attempt) Keys; A Left
// Row Takes The Latest Eligible Right Row Before It In Time Or Capture Order.
struct Correlation {
    rows: Vec<Projected>,
    keys: usize,
    strategy: Strategy,
}

impl Correlation {
    fn new(mut rows: Vec<Projected>, keys: usize, strategy: Strategy) -> Self {
        // Backward Matches Need A Present Value; Previous Matches Any Keyed Row.
        rows.retain(|row| {
            matches!(row.values[keys + 2], ScalarValue::Datetime(_))
                && !(strategy == Strategy::AtOrBefore
                    && matches!(row.values[0], ScalarValue::Missing))
        });
        rows.sort_by(|left, right| order(left, right, keys, strategy));
        Self {
            rows,
            keys,
            strategy,
        }
    }

    // The Last Right Row Not After The Probe Among Rows With Equal Keys.
    fn find(&self, probe: &[ScalarValue], capture: u64) -> Option<&Projected> {
        let ScalarValue::Datetime(time) = probe[2] else {
            return None;
        };
        let keys = self.keys;
        let precedes = |row: &Projected| {
            match text(&row.values[keys]).cmp(text(&probe[0])) {
                Ordering::Equal => {}
                order => return order.is_lt(),
            }
            match text(&row.values[keys + 1]).cmp(text(&probe[1])) {
                Ordering::Equal => {}
                order => return order.is_lt(),
            }
            match self.strategy {
                Strategy::AtOrBefore => {
                    matches!(row.values[keys + 2], ScalarValue::Datetime(at) if at <= time)
                }
                Strategy::Previous => row.capture < capture,
            }
        };
        let end = self.rows.partition_point(precedes);
        let candidate = self.rows[..end].last()?;
        (text(&candidate.values[keys]) == text(&probe[0])
            && text(&candidate.values[keys + 1]) == text(&probe[1]))
        .then_some(candidate)
    }
}

pub(crate) struct Lookup {
    correlation: Correlation,
}

pub(crate) struct Side<'a> {
    pub ordinal: usize,
    pub scope: Option<&'a RecordScope>,
    pub time: Option<i128>,
    pub present: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Strategy {
    AtOrBefore,
    Previous,
}

impl Lookup {
    pub(crate) fn new<'a>(right: impl IntoIterator<Item = Side<'a>>, strategy: Strategy) -> Self {
        let rows = right
            .into_iter()
            .map(|side| {
                let mut values = vec![if side.present {
                    ScalarValue::Unsigned(side.ordinal as u64)
                } else {
                    ScalarValue::Missing
                }];
                values.extend(keys(&side));
                Projected {
                    capture: side.ordinal as u64,
                    values,
                }
            })
            .collect::<Vec<_>>();
        Self {
            correlation: Correlation::new(rows, 1, strategy),
        }
    }

    pub(crate) fn find(&self, probe: &Side<'_>) -> Option<usize> {
        self.find_many(std::slice::from_ref(probe)).pop().flatten()
    }

    pub(crate) fn find_many(&self, probes: &[Side<'_>]) -> Vec<Option<usize>> {
        probes
            .iter()
            .enumerate()
            .map(|(index, probe)| {
                // Previous Pairs By Capture Order; Sample Probes Only By Time.
                let capture = match self.correlation.strategy {
                    Strategy::Previous => probe.ordinal as u64,
                    Strategy::AtOrBefore => index as u64,
                };
                match self.correlation.find(&keys(probe), capture)?.values[0] {
                    ScalarValue::Unsigned(ordinal) => Some(ordinal as usize),
                    _ => None,
                }
            })
            .collect()
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn fold_of(transform: &DataTransform) -> Option<Agg> {
    match transform {
        DataTransform::Aggregate(aggregate) => Some(Agg::Of(*aggregate)),
        _ => None,
    }
}

fn agg(transform: &DataTransform) -> Agg {
    fold_of(transform).unwrap_or(Agg::Last)
}

fn flag(agg: Agg) -> Agg {
    match agg {
        Agg::Last => Agg::Last,
        _ => Agg::Of(Aggregate::Max),
    }
}

fn order(left: &Projected, right: &Projected, keys: usize, strategy: Strategy) -> Ordering {
    text(&left.values[keys])
        .cmp(text(&right.values[keys]))
        .then_with(|| text(&left.values[keys + 1]).cmp(text(&right.values[keys + 1])))
        .then_with(|| match strategy {
            Strategy::AtOrBefore => datetime(&left.values[keys + 2])
                .cmp(&datetime(&right.values[keys + 2]))
                .then(left.capture.cmp(&right.capture)),
            Strategy::Previous => left.capture.cmp(&right.capture),
        })
}

fn text(value: &ScalarValue) -> &str {
    match value {
        ScalarValue::Text(text) => text,
        _ => "",
    }
}

fn datetime(value: &ScalarValue) -> Option<i128> {
    match value {
        ScalarValue::Datetime(at) => Some(*at),
        _ => None,
    }
}

fn elapsed(start: &ScalarValue, end: &ScalarValue) -> Result<ScalarValue> {
    let (ScalarValue::Datetime(start), ScalarValue::Datetime(end)) = (start, end) else {
        return Err(failure("elapsed requires datetimes"));
    };
    if end < start {
        return Ok(ScalarValue::Missing);
    }
    let nanos = end
        .checked_sub(*start)
        .ok_or_else(|| failure("elapsed time overflow"))? as u128;
    Ok(match u64::try_from(nanos) {
        Ok(nanos) => ScalarValue::Unsigned(nanos),
        Err(_) => ScalarValue::Exact(ExactNumber::new(false, nanos, NonZeroU128::MIN)),
    })
}

fn case_of(cases: &[Entry<'_>], scope: Option<&RecordScope>) -> Option<usize> {
    cases.iter().position(|case| case.covers(scope))
}

fn telemetry_field<'a>(sample: &'a TelemetrySample, field: &str) -> Option<Cow<'a, Value>> {
    if let Some(value) = sample.values.get(field) {
        Some(Cow::Borrowed(value))
    } else if field == "timestamp" {
        Some(Cow::Owned(Value::Datetime(
            sample.timestamp.to_datetime().ok()?,
        )))
    } else {
        sample
            .values
            .get(field.rsplit_once('.')?.1)
            .map(Cow::Borrowed)
    }
}

fn detail<'a>(mut value: Cow<'a, Value>, transforms: &[DataTransform]) -> Option<Cow<'a, Value>> {
    for transform in transforms {
        let DataTransform::Detail { name } = transform else {
            break;
        };
        value = match value {
            Cow::Borrowed(Value::Object(fields)) => Cow::Borrowed(fields.get(name)?),
            Cow::Owned(Value::Object(mut fields)) => Cow::Owned(fields.remove(name)?),
            _ => return None,
        };
    }
    Some(value)
}

pub(crate) fn nanos(timestamp: &crate::core::Timestamp) -> Option<i128> {
    timestamp
        .to_datetime()
        .ok()
        .map(|time| time.unix_timestamp_nanos())
}

fn keys(side: &Side<'_>) -> [ScalarValue; 3] {
    let execution = side
        .scope
        .and_then(|scope| scope.test_execution_id.as_ref())
        .map_or("", |id| id.as_str());
    let execution = ScalarValue::Text(execution.to_owned());
    let attempt = match side.scope.map(|scope| &scope.origin) {
        Some(EvidenceOrigin::Step { attempt_id }) => attempt_id.as_str(),
        _ => "",
    };
    [
        execution,
        ScalarValue::Text(attempt.to_owned()),
        side.time
            .map_or(ScalarValue::Missing, ScalarValue::Datetime),
    ]
}

fn floating(value: &Value) -> Result<ScalarValue> {
    Ok(ScalarValue::Float(match value {
        Value::Integer(value) => *value as f64,
        Value::Unsigned(value) => *value as f64,
        Value::Number(value) if value.is_finite() => *value,
        _ => return Err(failure(NONNUMERIC)),
    }))
}

fn exact(value: &Value) -> Result<ScalarValue> {
    let one = NonZeroU128::MIN;
    Ok(ScalarValue::Exact(match value {
        Value::Integer(value) => {
            ExactNumber::new(*value < 0, u128::from(value.unsigned_abs()), one)
        }
        Value::Unsigned(value) => ExactNumber::new(false, u128::from(*value), one),
        _ => return Err(failure(NONNUMERIC)),
    }))
}

fn float(value: ExactNumber) -> f64 {
    let magnitude = value.numerator as f64 / value.denominator.get() as f64;
    if value.negative {
        -magnitude
    } else {
        magnitude
    }
}

fn seconds(value: ScalarValue) -> Result<Option<Value>> {
    let exact = match value {
        ScalarValue::Missing => return Ok(None),
        ScalarValue::Null => return Ok(Some(Value::Null)),
        ScalarValue::Unsigned(value) => {
            ExactNumber::new(false, u128::from(value), NonZeroU128::MIN)
        }
        ScalarValue::Exact(value) => value,
        _ => return Err(failure("elapsed time is invalid")),
    };
    if exact.negative {
        return Err(failure("elapsed time is invalid"));
    }
    let denominator = exact.denominator.get().saturating_mul(NANOS_PER_SECOND);
    if exact.numerator % denominator == 0 {
        return Ok(Some(Value::Unsigned(
            u64::try_from(exact.numerator / denominator)
                .map_err(|_| failure("aggregate exceeds unsigned integer range"))?,
        )));
    }
    Ok(Some(Value::Number(float(exact) / NANOS_PER_SECOND as f64)))
}

fn failure(message: &'static str) -> dokime::Error {
    DocumentFailure::Evaluation { message }.into()
}

/* === Tests === */
