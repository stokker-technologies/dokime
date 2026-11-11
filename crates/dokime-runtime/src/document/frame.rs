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
use std::collections::HashMap;
use std::num::NonZeroU128;

// Third Party Imports
use harmos::Aggregate;
use harmos::storage::query::{
    Accumulator,
    scalar::{Agg, ExactNumber, ScalarValue},
};

// Local Imports
use super::DocumentFailure;
use super::binding::{Evidence, Row};
use super::context::Entry;
use super::input::RecordView;
use crate::expressions::{DataExpression, DataSource, DataTransform};
use dokime_domain::model::execution::EvidenceOrigin;
use dokime_domain::model::execution::RecordScope;
use dokime_domain::model::values::Value;

/* === Constants === */

const NANOS_PER_SECOND: u128 = 1_000_000_000;
const NONNUMERIC: &str = "numeric aggregate contains nonnumeric or nonfinite evidence";

/* === Macros === */

/* === Type Aliases === */

type Result<T> = crate::Result<T>;

/* === Structs === */

pub(super) struct Plan<'a> {
    left: Columns,
    right: Columns,
    join: Join<'a>,
    kind: Kind,
    shape: Shape,
    cases: Option<usize>,
    inner: Agg,
    measured: usize,
    outer: Option<Agg>,
    post: Vec<DataTransform>,
    inner_post: Vec<DataTransform>,
    correlation_maps: Vec<DataTransform>,
}

pub(crate) struct Fold {
    plan: Plan<'static>,
    values: Vec<Accumulator>,
    original: Option<(u64, Value)>,
    floating: Accumulator,
    has_float: bool,
}

struct Columns {
    values: [Column; 7],
    len: usize,
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

struct Selected<'a> {
    ordinal: usize,
    value: Option<Cow<'a, Value>>,
    keys: Option<Keys<'a>>,
    case: Option<usize>,
}

struct Keys<'a> {
    scope: Option<&'a RecordScope>,
    time: Option<i128>,
}

// Previous-Record Correlation: Right Rows Grouped By (Execution, Attempt) Keys;
// A Left Row Takes The Latest Right Row Captured Before It.
struct Correlation {
    rows: Vec<Projected>,
}

pub(crate) struct Lookup<'a> {
    rows: Vec<Side<'a>>,
    strategy: Strategy,
}

#[derive(Clone, Copy)]
pub(crate) struct Side<'a> {
    pub ordinal: usize,
    pub scope: Option<&'a RecordScope>,
    pub time: Option<i128>,
    pub present: bool,
}

