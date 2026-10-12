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

// Third Party Imports
use serde::{Deserialize, Serialize};
// Local Imports
use crate::core::Value;
use crate::expressions::{Expression, ExpressionKind};

/* === Constants === */
const FN_DATA_TELEMETRY: &str = "data.telemetry";
const FN_DATA_PARAMETER: &str = "data.parameter";
const FN_DATA_DIMENSION: &str = "data.dimension";
const FN_DATA_RECORD: &str = "data.record";
const FN_DATA_ARTIFACT: &str = "data.artifact";
const FN_TRANSFORM_COUNT: &str = "data.transform.count";
const FN_TRANSFORM_MEAN: &str = "data.transform.mean";
const FN_TRANSFORM_MEDIAN: &str = "data.transform.median";
const FN_TRANSFORM_MIN: &str = "data.transform.min";
const FN_TRANSFORM_MAX: &str = "data.transform.max";
const FN_TRANSFORM_TIME_BETWEEN: &str = "data.transform.time_between";
const FN_TRANSFORM_TELEMETRY_AT_OR_BEFORE: &str = "data.transform.telemetry_at_or_before";
const FN_TRANSFORM_DETAIL: &str = "data.transform.detail";
const FN_TRANSFORM_PER_CASE: &str = "data.transform.per_case";

/* === Macros === */
product_error!(DataExpressionError, formatter {
    Self::UnsupportedExpression => formatter.write_str("expression cannot be represented as a data expression"),
});

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DataExpression {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub source: DataSource,
    #[serde(default)]
    pub transforms: Vec<DataTransform>,
}

/* === Enums === */
/* === Errors === */

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DataExpressionError {
    UnsupportedExpression,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DataSource {
    Telemetry { stream: String, field: String },
    Parameter { name: String },
    Dimension { name: String },
    Record { record_type: String },
    Artifact { artifact: String },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DataTransform {
    Count,
    Mean,
    Median,
    Min,
    Max,
    TimeBetween { previous_record_type: String },
    TelemetryAtOrBefore { stream: String, field: String },
    Detail { name: String },
    PerCase,
}

/* === Traits === */

/* === Implementations === */
impl harmos::MemorySize for DataSource {
    fn memory_size(&self) -> usize {
        let heap = match self {
            Self::Telemetry { stream, field } => stream.capacity().saturating_add(field.capacity()),
            Self::Parameter { name } | Self::Dimension { name } => name.capacity(),
            Self::Record { record_type } => record_type.capacity(),
            Self::Artifact { artifact } => artifact.capacity(),
        };
        std::mem::size_of::<Self>().saturating_add(heap)
    }
}

impl harmos::MemorySize for DataTransform {
    fn memory_size(&self) -> usize {
        let heap = match self {
            Self::Count | Self::Mean | Self::Median | Self::Min | Self::Max | Self::PerCase => 0,
            Self::TimeBetween {
                previous_record_type,
            } => previous_record_type.capacity(),
            Self::TelemetryAtOrBefore { stream, field } => {
                stream.capacity().saturating_add(field.capacity())
            }
            Self::Detail { name } => name.capacity(),
        };
        std::mem::size_of::<Self>().saturating_add(heap)
    }
}

impl harmos::MemorySize for DataExpression {
    fn memory_size(&self) -> usize {
        let fields = [
            (
                harmos::MemorySize::memory_size(&self.label),
                std::mem::size_of_val(&self.label),
            ),
            (
                harmos::MemorySize::memory_size(&self.source),
                std::mem::size_of_val(&self.source),
            ),
            (
                harmos::MemorySize::memory_size(&self.transforms),
                std::mem::size_of_val(&self.transforms),
            ),
        ];
        fields
            .into_iter()
            .fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
                if owned == usize::MAX {
                    usize::MAX
                } else {
                    bytes.saturating_add(owned.saturating_sub(inline))
                }
            })
    }
}

impl DataExpression {
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub(crate) fn axis_label(&self) -> &str {
        if let Some(label) = self
            .label
            .as_deref()
            .filter(|label| !label.trim().is_empty())
        {
            return label;
        }

        match &self.source {
            DataSource::Telemetry { field, .. } => field,
            DataSource::Parameter { name } | DataSource::Dimension { name } => name,
            DataSource::Record { record_type } => record_type,
            DataSource::Artifact { artifact } => artifact,
        }
    }

    pub(crate) fn to_expression(&self) -> Expression {
        self.clone().into_expression()
    }

    pub fn into_expression(self) -> Expression {
        let mut expression = data_source_expression(self.source);
        for transform in self.transforms {
            expression = data_transform_expression(transform, expression);
        }

        expression
    }

    pub fn try_from_expression(expression: &Expression) -> crate::Result<Self> {
        let (source, transforms) = parse_data_expression(expression)?;

        Ok(Self {
            label: None,
            source,
            transforms,
        })
    }

