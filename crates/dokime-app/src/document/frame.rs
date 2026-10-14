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
use std::future::Future;
use std::num::{NonZeroU128, NonZeroUsize};
use std::task::{Context as Task, Poll, Waker};

// Third Party Imports
use harmos::Aggregate;
use harmos::storage::frame::{
    AsOf, AsOfStrategy, Collected, Expr, Frame, GroupBy, ProjectedRow, ScalarField, ScalarPath,
    ScanCursor, ScanOrder, ScanPage, ScanSource, collect,
};
use harmos::storage::reduce::{Agg, ExactNumber, ScalarKind, ScalarValue};
use harmos::storage::{ReadKind, ReadLimits, ReadSelection};

// Local Imports
use super::DocumentFailure;
use super::binding::{Evidence, Row};
use super::context::Entry;
use super::input::RecordView;
use crate::core::{EvidenceOrigin, RecordScope, Value};
use crate::expressions::{DataExpression, DataSource, DataTransform};
use crate::telemetry::TelemetrySample;

/* === Constants === */
const TOKEN_BYTES: usize = std::mem::size_of::<u64>();
const NANOS_PER_SECOND: u128 = 1_000_000_000;
const NONNUMERIC: &str = "numeric aggregate contains nonnumeric or nonfinite evidence";

/* === Macros === */

/* === Type Aliases === */
type Result<T> = dokime::Result<T>;

/* === Structs === */
pub(super) struct Plan {
    frame: Frame,
    left: Vec<Column>,
    right: Vec<Column>,
    join: Join,
    kind: Kind,
    shape: Shape,
    cases: Option<usize>,
}
struct Source {
    scans: Vec<Vec<ProjectedRow>>,
    sorted: Vec<(usize, ScanOrder, Vec<ProjectedRow>)>,
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
        rows: usize,
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

        let max_rows = NonZeroUsize::new(rows.max(1)).ok_or(DocumentFailure::Limit)?;
        let max_groups = NonZeroUsize::new(cases.unwrap_or(1).max(1))
            .filter(|groups| groups.get() <= 4096)
            .ok_or(DocumentFailure::Limit)?;
        let mut frame = Frame::new(max_rows, max_groups);
        let mut left = Vec::new();
        let mut right = Vec::new();
        let elapsed = matches!(join, Join::Previous(_));

        // The Left Scan Measures Its Own Value Unless A Correlation Supplies It.
        if join == Join::None {
            left.push(Column::Value);
            if kind == Kind::Numeric {
                left.extend([Column::Signed, Column::Float]);
            }
        } else {
            left.extend([Column::Execution, Column::Attempt, Column::Time]);
        }
        if cases.is_some() {
            left.push(Column::Case);
        }
        let left_scan = frame.scan(scan_selection(), fields(&left, kind)?);
        let mut measured = Vec::new();
        match &join {
            Join::None => {
                measured.push(Expr::Col(0));
                if kind == Kind::Numeric {
                    measured.extend([Expr::Col(1), Expr::Col(2)]);
                }
            }
            Join::Previous(_) => {
                right.extend([Column::Execution, Column::Attempt, Column::Time]);
                let right_scan = frame.scan(scan_selection(), fields(&right, kind)?);
                let join = frame.join_asof(AsOf {
                    left: left_scan,
                    right: right_scan,
                    by: vec![(0, 0), (1, 1)],
                    on: (2, 2),
                    strategy: AsOfStrategy::Previous,
                    right_filter: Vec::new(),
                });
                measured.push(Expr::diff(join));
            }
            Join::AtOrBefore { .. } => {
                right.push(Column::Value);
                if kind == Kind::Numeric {
                    right.extend([Column::Signed, Column::Float]);
                }
                right.extend([Column::Execution, Column::Attempt, Column::Time]);
                let keys = right.len() - 3;
                let right_scan = frame.scan(scan_selection(), fields(&right, kind)?);
                let join = frame.join_asof(AsOf {
                    left: left_scan,
                    right: right_scan,
                    by: vec![(0, keys), (1, keys + 1)],
                    on: (2, keys + 2),
                    strategy: AsOfStrategy::Backward { present: 0 },
                    right_filter: Vec::new(),
                });
                measured.extend(
                    (0..1 + 2 * usize::from(kind == Kind::Numeric))
                        .map(|col| Expr::AsOf { join, col }),
                );
            }
        }