/* === Enums === */

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Ordinal,
    Presence,
    Numeric,
    Occurrence,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Join<'a> {
    None,
    Previous(&'a str),
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
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Strategy {
    Latest,
    Previous,
}

/* === Traits === */

/* === Implementations === */

impl Columns {
    fn new() -> Self {
        Self { values: [Column::Value; 7], len: 0 }
    }
    fn push(&mut self, value: Column) {
        *self.values.get_mut(self.len).expect("a plan has at most one slot per column variant") = value;
        self.len += 1;
    }
    fn extend(&mut self, values: impl IntoIterator<Item = Column>) {
        for value in values {
            self.push(value);
        }
    }
}

impl std::ops::Deref for Columns {
    type Target = [Column];
    fn deref(&self) -> &[Column] {
        &self.values[..self.len]
    }
}

impl<'plan> Plan<'plan> {
    pub(super) fn compile(expression: &'plan DataExpression, cases: Option<usize>) -> Result<Self> {
        let transforms = &expression.transforms;
        let split = transforms.iter().position(|transform| matches!(transform, DataTransform::PerCase));
        let inner = &transforms[..split.unwrap_or(transforms.len())];
        let suffix = split.map(|index| &transforms[index + 1..]).unwrap_or_default();
        let outer_index = suffix.iter().rposition(|transform| fold_of(transform).is_some());
        let outer = outer_index.map(|index| agg(&suffix[index]));
        let fold_index = inner.iter().rposition(|transform| fold_of(transform).is_some());
        let fold = fold_index.and_then(|index| fold_of(&inner[index]));
        let selector = &inner[..fold_index.unwrap_or(inner.len())];
        let inner_maps = fold_index.map(|index| &inner[index + 1..]).unwrap_or_default();
        let post =
            if split.is_some() { outer_index.map(|index| suffix[index + 1..].to_vec()).unwrap_or_default() } else { inner_maps.to_vec() };
        let mut inner_post = if split.is_some() { inner_maps.to_vec() } else { Vec::new() };
        if let Some(index) = outer_index {
            inner_post.extend_from_slice(&suffix[..index]);
        }
        let join = match (&expression.source, selector.first()) {
            (DataSource::Record { .. }, Some(DataTransform::TimeBetween { previous_record_type })) => {
                Join::Previous(previous_record_type.as_str())
            }
            _ => Join::None,
        };
        let occurrence = matches!(expression.source, DataSource::Record { .. } | DataSource::Artifact { .. })
            && selector.is_empty()
            && fold == Some(Agg::Of(Aggregate::Count));
        let kind = match (fold, outer) {
            _ if occurrence => Kind::Occurrence,
            (Some(Agg::Of(Aggregate::Count)), _) | (None, Some(Agg::Of(Aggregate::Count))) => Kind::Presence,
            (Some(_), _) | (None, Some(_)) => Kind::Numeric,
            (None, None) => Kind::Ordinal,
        };
        if cases.is_some() != split.is_some() {
            return Err(failure("per-case bindings require the original execution context"));
        }
        // One Slot Per Column Variant Is A Structural Upper Bound.
        let mut left = Columns::new();
        let mut right = Columns::new();
        let elapsed = matches!(join, Join::Previous(_));
        let correlation_maps =
            if elapsed { selector.iter().filter(|transform| transform.is_numeric()).cloned().collect::<Vec<_>>() } else { Vec::new() };

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
            Some(_) => Some(outer.ok_or_else(|| failure("per-case scalar summaries require one final outer fold"))?),
            None => None,
        };
        let shape = match (cases.is_some(), outer, kind) {
            (true, Some(Agg::Of(Aggregate::Count)), _) | (false, _, Kind::Presence | Kind::Occurrence) => Shape::Count,
            (true, _, Kind::Presence | Kind::Occurrence) => Shape::Numeric { flags: false },
            _ if elapsed && correlation_maps.is_empty() => Shape::Elapsed,
            _ if elapsed => Shape::Numeric { flags: false },
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
            post,
            inner_post,
            correlation_maps,
        })
    }
    pub(super) fn evaluate<'a>(
        &self, expression: &DataExpression, evidence: Evidence<'a>, cases: &[Entry<'a>], selected: impl Fn(Row<'a>) -> bool,
    ) -> Result<Option<Value>> {
        let cases_by_execution = cases
            .iter()
            .enumerate()
            .filter_map(|(index, case)| Some((case.scope.test_execution_id.as_ref()?, index)))
            .collect::<HashMap<_, _>>();
        let capacity = evidence.upper_rows();
        let mut left = Vec::with_capacity(capacity);
        let mut right = Vec::with_capacity(capacity);
        match &self.join {
            Join::None if self.cases.is_some() => {
                let by_execution = cases
                    .iter()
                    .enumerate()
                    .filter_map(|(index, entry)| Some((entry.scope.test_execution_id.as_ref()?.as_str(), index)))
                    .collect::<HashMap<_, _>>();
                let contextual = cases
                    .iter()
                    .map(|entry| {
                        matches!(expression.source, DataSource::Parameter { .. })
                            || (matches!(expression.source, DataSource::Dimension { .. }) && entry.value(&expression.source).is_some())
                    })
                    .collect::<Vec<_>>();
                let mut ordinal = 0;
                let mut push = |row: Row<'a>, case| {
                    if selected(row) {
                        let value = super::binding::row(row, expression, evidence);
                        left.push(Selected::new(ordinal, value, None, Some(case)));
                    }
                    ordinal += 1;
                };
                for (case, entry) in cases.iter().copied().enumerate() {
                    if contextual[case] {
                        push(Row::Context(entry), case);
                    }
                }
                let case_of = |scope: Option<&RecordScope>| {
                    let scope = scope?;
                    let case = *by_execution.get(scope.test_execution_id.as_ref()?.as_str())?;
                    cases[case].covers(Some(scope)).then_some(case)
                };
                match &expression.source {
                    DataSource::Parameter { .. } => {}
                    DataSource::Record { record_type } => {
                        for record in evidence.records.iter() {
                            if record.entry.record_type.as_str() != record_type
                                || evidence.selection.is_some_and(|selection| !selection.matches_scope(record.scope))
                            {
                                continue;
                            }
                            if let Some(case) = case_of(record.scope) {
                                push(Row::Record { record, context: Some(cases[case]) }, case);
                            }
                        }
                    }
                    DataSource::Artifact { artifact: filename } => {
                        for artifact in evidence.artifacts.iter() {
                            if artifact.filename != *filename
                                || evidence.selection.is_some_and(|selection| !selection.matches_scope(Some(&artifact.scope)))
                            {
                                continue;
                            }
                            if let Some(case) = case_of(Some(&artifact.scope)) {
                                push(Row::Artifact { artifact, context: Some(cases[case]) }, case);
                            }
                        }
                    }
                    _ => {
                        for sample in evidence.telemetry {
                            if !evidence.selected(sample) {
                                continue;
                            }
                            if let Some(case) = case_of(sample.scope.as_ref())
                                && !contextual[case]
                            {
                                push(Row::Sample { sample, context: Some(cases[case]) }, case);
                            }
                        }
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
            Join::Previous(_) => {
                let DataSource::Record { record_type } = &expression.source else {
                    return Err(failure("record correlations require a record source"));
                };
                for (ordinal, record) in evidence.records.iter().enumerate() {
                    let case = record
                        .scope
                        .and_then(|scope| scope.test_execution_id.as_ref())
                        .and_then(|id| cases_by_execution.get(id))
                        .copied()
                        .filter(|&case| cases[case].covers(record.scope));
                    if self.cases.is_some() && case.is_none() {
                        continue;
                    }
                    let row = Row::Record {
                        record,
                        context: case.map(|case| cases[case]).or_else(|| evidence.context.for_scope(record.scope)),
                    };
                    if record.entry.record_type.as_str() == record_type
                        && evidence.selection.is_none_or(|selection| selection.matches_scope(record.scope))
                        && (self.cases.is_none() || case.is_some())
                        && selected(row)
                    {
                        left.push(Selected::new(ordinal, None, Some(Keys::record(record)), case));
                    }
                }
                self.right_rows(evidence, &mut right);
            }
        }
        let collected = self.collect(left, right)?;
        Ok(match self.output(&collected)? {
            Output::Value(value) => value,
            Output::Left(ordinal) => ordinal.and_then(|ordinal| {
                let row = evidence.rows(expression).nth(ordinal as usize)?;
                super::binding::row(row, expression, evidence).map(Cow::into_owned)
            }),
        })
    }

    pub(super) fn correlate<'a>(&self, record: RecordView<'a>, evidence: Evidence<'a>) -> Result<Option<Cow<'a, Value>>> {
        // Elapsed Time Pairs By Capture Order Inside The Window.
        let Some(ordinal) = evidence.records.iter().position(|candidate| std::ptr::eq(candidate.entry, record.entry)) else {
            return Ok(None);
        };
        let left = vec![Selected::new(ordinal, None, Some(Keys::record(record)), None)];
        let mut right = Vec::with_capacity(evidence.upper_rows());
        self.right_rows(evidence, &mut right);
        let collected = self.collect(left, right)?;
        Ok(match self.output(&collected)? {
            Output::Value(value) => value.map(Cow::Owned),
            Output::Left(_) => None,
        })
    }

    fn right_rows<'a>(&self, evidence: Evidence<'a>, right: &mut Vec<Selected<'a>>) {
        let Join::Previous(previous) = self.join else {
            return;
        };
        for (ordinal, record) in evidence.records.iter().enumerate() {
            if record.entry.record_type.as_str() == previous
                && evidence.selection.is_none_or(|selection| selection.matches_scope(record.scope))
            {
                right.push(Selected::new(ordinal, None, Some(Keys::record(record)), None));
            }
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
                Column::Case => row.case.map_or(ScalarValue::Missing, |case| ScalarValue::Unsigned(case as u64)),
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
                Column::Time => row.keys.as_ref().and_then(|keys| keys.time).map_or(ScalarValue::Missing, ScalarValue::Datetime),
            };
            values.push(projected);
        }
        Ok(Projected { capture: row.ordinal as u64, values })
    }

    fn collect(&self, left: Vec<Selected<'_>>, right: Vec<Selected<'_>>) -> Result<Collected> {
        let floats = |rows: &[Selected<'_>]| {
            self.kind == Kind::Numeric && rows.iter().any(|row| matches!(row.value.as_deref(), Some(Value::Number(_))))
        };
        let (left_float, right_float) = (floats(&left), floats(&right));
        let project = |rows: &[Selected<'_>], columns: &[Column], float| -> Result<Vec<Projected>> {
            let mut projected = Vec::with_capacity(rows.len());
            for row in rows {
                projected.push(self.project(columns, row, float)?);
            }
            Ok(projected)
        };
        let mut left = project(&left, &self.left, left_float)?;
        let right = project(&right, &self.right, right_float)?;
        left.sort_unstable_by_key(|row| row.capture);

        // Merge Equal Scope Partitions In Capture Order, Then Fold In Original Order.
        let correlation = match &self.join {
            Join::None => None,
            Join::Previous(_) => {
                let correlation = Correlation::new(right);
                let matched = correlation.merge(&left);
                Some((correlation, matched))
            }
        };
        let case = self.left.iter().position(|column| *column == Column::Case);
        let groups = self.cases.unwrap_or(1);

        let mut folds = Vec::with_capacity(groups);
        for _ in 0..groups {
            let mut group = Vec::with_capacity(self.measured);
            for index in 0..self.measured {
                group.push(Accumulator::new(if index == 0 { self.inner } else { flag(self.inner) }));
            }
            folds.push(group);
        }
        for (index, row) in left.iter().enumerate() {
            let group = match case {
                Some(column) => match row.values[column] {
                    ScalarValue::Unsigned(case) if (case as usize) < groups => case as usize,
                    _ => continue,
                },
                None => 0,
            };
            let measured = match &correlation {
                None => row.values[..self.measured].to_vec(),
                Some((correlation, matched)) => match matched[index].map(|index| &correlation.rows[index]) {
                    Some(matched) => {
                        let value = elapsed(&matched.values[2], &row.values[2])?;
                        let mut values = if self.correlation_maps.is_empty() {
                            vec![value]
                        } else {
                            vec![seconds(value)?.as_ref().map(floating).transpose()?.unwrap_or(ScalarValue::Missing)]
                        };
                        map_scalars(&mut values, &self.correlation_maps)?;
                        values
                    }
                    None => vec![ScalarValue::Missing],
                },
            };
            for (accumulator, value) in folds[group].iter_mut().zip(&measured) {
                accumulator.push(row.capture, value).map_err(reduction)?;
            }
        }
        let mut groups = Vec::with_capacity(folds.len());
        for fold in folds {
            let mut values = Vec::with_capacity(fold.len());
            for accumulator in fold {
                values.push(accumulator.finish(&mut || Ok(())).map_err(reduction)?);
            }
            map_scalars(&mut values, &self.inner_post)?;
            groups.push(values);
        }
        let outer = match self.outer {
            None => None,
            Some(outer) => {
                let mut values = Vec::with_capacity(self.measured);

                for column in 0..self.measured {
                    let mut accumulator = Accumulator::new(if column == 0 { outer } else { Agg::Of(Aggregate::Max) });
                    for (position, group) in groups.iter().enumerate() {
                        accumulator.push(position as u64, &group[column]).map_err(reduction)?;
                    }
                    values.push(accumulator.finish(&mut || Ok(())).map_err(reduction)?);
                }
                Some(values)
            }
        };
        Ok(Collected { groups, outer })
    }

    fn output(&self, collected: &Collected) -> Result<Output> {
        let values = match self.cases {
            Some(_) => collected.outer.as_deref().ok_or_else(|| failure("document fold produced no outer value"))?,
            None => collected.groups.first().ok_or_else(|| failure("document fold produced no group"))?,
        };
        let first = values.first().cloned().unwrap_or(ScalarValue::Missing);
        if matches!(first, ScalarValue::Missing) {
            return Ok(Output::Value(None));
        }
        let output = match self.shape {
            Shape::Original => match first {
                ScalarValue::Unsigned(ordinal) => Output::Left(Some(ordinal)),
                ScalarValue::Null => Output::Value(Some(Value::Null)),
                _ => return Err(failure("bounded frame ordinal is invalid")),
            },
            Shape::Count => match first {
                ScalarValue::Unsigned(count) => Output::Value(Some(Value::Unsigned(count))),
                _ => return Err(failure("bounded frame count is invalid")),
            },
            Shape::Elapsed => Output::Value(seconds(first)?),
            Shape::Numeric { flags } => {
                let flag = |index: usize| flags && matches!(values.get(index), Some(ScalarValue::Bool(true)));
                Output::Value(
                    Numeric {
                        value: first,
                        signed: flag(1),
                        float: flag(2),
                    }
                    .value()?,
                )
            }
        };
        Ok(match output {
            Output::Value(Some(value)) if !self.post.is_empty() && !matches!(value, Value::Null) => {
                let mut number = dokime_domain::model::values::numeric_value(&value).ok_or_else(|| failure(NONNUMERIC))?;
                for transform in &self.post {
                    number = transform.map_numeric(number).ok_or_else(|| failure(NONNUMERIC))?;
                }
                Output::Value(Some(Value::Number(number)))
            }
            other => other,
        })
    }
}

