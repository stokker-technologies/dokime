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
use dokime_domain::model::identity::Metadata;
use dokime_domain::model::values::Value;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct AggregateOutput {
    pub stream: String,
    pub field: String,
    pub statistic: String,
    pub value: Value,
    pub created_at: Option<time::OffsetDateTime>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl AggregateOutput {
    pub(crate) fn key(&self) -> String {
        format!("telemetry.{}.values.{}.{}", self.stream, self.field, self.statistic)
            .chars()
            .map(|character| if character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-') { character } else { '_' })
            .collect()
    }

    pub(crate) fn parameter(&self) -> Result<crate::definition::MaterializedParameter, String> {
        Ok(crate::definition::MaterializedParameter {
            metadata: self.metadata()?,
            group_id: Some(dokime_domain::model::identity::Identifier::new("telemetry.aggregate").expect("valid aggregate group")),
            source: crate::definition::ParameterSource::Telemetry,
            value: self.value.clone(),
            unit: None,
        })
    }

    pub(crate) fn from_parameter(key: &str, parameter: &crate::definition::MaterializedParameter) -> Option<Self> {
        if parameter.source != crate::definition::ParameterSource::Telemetry {
            return None;
        }
        let (stream, value) = key.strip_prefix("telemetry.")?.rsplit_once(".values.")?;
        let (field, statistic) = value.rsplit_once('.')?;
        let output = Self {
            stream: stream.to_owned(),
            field: field.to_owned(),
            statistic: statistic.to_owned(),
            value: parameter.value.clone(),
            created_at: parameter.metadata.created_at,
        };
        (output.parameter().ok().as_ref() == Some(parameter)).then_some(output)
    }

    pub(crate) fn metadata(&self) -> Result<Metadata, String> {
        let (stream, field, statistic) = (&self.stream, &self.field, &self.statistic);
        let mut metadata = Metadata::definition(
            dokime_domain::model::identity::Identifier::new(self.key()).map_err(|error| error.to_string())?,
            format!("{} {} {}", stream, field.replace('_', " "), statistic.replace('_', " ")),
            Some(format!("Telemetry aggregate {statistic} for {stream} {field}.")),
            ["output", "telemetry", "aggregate"],
        );
        metadata.created_at = self.created_at;
        Ok(metadata)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