    pub fn count(mut self) -> Self {
        self.transforms.push(DataTransform::Count);
        self
    }

    pub fn mean(mut self) -> Self {
        self.transforms.push(DataTransform::Mean);
        self
    }

    pub fn median(mut self) -> Self {
        self.transforms.push(DataTransform::Median);
        self
    }

    pub fn min(mut self) -> Self {
        self.transforms.push(DataTransform::Min);
        self
    }

    pub fn max(mut self) -> Self {
        self.transforms.push(DataTransform::Max);
        self
    }

    pub fn time_between(mut self, previous_record_type: impl Into<String>) -> Self {
        self.transforms.push(DataTransform::TimeBetween {
            previous_record_type: previous_record_type.into(),
        });
        self
    }

    pub fn telemetry_at_or_before(
        mut self,
        stream: impl Into<String>,
        field: impl Into<String>,
    ) -> Self {
        self.transforms.push(DataTransform::TelemetryAtOrBefore {
            stream: stream.into(),
            field: field.into(),
        });
        self
    }

    pub fn detail(mut self, name: impl Into<String>) -> Self {
        self.transforms
            .push(DataTransform::Detail { name: name.into() });
        self
    }

    pub fn per_case(mut self) -> Self {
        self.transforms.push(DataTransform::PerCase);
        self
    }
}

impl From<DataExpression> for Expression {
    fn from(value: DataExpression) -> Self {
        value.into_expression()
    }
}

impl From<&DataExpression> for Expression {
    fn from(value: &DataExpression) -> Self {
        value.to_expression()
    }
}

impl TryFrom<Expression> for DataExpression {
    type Error = crate::Error;

    fn try_from(value: Expression) -> Result<Self, Self::Error> {
        Self::try_from_expression(&value)
    }
}

/* === Public Functions === */
pub fn telemetry(stream: impl Into<String>, field: impl Into<String>) -> DataExpression {
    DataExpression {
        label: None,
        source: DataSource::Telemetry {
            stream: stream.into(),
            field: field.into(),
        },
        transforms: Vec::new(),
    }
}

pub fn parameter(name: impl Into<String>) -> DataExpression {
    DataExpression {
        label: None,
        source: DataSource::Parameter { name: name.into() },
        transforms: Vec::new(),
    }
}

pub fn dimension(name: impl Into<String>) -> DataExpression {
    DataExpression {
        label: None,
        source: DataSource::Dimension { name: name.into() },
        transforms: Vec::new(),
    }
}

pub fn record(record_type: impl Into<String>) -> DataExpression {
    DataExpression {
        label: None,
        source: DataSource::Record {
            record_type: record_type.into(),
        },
        transforms: Vec::new(),
    }
}

pub fn artifact(artifact: impl Into<String>) -> DataExpression {
    DataExpression {
        label: None,
        source: DataSource::Artifact {
            artifact: artifact.into(),
        },
        transforms: Vec::new(),
    }
}

pub fn per_case(expression: impl Into<DataExpression>) -> DataExpression {
    expression.into().per_case()
}

/* === Private Functions === */
fn data_source_expression(source: DataSource) -> Expression {
    match source {
        DataSource::Telemetry { stream, field } => Expression::call(
            FN_DATA_TELEMETRY,
            vec![Expression::literal(stream), Expression::literal(field)],
        ),
        DataSource::Parameter { name } => {
            Expression::call(FN_DATA_PARAMETER, vec![Expression::literal(name)])
        }
        DataSource::Dimension { name } => {
            Expression::call(FN_DATA_DIMENSION, vec![Expression::literal(name)])
        }
        DataSource::Record { record_type } => {
            Expression::call(FN_DATA_RECORD, vec![Expression::literal(record_type)])
        }
        DataSource::Artifact { artifact } => {
            Expression::call(FN_DATA_ARTIFACT, vec![Expression::literal(artifact)])
        }
    }
}

fn data_transform_expression(transform: DataTransform, expression: Expression) -> Expression {
    match transform {
        DataTransform::Count => Expression::call(FN_TRANSFORM_COUNT, vec![expression]),
        DataTransform::Mean => Expression::call(FN_TRANSFORM_MEAN, vec![expression]),
        DataTransform::Median => Expression::call(FN_TRANSFORM_MEDIAN, vec![expression]),
        DataTransform::Min => Expression::call(FN_TRANSFORM_MIN, vec![expression]),
        DataTransform::Max => Expression::call(FN_TRANSFORM_MAX, vec![expression]),
        DataTransform::TimeBetween {
            previous_record_type,
        } => Expression::call(
            FN_TRANSFORM_TIME_BETWEEN,
            vec![expression, Expression::literal(previous_record_type)],
        ),
        DataTransform::TelemetryAtOrBefore { stream, field } => Expression::call(
            FN_TRANSFORM_TELEMETRY_AT_OR_BEFORE,
            vec![
                expression,
                Expression::literal(stream),
                Expression::literal(field),
            ],
        ),
        DataTransform::Detail { name } => Expression::call(
            FN_TRANSFORM_DETAIL,
            vec![expression, Expression::literal(name)],
        ),
        DataTransform::PerCase => Expression::call(FN_TRANSFORM_PER_CASE, vec![expression]),
    }
}