impl Fold {
    pub(crate) fn new(expression: &DataExpression) -> Result<Self> {
        let per_case = expression.transforms.contains(&DataTransform::PerCase);
        let mut compiled = Plan::compile(expression, per_case.then_some(1))?;
        if !matches!(compiled.join, Join::None) {
            // Record Queries Supply Correlated Scalar Seconds.
            compiled.left = Columns::new();
            compiled.left.push(Column::Value);
            if compiled.kind == Kind::Numeric {
                compiled.left.extend([Column::Signed, Column::Float]);
            }
            compiled.measured = compiled.left.len();
            if compiled.shape == Shape::Elapsed {
                compiled.shape = match compiled.kind {
                    Kind::Ordinal => Shape::Original,
                    _ => Shape::Numeric { flags: true },
                };
            }
        }
        let values =
            (0..compiled.measured).map(|index| Accumulator::new(if index == 0 { compiled.inner } else { flag(compiled.inner) })).collect();
        let plan = Plan {
            left: compiled.left,
            right: compiled.right,
            join: Join::None,
            kind: compiled.kind,
            shape: compiled.shape,
            cases: compiled.cases,
            inner: compiled.inner,
            measured: compiled.measured,
            outer: compiled.outer,
            post: compiled.post,
            inner_post: compiled.inner_post,
            correlation_maps: compiled.correlation_maps,
        };
        let floating = Accumulator::new(plan.inner);
        Ok(Self {
            plan,
            values,
            original: None,
            floating,
            has_float: false,
        })
    }