        // Inner Folds Measure Each Case; Flags Keep Their Original Representation.
        let inner = match (kind, fold) {
            (Kind::Occurrence, _) => Agg::Len,
            (_, Some(fold)) => fold,
            (_, None) => Agg::Last,
        };
        let mut group_by = GroupBy::new(left_scan);
        for (index, expr) in measured.into_iter().enumerate() {
            group_by = group_by.agg(expr, if index == 0 { inner } else { flag(inner) });
        }
        if let Some(count) = cases {
            let case = left
                .iter()
                .position(|column| *column == Column::Case)
                .unwrap();
            group_by.keys = vec![case];
            group_by.groups = Some(
                (0..count as u64)
                    .map(|case| vec![ScalarValue::Unsigned(case)])
                    .collect(),
            );
            let outer = outer
                .ok_or_else(|| failure("per-case scalar summaries require one final outer fold"))?;
            group_by.outer = Some(
                (0..group_by.aggs.len())
                    .map(|index| {
                        if index == 0 {
                            outer
                        } else {
                            Agg::Of(Aggregate::Max)
                        }
                    })
                    .collect(),
            );
        }
        frame.group_by(group_by);
        frame.check().map_err(harmos_failure)?;
        let shape = match (cases.is_some(), outer, kind) {
            (true, Some(Agg::Of(Aggregate::Count)), _)
            | (false, _, Kind::Presence | Kind::Occurrence) => Shape::Count,
            (true, _, Kind::Presence | Kind::Occurrence) => Shape::Numeric { flags: false },
            _ if elapsed => Shape::Elapsed,
            (_, _, Kind::Ordinal) => Shape::Original,
            _ => Shape::Numeric { flags: true },
        };
        Ok(Self {
            frame,
            left,
            right,
            join,
            kind,
            shape,
            cases,
        })
    }

    pub(super) fn memory(&self, rows: usize) -> usize {
        let width = self.left.len().max(self.right.len());
        let copies = rows
            .saturating_mul(
                width
                    .saturating_mul(std::mem::size_of::<ScalarValue>())
                    .saturating_add(std::mem::size_of::<ProjectedRow>()),
            )
            // Each Scan Keeps Capture Order Plus One Keyed Copy.
            .saturating_mul(2 * (1 + usize::from(!self.right.is_empty())));
        self.frame
            .explain(page(rows, 0))
            .map_or(usize::MAX, |explain| explain.peak_bytes)
            .saturating_add(copies)
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

    fn project(
        &self,
        columns: &[Column],
        row: &Selected<'_>,
        float: bool,
        text: &mut usize,
    ) -> Result<ProjectedRow> {
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
                    *text = text.saturating_add(id.len());
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
                    *text = text.saturating_add(attempt.len());
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
        Ok(ProjectedRow {
            capture_order: row.ordinal as u64,
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
        let mut text = 0usize;
        let left = left
            .iter()
            .map(|row| self.project(&self.left, row, left_float, &mut text))
            .collect::<Result<Vec<_>>>()?;
        let right = right
            .iter()
            .map(|row| self.project(&self.right, row, right_float, &mut text))
            .collect::<Result<Vec<_>>>()?;
        let rows = left.len().max(right.len());
        let mut frame = self.frame.clone();
        frame.max_rows = NonZeroUsize::new(rows.max(frame.max_rows.get())).unwrap();
        for (scan, float) in [(0, left_float), (1, right_float)] {
            if let Some(field) = frame
                .scans
                .get_mut(scan)
                .and_then(|scan| scan.columns.first_mut())
                .filter(|_| float)
            {
                field.kind = ScalarKind::Float;
            }
        }
        Self::run(&frame, left, right, text)
    }

    fn run(
        frame: &Frame,
        left: Vec<ProjectedRow>,
        right: Vec<ProjectedRow>,
        text: usize,
    ) -> Result<Collected> {
        let page = page(left.len().max(right.len()), text);
        let explain = frame.explain(page).map_err(harmos_failure)?;
        let mut scans = vec![left];
        if frame.scans.len() > 1 {
            scans.push(right);
        }
        let mut source = Source {
            scans,
            sorted: Vec::new(),
        };
        let mut check = || Ok(());
        ready(collect(frame, &mut source, &explain, page, &mut check))
            .ok_or_else(|| failure("bounded frame evaluation did not complete"))?
            .map_err(harmos_failure)
    }

    fn output(&self, collected: &Collected) -> Result<Output> {
        let output = collected
            .outputs
            .first()
            .ok_or_else(|| failure("bounded frame produced no output"))?;
        let values = match self.cases {
            Some(_) => output
                .outer
                .as_deref()
                .ok_or_else(|| failure("bounded frame produced no outer fold"))?,
            None => {
                &output
                    .groups
                    .first()
                    .ok_or_else(|| failure("bounded frame produced no group"))?
                    .values
            }
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

impl ScanSource for Source {
    fn page(
        &mut self,
        scan: usize,
        order: &ScanOrder,
        cursor: Option<&ScanCursor>,
        limits: ReadLimits,
    ) -> impl Future<Output = harmos::Result<ScanPage>> + Send {
        std::future::ready(self.page_now(scan, order, cursor, limits))
    }
}

impl Source {
    fn page_now(
        &mut self,
        scan: usize,
        order: &ScanOrder,
        cursor: Option<&ScanCursor>,
        limits: ReadLimits,
    ) -> harmos::Result<ScanPage> {
        let offset = match cursor {
            None => 0,
            Some(cursor) => {
                u64::from_le_bytes(cursor.token.as_slice().try_into().map_err(|_| {
                    harmos::Error::InvalidInput {
                        field: "document frame",
                        reason: "foreign continuation",
                    }
                })?) as usize
            }
        };
        let rows = match order {
            ScanOrder::Capture => self.scans.get(scan).ok_or(harmos::Error::InvalidInput {
                field: "document frame",
                reason: "scan out of range",
            })?,
            ScanOrder::Keyed { .. } => {
                let at = match self
                    .sorted
                    .iter()
                    .position(|(sorted, keyed, _)| *sorted == scan && keyed == order)
                {
                    Some(at) => at,
                    None => {
                        let mut rows = self.scans.get(scan).cloned().unwrap_or_default();
                        let mut failure = None;
                        rows.sort_by(|a, b| {
                            order.compare(a, b).unwrap_or_else(|error| {
                                failure.get_or_insert(error);
                                std::cmp::Ordering::Equal
                            })
                        });
                        if let Some(error) = failure {
                            return Err(error);
                        }
                        self.sorted.push((scan, order.clone(), rows));
                        self.sorted.len() - 1
                    }
                };
                &self.sorted[at].2
            }
        };
        let mut page = Vec::new();
        let mut bytes = TOKEN_BYTES;
        let mut more = false;
        for row in rows.iter().skip(offset) {
            let text = row
                .values
                .iter()
                .map(|value| match value {
                    ScalarValue::Text(text) => text.capacity(),
                    _ => 0,
                })
                .sum::<usize>();
            if page.len() == limits.rows.get() || bytes.saturating_add(text) > limits.bytes.get() {
                more = true;
                break;
            }
            bytes += text;
            page.push(row.clone());
        }
        if page.is_empty() && more {
            return Err(harmos::Error::ResourceExhausted {
                resource: "document frame row text",
            });
        }
        let next = more.then(|| ScanCursor {
            token: ((offset + page.len()) as u64).to_le_bytes().to_vec(),
        });
        Ok(ScanPage { rows: page, next })
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

pub(crate) struct Lookup {
    frame: Frame,
    strategy: Strategy,
    order: ScanOrder,
    right: Vec<ProjectedRow>,
    text: usize,
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
        let mut text = 0usize;
        let rows = right
            .into_iter()
            .map(|side| {
                let mut values = vec![if side.present {
                    ScalarValue::Unsigned(side.ordinal as u64)
                } else {
                    ScalarValue::Missing
                }];
                values.extend(keys(&side, &mut text));
                ProjectedRow {
                    capture_order: side.ordinal as u64,
                    values,
                }
            })
            .collect::<Vec<_>>();
        // Left Probes Carry Their Index First So One Frame Answers A Batch.
        let columns = [
            Column::Value,
            Column::Execution,
            Column::Attempt,
            Column::Time,
        ];
        let mut frame = Frame::new(
            NonZeroUsize::new(rows.len().max(1)).unwrap(),
            NonZeroUsize::MIN,
        );
        let left = frame.scan(
            scan_selection(),
            fields(&columns, Kind::Ordinal).expect("static lookup fields"),
        );
        let right_scan = frame.scan(
            scan_selection(),
            fields(&columns, Kind::Ordinal).expect("static lookup fields"),
        );
        let join = frame.join_asof(AsOf {
            left,
            right: right_scan,
            by: vec![(1, 1), (2, 2)],
            on: (3, 3),
            strategy: match strategy {
                Strategy::AtOrBefore => AsOfStrategy::Backward { present: 0 },
                Strategy::Previous => AsOfStrategy::Previous,
            },
            right_filter: Vec::new(),
        });
        frame.group_by(
            GroupBy::new(left)
                .keys(vec![0])
                .agg(Expr::AsOf { join, col: 0 }, Agg::Last),
        );
        let order = ScanOrder::Keyed {
            by: vec![1, 2],
            time: (strategy == Strategy::AtOrBefore).then_some(3),
        };
        let mut right = rows;
        // Keys And Times Are Typed Per Column, So Ordering Cannot Fail.
        right.sort_by(|a, b| order.compare(a, b).unwrap_or(std::cmp::Ordering::Equal));
        Self {
            frame,
            strategy,
            order,
            right,
            text,
        }
    }

    pub(crate) fn find(&self, probe: &Side<'_>) -> Option<usize> {
        self.find_many(std::slice::from_ref(probe)).pop().flatten()
    }

    pub(crate) fn find_many(&self, probes: &[Side<'_>]) -> Vec<Option<usize>> {
        let mut found = vec![None; probes.len()];
        for (chunk, probes) in probes.chunks(4096).enumerate() {
            let base = chunk * 4096;
            if let Err(error) = self.find_chunk(base, probes, &mut found) {
                tracing::warn!(%error, "plot correlation frame was refused");
            }
        }
        found
    }

    fn find_chunk(
        &self,
        base: usize,
        probes: &[Side<'_>],
        found: &mut [Option<usize>],
    ) -> Result<()> {
        let mut text = self.text;
        let left = probes
            .iter()
            .enumerate()
            .map(|(index, probe)| {
                let mut values = vec![ScalarValue::Unsigned((base + index) as u64)];
                values.extend(keys(probe, &mut text));
                ProjectedRow {
                    // Previous Pairs By Capture Order; Sample Probes Only By Time.
                    capture_order: match self.strategy {
                        Strategy::Previous => probe.ordinal as u64,
                        Strategy::AtOrBefore => (base + index) as u64,
                    },
                    values,
                }
            })
            .collect::<Vec<_>>();
        let mut frame = self.frame.clone();
        frame.max_rows = NonZeroUsize::new(self.right.len().max(left.len()).max(1)).unwrap();
        frame.max_groups = NonZeroUsize::new(left.len().max(1)).unwrap();
        let page = page(self.right.len().max(left.len()), text);
        let explain = frame.explain(page).map_err(harmos_failure)?;
        let mut source = Source {
            scans: vec![left, Vec::new()],
            sorted: vec![(1, self.order.clone(), self.right.clone())],
        };
        let mut check = || Ok(());
        let collected = ready(collect(&frame, &mut source, &explain, page, &mut check))
            .ok_or_else(|| failure("bounded frame evaluation did not complete"))?
            .map_err(harmos_failure)?;
        let output = collected
            .outputs
            .first()
            .ok_or_else(|| failure("bounded frame produced no output"))?;
        for group in &output.groups {
            if let (Some(ScalarValue::Unsigned(probe)), Some(ScalarValue::Unsigned(right))) =
                (group.key.first(), group.values.first())
                && let Some(slot) = found.get_mut(*probe as usize)
            {
                *slot = Some(*right as usize);
            }
        }
        Ok(())
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

fn scan_selection() -> ReadSelection {
    ReadSelection::new(ReadKind::Records)
}

fn fields(columns: &[Column], kind: Kind) -> Result<Vec<ScalarField>> {
    columns
        .iter()
        .map(|column| {
            let (name, kind) = match column {
                Column::Value => (
                    "value",
                    match kind {
                        Kind::Numeric => ScalarKind::Exact,
                        Kind::Presence => ScalarKind::Bool,
                        Kind::Ordinal | Kind::Occurrence => ScalarKind::Unsigned,
                    },
                ),
                Column::Signed => ("signed", ScalarKind::Bool),
                Column::Float => ("float", ScalarKind::Bool),
                Column::Case => ("case", ScalarKind::Unsigned),
                Column::Execution => ("execution", ScalarKind::Text),
                Column::Attempt => ("attempt", ScalarKind::Text),
                Column::Time => ("time", ScalarKind::Datetime),
            };
            Ok(ScalarField {
                path: ScalarPath::Payload(vec![
                    harmos::Identifier::new(name).map_err(|_| DocumentFailure::Limit)?,
                ]),
                kind,
            })
        })
        .collect()
}

fn page(rows: usize, text: usize) -> ReadLimits {
    ReadLimits {
        rows: NonZeroUsize::new(rows.max(1)).unwrap(),
        bytes: NonZeroUsize::new(text.saturating_add(TOKEN_BYTES).saturating_add(64)).unwrap(),
    }
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

fn keys(side: &Side<'_>, text: &mut usize) -> [ScalarValue; 3] {
    let execution = side
        .scope
        .and_then(|scope| scope.test_execution_id.as_ref())
        .map_or("", |id| id.as_str());
    *text = text.saturating_add(execution.len());
    let execution = ScalarValue::Text(execution.to_owned());
    let attempt = match side.scope.map(|scope| &scope.origin) {
        Some(EvidenceOrigin::Step { attempt_id }) => attempt_id.as_str(),
        _ => "",
    };
    *text = text.saturating_add(attempt.len());
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

fn ready<T>(future: impl Future<Output = T>) -> Option<T> {
    let mut future = std::pin::pin!(future);
    match future.as_mut().poll(&mut Task::from_waker(Waker::noop())) {
        Poll::Ready(value) => Some(value),
        Poll::Pending => None,
    }
}

fn harmos_failure(error: harmos::Error) -> dokime::Error {
    match error {
        harmos::Error::ResourceExhausted { .. } => DocumentFailure::Limit.into(),
        harmos::Error::InvalidInput { reason, .. } => failure(reason),
        _ => failure("bounded frame evaluation failed"),
    }
}

fn failure(message: &'static str) -> dokime::Error {
    DocumentFailure::Evaluation { message }.into()
}

/* === Tests === */