fn parse_data_expression(
    expression: &Expression,
) -> Result<(DataSource, Vec<DataTransform>), DataExpressionError> {
    let ExpressionKind::Call { function, args } = &expression.kind else {
        return Err(DataExpressionError::UnsupportedExpression);
    };

    match function.as_str() {
        FN_DATA_TELEMETRY => {
            let [stream, field] = args.as_slice() else {
                return Err(DataExpressionError::UnsupportedExpression);
            };

            Ok((
                DataSource::Telemetry {
                    stream: text_literal(stream)?,
                    field: text_literal(field)?,
                },
                Vec::new(),
            ))
        }
        FN_DATA_PARAMETER => {
            let [name] = args.as_slice() else {
                return Err(DataExpressionError::UnsupportedExpression);
            };

            Ok((
                DataSource::Parameter {
                    name: text_literal(name)?,
                },
                Vec::new(),
            ))
        }
        FN_DATA_DIMENSION => {
            let [name] = args.as_slice() else {
                return Err(DataExpressionError::UnsupportedExpression);
            };

            Ok((
                DataSource::Dimension {
                    name: text_literal(name)?,
                },
                Vec::new(),
            ))
        }
        FN_DATA_RECORD => {
            let [record_type] = args.as_slice() else {
                return Err(DataExpressionError::UnsupportedExpression);
            };

            Ok((
                DataSource::Record {
                    record_type: text_literal(record_type)?,
                },
                Vec::new(),
            ))
        }
        FN_DATA_ARTIFACT => {
            let [artifact] = args.as_slice() else {
                return Err(DataExpressionError::UnsupportedExpression);
            };

            Ok((
                DataSource::Artifact {
                    artifact: text_literal(artifact)?,
                },
                Vec::new(),
            ))
        }
        FN_TRANSFORM_COUNT => parse_unary_transform(args, DataTransform::Count),
        FN_TRANSFORM_MEAN => parse_unary_transform(args, DataTransform::Mean),
        FN_TRANSFORM_MEDIAN => parse_unary_transform(args, DataTransform::Median),
        FN_TRANSFORM_MIN => parse_unary_transform(args, DataTransform::Min),
        FN_TRANSFORM_MAX => parse_unary_transform(args, DataTransform::Max),
        FN_TRANSFORM_TIME_BETWEEN => {
            let [inner, previous_record_type] = args.as_slice() else {
                return Err(DataExpressionError::UnsupportedExpression);
            };
            let (source, mut transforms) = parse_data_expression(inner)?;
            transforms.push(DataTransform::TimeBetween {
                previous_record_type: text_literal(previous_record_type)?,
            });

            Ok((source, transforms))
        }
        FN_TRANSFORM_TELEMETRY_AT_OR_BEFORE => {
            let [inner, stream, field] = args.as_slice() else {
                return Err(DataExpressionError::UnsupportedExpression);
            };
            let (source, mut transforms) = parse_data_expression(inner)?;
            transforms.push(DataTransform::TelemetryAtOrBefore {
                stream: text_literal(stream)?,
                field: text_literal(field)?,
            });

            Ok((source, transforms))
        }
        FN_TRANSFORM_DETAIL => {
            let [inner, name] = args.as_slice() else {
                return Err(DataExpressionError::UnsupportedExpression);
            };
            let (source, mut transforms) = parse_data_expression(inner)?;
            transforms.push(DataTransform::Detail {
                name: text_literal(name)?,
            });

            Ok((source, transforms))
        }
        FN_TRANSFORM_PER_CASE => parse_unary_transform(args, DataTransform::PerCase),
        _ => Err(DataExpressionError::UnsupportedExpression),
    }
}

fn parse_unary_transform(
    args: &[Expression],
    transform: DataTransform,
) -> Result<(DataSource, Vec<DataTransform>), DataExpressionError> {
    let [inner] = args else {
        return Err(DataExpressionError::UnsupportedExpression);
    };
    let (source, mut transforms) = parse_data_expression(inner)?;
    transforms.push(transform);

    Ok((source, transforms))
}

fn text_literal(expression: &Expression) -> Result<String, DataExpressionError> {
    let ExpressionKind::Literal {
        value: Value::Text(value),
    } = &expression.kind
    else {
        return Err(DataExpressionError::UnsupportedExpression);
    };

    Ok(value.clone())
}

/* === Tests === */