    pub(crate) fn push_at(&mut self, capture: u64, value: Option<&Value>) -> Result<()> {
        let row = Selected::new(capture as usize, value.map(Cow::Borrowed), None, self.plan.cases.map(|_| 0));
        let float = matches!(value, Some(Value::Number(_)));
        let mut projected = self.plan.project(&self.plan.left, &row, float)?;
        if self.plan.kind == Kind::Numeric {
            let scalar = match value {
                Some(Value::Null) => ScalarValue::Null,
                Some(value) => floating(value)?,
                None => ScalarValue::Missing,
            };
            self.floating.push(capture, &scalar).map_err(reduction)?;
            self.has_float |= float;
            if float {
                projected.values[0] = ScalarValue::Missing;
            }
        }
        for (accumulator, value) in self.values.iter_mut().zip(&projected.values) {
            accumulator.push(capture, value).map_err(reduction)?;
        }
        if let Some(value) = value
            && self.original.as_ref().is_none_or(|(last, _)| capture > *last)
        {
            self.original = Some((capture, value.clone()));
        }
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn finish(self) -> Result<Option<Value>> {
        self.finish_checked(&mut || Ok(()))
    }

    pub(crate) fn finish_checked(self, check: &mut impl FnMut() -> harmos::storage::Result<()>) -> Result<Option<Value>> {
        if self.plan.shape == Shape::Original {
            return Ok(self.original.map(|(_, value)| value));
        }
        let mut values =
            self.values.into_iter().map(|accumulator| accumulator.finish(check).map_err(reduction)).collect::<Result<Vec<_>>>()?;
        if self.has_float {
            values[0] = self.floating.finish(check).map_err(reduction)?;
        }
        map_scalars(&mut values, &self.plan.inner_post)?;
        let outer = self
            .plan
            .outer
            .map(|outer| {
                values
                    .iter()
                    .enumerate()
                    .map(|(column, value)| {
                        let mut accumulator = Accumulator::new(if column == 0 { outer } else { Agg::Of(Aggregate::Max) });
                        accumulator.push(0, value).map_err(reduction)?;
                        accumulator.finish(check).map_err(reduction)
                    })
                    .collect::<Result<Vec<_>>>()
            })
            .transpose()?;
        match self.plan.output(&Collected { groups: vec![values], outer })? {
            Output::Value(value) => Ok(value),
            Output::Left(_) => unreachable!("ordinal folds finish directly"),
        }
    }
}

impl Numeric {
    fn value(self) -> Result<Option<Value>> {
        let exact = match self.value {
            ScalarValue::Null => return Ok(Some(Value::Null)),
            ScalarValue::Missing => return Ok(None),
            ScalarValue::Unsigned(value) => ExactNumber::new(false, u128::from(value), NonZeroU128::MIN),
            ScalarValue::Signed(value) => ExactNumber::new(value < 0, u128::from(value.unsigned_abs()), NonZeroU128::MIN),
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
            let magnitude = i128::try_from(exact.numerator).map_err(|_| failure("aggregate exceeds signed integer range"))?;
            let value = if exact.negative { -magnitude } else { magnitude };
            if let Ok(value) = i64::try_from(value) {
                return Ok(Some(Value::Integer(value)));
            }
        }
        Ok(Some(Value::Unsigned(u64::try_from(exact.numerator).map_err(|_| failure("aggregate exceeds unsigned integer range"))?)))
    }
}

impl<'a> Selected<'a> {
    fn new(ordinal: usize, value: Option<Cow<'a, Value>>, keys: Option<Keys<'a>>, case: Option<usize>) -> Self {
        Self { ordinal, value, keys, case }
    }
}

impl<'a> Keys<'a> {
    fn record(record: RecordView<'a>) -> Self {
        Self {
            scope: record.scope,
            time: nanos(&record.entry.timestamp),
        }
    }
}

impl Correlation {
    fn new(mut rows: Vec<Projected>) -> Self {
        rows.retain(|row| matches!(row.values[2], ScalarValue::Datetime(_)));
        rows.sort_unstable_by(order);
        Self { rows }
    }

    // The Sorted Merge Advances Through Each Side Once.
    fn merge(&self, left: &[Projected]) -> Vec<Option<usize>> {
        let mut probes = left.iter().enumerate().collect::<Vec<_>>();
        probes.sort_unstable_by(|(_, left), (_, right)| order(left, right));
        let mut matches = vec![None; left.len()];
        let mut next = 0;
        let mut previous = None;
        for (index, probe) in probes {
            while let Some(candidate) = self.rows.get(next) {
                if !order(candidate, probe).is_lt() {
                    break;
                }
                previous = Some(next);
                next += 1;
            }
            matches[index] = previous.filter(|&index| {
                let candidate = &self.rows[index];
                text(&candidate.values[0]) == text(&probe.values[0]) && text(&candidate.values[1]) == text(&probe.values[1])
            });
        }
        matches
    }
}

impl<'a> Lookup<'a> {
    pub(crate) fn try_new<I>(right: I, strategy: Strategy) -> std::result::Result<Self, harmos::Error>
    where
        I: IntoIterator<Item = Side<'a>>,
        I::IntoIter: Clone,
    {
        let right = right.into_iter().filter(|side| side.time.is_some() && (strategy != Strategy::Latest || side.present));
        let mut rows = Vec::new();
        rows.try_reserve_exact(right.clone().count()).map_err(|_| harmos::Error::ResourceExhausted { resource: "correlation lookup" })?;
        rows.extend(right);
        rows.sort_unstable_by(|left, right| {
            lookup_keys(left).cmp(&lookup_keys(right)).then_with(|| match strategy {
                Strategy::Latest => left.time.cmp(&right.time).then(left.ordinal.cmp(&right.ordinal)),
                Strategy::Previous => left.ordinal.cmp(&right.ordinal),
            })
        });
        Ok(Self { rows, strategy })
    }

    pub(crate) fn find(&self, probe: &Side<'_>) -> Option<usize> {
        let time = probe.time?;
        let keys = lookup_keys(probe);
        let end = self.rows.partition_point(|row| match lookup_keys(row).cmp(&keys) {
            Ordering::Equal => match self.strategy {
                Strategy::Latest => row.time.is_some_and(|at| at <= time),
                Strategy::Previous => row.ordinal < probe.ordinal,
            },
            order => order.is_lt(),
        });
        let candidate = self.rows[..end].last()?;
        (lookup_keys(candidate) == keys && candidate.present).then_some(candidate.ordinal)
    }
}

/* === Public Functions === */

pub(crate) fn nanos(timestamp: &dokime_domain::model::time::Timestamp) -> Option<i128> {
    timestamp.to_datetime().ok().map(|time| time.unix_timestamp_nanos())
}

/* === Private Functions === */

fn lookup_keys<'a>(side: &Side<'a>) -> (&'a str, &'a str) {
    let execution = side.scope.and_then(|scope| scope.test_execution_id.as_ref()).map_or("", |id| id.as_str());
    let attempt = match side.scope.map(|scope| &scope.origin) {
        Some(EvidenceOrigin::Step { attempt_id }) => attempt_id.as_str(),
        _ => "",
    };
    (execution, attempt)
}

fn map_scalars(values: &mut [ScalarValue], transforms: &[DataTransform]) -> Result<()> {
    if transforms.is_empty() {
        return Ok(());
    }
    let Some(first) = values.first() else { return Ok(()) };
    let mut value = match first {
        ScalarValue::Null | ScalarValue::Missing => return Ok(()),
        ScalarValue::Unsigned(value) => *value as f64,
        ScalarValue::Signed(value) => *value as f64,
        ScalarValue::Float(value) => *value,
        ScalarValue::Exact(value) => float(*value),
        _ => return Err(failure(NONNUMERIC)),
    };
    for transform in transforms {
        value = transform.map_numeric(value).ok_or_else(|| failure(NONNUMERIC))?;
    }
    values[0] = ScalarValue::Float(value);
    if values.len() >= 3 {
        values[1] = ScalarValue::Bool(false);
        values[2] = ScalarValue::Bool(true);
    }
    Ok(())
}

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

fn order(left: &Projected, right: &Projected) -> Ordering {
    text(&left.values[0])
        .cmp(text(&right.values[0]))
        .then_with(|| text(&left.values[1]).cmp(text(&right.values[1])))
        .then_with(|| left.capture.cmp(&right.capture))
}

fn text(value: &ScalarValue) -> &str {
    match value {
        ScalarValue::Text(text) => text,
        _ => "",
    }
}

fn elapsed(start: &ScalarValue, end: &ScalarValue) -> Result<ScalarValue> {
    let (ScalarValue::Datetime(start), ScalarValue::Datetime(end)) = (start, end) else {
        return Err(failure("elapsed requires datetimes"));
    };
    if end < start {
        return Ok(ScalarValue::Missing);
    }
    let nanos = end.checked_sub(*start).ok_or_else(|| failure("elapsed time overflow"))? as u128;
    Ok(match u64::try_from(nanos) {
        Ok(nanos) => ScalarValue::Unsigned(nanos),
        Err(_) => ScalarValue::Exact(ExactNumber::new(false, nanos, NonZeroU128::MIN)),
    })
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
        Value::Integer(value) => ExactNumber::new(*value < 0, u128::from(value.unsigned_abs()), one),
        Value::Unsigned(value) => ExactNumber::new(false, u128::from(*value), one),
        _ => return Err(failure(NONNUMERIC)),
    }))
}

fn float(value: ExactNumber) -> f64 {
    let magnitude = value.numerator as f64 / value.denominator.get() as f64;
    if value.negative { -magnitude } else { magnitude }
}

fn seconds(value: ScalarValue) -> Result<Option<Value>> {
    let exact = match value {
        ScalarValue::Missing => return Ok(None),
        ScalarValue::Null => return Ok(Some(Value::Null)),
        ScalarValue::Unsigned(value) => ExactNumber::new(false, u128::from(value), NonZeroU128::MIN),
        ScalarValue::Exact(value) => value,
        _ => return Err(failure("elapsed time is invalid")),
    };
    if exact.negative {
        return Err(failure("elapsed time is invalid"));
    }
    let denominator = exact.denominator.get().saturating_mul(NANOS_PER_SECOND);
    if exact.numerator % denominator == 0 {
        return Ok(Some(Value::Unsigned(
            u64::try_from(exact.numerator / denominator).map_err(|_| failure("aggregate exceeds unsigned integer range"))?,
        )));
    }
    Ok(Some(Value::Number(float(exact) / NANOS_PER_SECOND as f64)))
}

fn failure(message: &'static str) -> crate::Error {
    DocumentFailure::Evaluation { message }.into()
}

fn reduction(error: harmos::storage::Error) -> crate::Error {
    match error {
        harmos::storage::Error::InvalidInput { reason, .. } => failure(reason),
        harmos::storage::Error::ResourceExhausted { .. } => failure("document fold exceeds its value bound"),
        _ => failure("document fold failed"),
    }
}

/* === Tests === */

#[cfg(test)]
mod streaming_tests {
    use super::*;

    #[test]
    fn numeric_maps_respect_aggregate_order() {
        let samples = [2.0, 6.0]
            .into_iter()
            .enumerate()
            .map(|(index, value)| {
                let mut sample = crate::telemetry::TelemetrySample::new(
                    dokime_domain::model::identity::TelemetrySampleId::from_seed(format!("map-{index}")),
                    dokime_domain::model::identity::Identifier::new("scan").unwrap(),
                    index as u64,
                    dokime_domain::model::time::Timestamp::new("2026-01-01T00:00:00Z"),
                    "scan",
                );
                sample.values.insert("value".into(), Value::Number(value));
                sample
            })
            .collect::<Vec<_>>();
        for expression in [
            crate::expressions::telemetry("scan", "value").scale(3.0).offset(2.0).mean().scale(2.0).offset(-1.0),
            crate::expressions::telemetry("scan", "value").mean().scale(6.0).offset(3.0),
        ] {
            let actual =
                Plan::compile(&expression, None).unwrap().evaluate(&expression, Evidence::from(samples.as_slice()), &[], |_| true).unwrap();
            assert_eq!(actual, Some(Value::Number(27.0)));
        }
    }

    #[test]
    fn streaming_statistics() {
        let values = [Value::Unsigned(9_007_199_254_740_993), Value::Unsigned(9_007_199_254_740_995)];
        for (aggregate, expected) in [
            (Aggregate::Min, Value::Unsigned(9_007_199_254_740_993)),
            (Aggregate::Max, Value::Unsigned(9_007_199_254_740_995)),
            (Aggregate::Mean, Value::Unsigned(9_007_199_254_740_994)),
            (Aggregate::Median, Value::Unsigned(9_007_199_254_740_994)),
        ] {
            let expression = crate::expressions::telemetry("fixture", "value").aggregate(aggregate);
            let mut fold = Fold::new(&expression).unwrap();
            for (position, value) in values.iter().enumerate() {
                fold.push_at(position as u64, Some(value)).unwrap();
            }
            assert_eq!(fold.finish().unwrap(), Some(expected));
        }
        let expression = crate::expressions::telemetry("fixture", "value").mean();
        let mut fold = Fold::new(&expression).unwrap();
        for (position, value) in [Value::Integer(-2), Value::Null, Value::Number(5.0), Value::Unsigned(3)].iter().enumerate() {
            fold.push_at(position as u64, Some(value)).unwrap();
        }
        assert_eq!(fold.finish().unwrap(), Some(Value::Number(2.0)));

        let mut per_case = crate::expressions::telemetry("fixture", "value").mean();
        per_case.transforms.extend([DataTransform::PerCase, DataTransform::Aggregate(Aggregate::Count)]);
        assert_eq!(Fold::new(&per_case).unwrap().finish().unwrap(), Some(Value::Unsigned(0)));
        let mut fold = Fold::new(&per_case).unwrap();
        fold.push_at(3, Some(&Value::Integer(7))).unwrap();
        fold.push_at(8, Some(&Value::Integer(9))).unwrap();
        assert_eq!(fold.finish().unwrap(), Some(Value::Unsigned(1)));

        let expression = crate::expressions::record("end").time_between("start").mean();
        let mut fold = Fold::new(&expression).unwrap();
        fold.push_at(2, Some(&Value::Number(0.5))).unwrap();
        fold.push_at(4, Some(&Value::Unsigned(1))).unwrap();
        assert_eq!(fold.finish().unwrap(), Some(Value::Number(0.75)));
    }

    #[test]
    fn sorted_record_merge_matches_capture_reference() {
        let row = |capture: u64| Projected {
            capture,
            values: vec![
                ScalarValue::Text(format!("execution.{}", capture % 3)),
                ScalarValue::Text(format!("attempt.{}", capture % 5)),
                ScalarValue::Datetime(i128::from(capture) * 1_000_000_000),
            ],
        };
        let right = (0..257).filter(|capture| capture % 4 != 0).rev().map(row).collect::<Vec<_>>();
        let probes = (0..307).rev().map(row).collect::<Vec<_>>();
        let expected = probes
            .iter()
            .map(|probe| {
                right
                    .iter()
                    .filter(|candidate| {
                        candidate.capture < probe.capture
                            && text(&candidate.values[0]) == text(&probe.values[0])
                            && text(&candidate.values[1]) == text(&probe.values[1])
                    })
                    .max_by_key(|candidate| candidate.capture)
                    .map(|candidate| candidate.capture)
            })
            .collect::<Vec<_>>();
        let correlation = Correlation::new(right);
        let actual =
            correlation.merge(&probes).into_iter().map(|index| index.map(|index| correlation.rows[index].capture)).collect::<Vec<_>>();
        assert_eq!(actual, expected);
    }

    #[test]
    fn partitioned_cases_match_individual_folds() {
        use std::collections::BTreeMap;
        let scopes = (0..7)
            .map(|index| {
                RecordScope::default()
                    .with_test_execution(dokime_domain::model::identity::TestExecutionId::from_seed(format!("execution-{index}")))
                    .with_test_case(dokime_domain::model::identity::CaseId::from_seed(format!("case-{index}")))
            })
            .collect::<Vec<_>>();
        let dimensions = BTreeMap::new();
        let parameters = BTreeMap::new();
        let cases = scopes
            .iter()
            .map(|scope| Entry {
                scope,
                dimensions: &dimensions,
                inputs: &parameters,
                outputs: &parameters,
            })
            .collect::<Vec<_>>();
        let samples = (0..137)
            .map(|index| {
                let mut sample = crate::telemetry::TelemetrySample::new(
                    dokime_domain::model::identity::TelemetrySampleId::from_seed(format!("sample-{index}")),
                    dokime_domain::model::identity::Identifier::new("fixture").unwrap(),
                    index,
                    dokime_domain::model::time::Timestamp::new("2026-01-01T00:00:00Z"),
                    "fixture",
                );
                sample.scope = Some(scopes[index as usize % (scopes.len() - 1)].clone());
                sample.values.insert("value".into(), Value::Integer(index as i64 - 37));
                sample
            })
            .collect::<Vec<_>>();
        let inner = crate::expressions::telemetry("fixture", "value").min();
        let mut expression = inner.clone();
        expression.transforms.extend([DataTransform::PerCase, DataTransform::Aggregate(Aggregate::Mean)]);
        let evidence = Evidence::from(samples.as_slice());
        let mut reference = Fold::new(&crate::expressions::telemetry("fixture", "value").mean()).unwrap();
        for (index, case) in cases.iter().enumerate() {
            let selected = samples.iter().filter(|sample| case.covers(sample.scope.as_ref())).cloned().collect::<Vec<_>>();
            let value = Plan::compile(&inner, None).unwrap().evaluate(&inner, Evidence::from(selected.as_slice()), &[], |_| true).unwrap();
            reference.push_at(index as u64, value.as_ref()).unwrap();
        }
        let actual = Plan::compile(&expression, Some(cases.len())).unwrap().evaluate(&expression, evidence, &cases, |_| true).unwrap();
        assert_eq!(actual, reference.finish().unwrap());
    }
}
