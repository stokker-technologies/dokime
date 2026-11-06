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

pub use super::templates::analysis::Selection as AnalysisSelection;

// The Native Leaves Keep Their Existing Schemas And Conversion Owners.
pub use super::annotations::{Health, Note, Set as Annotations};
pub use super::execution::Revision as ExecutionRevision;
pub use super::metadata::Row as Metadata;
pub use super::templates::documents::{
    Axis as PlotAxis, Band as ReferenceBand, CardLayout, Data as DataExpression, Document as DocumentDefinition, Fft as FftOptions,
    Filter as DocumentFilter, Format as ValueFormat, Histogram as HistogramOptions, Interval as SigmaIntervalOptions,
    Line as ReferenceLine,
};
pub use super::templates::prompts::Prompt as PromptDefinition;
pub use super::templates::{Map as Values, Unit as MeasurementUnit, Value};
pub use crate::storage::records::ScopeRow as Scope;
pub use crate::storage::telemetry::Sample as TelemetrySample;

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

// Local Imports
use crate::prompts::{PromptInstance as DomainPrompt, PromptKind};
use crate::storage::{Error, Result};
use dokime_domain::model::identity::ContractId;
use dokime_domain::model::identity::PromptId;
use dokime_domain::model::time::Timestamp;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[harmos::message]
#[derive(Clone, PartialEq)]
pub struct PromptInstance {
    prompt_id: String,
    definition: Option<PromptDefinition>,
    kind: u32,
    allow_timeout: bool,
    expires_at: Option<String>,
    values: Option<Values>,
    scope: Option<Scope>,
    created_at: String,
    contract_id: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl TryFrom<&DomainPrompt> for PromptInstance {
    type Error = Error;

    fn try_from(value: &DomainPrompt) -> Result<Self> {
        value.created_at.to_datetime()?;
        if let Some(time) = &value.expires_at {
            time.to_datetime()?;
        }
        Ok(Self {
            prompt_id: value.prompt_id.as_str().to_owned(),
            definition: Some(PromptDefinition::try_from(&value.definition)?),
            kind: match value.kind {
                PromptKind::Confirmation => 1,
                PromptKind::Form => 2,
                PromptKind::Choice => 3,
                PromptKind::Status => 4,
                PromptKind::Verification => 5,
            },
            allow_timeout: value.allow_timeout,
            expires_at: value.expires_at.as_ref().map(|time| time.as_str().to_owned()),
            values: Some(Values::try_from(&value.values)?),
            scope: Some(Scope::from(&value.scope)),
            created_at: value.created_at.as_str().to_owned(),
            contract_id: value.contract_id.as_str().to_owned(),
        })
    }
}

impl TryFrom<PromptInstance> for DomainPrompt {
    type Error = Error;

    fn try_from(value: PromptInstance) -> Result<Self> {
        OffsetDateTime::parse(&value.created_at, &Rfc3339).map_err(|_| Error::Invalid("captured prompt timestamp is invalid"))?;
        if let Some(time) = &value.expires_at {
            OffsetDateTime::parse(time, &Rfc3339).map_err(|_| Error::Invalid("captured prompt expiry is invalid"))?;
        }
        Ok(Self {
            prompt_id: PromptId::from_uuid(value.prompt_id)?,
            definition: value.definition.ok_or(Error::Invalid("captured prompt definition is missing"))?.try_into()?,
            kind: kind(value.kind)?,
            allow_timeout: value.allow_timeout,
            expires_at: value.expires_at.map(Timestamp::new),
            values: value.values.ok_or(Error::Invalid("captured prompt values are missing"))?.try_into()?,
            scope: value.scope.ok_or(Error::Invalid("captured prompt scope is missing"))?.try_into()?,
            created_at: Timestamp::new(value.created_at),
            contract_id: ContractId::from_uuid(value.contract_id)?,
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn kind(value: u32) -> Result<PromptKind> {
    match value {
        1 => Ok(PromptKind::Confirmation),
        2 => Ok(PromptKind::Form),
        3 => Ok(PromptKind::Choice),
        4 => Ok(PromptKind::Status),
        5 => Ok(PromptKind::Verification),
        _ => Err(Error::Invalid("captured prompt kind is unknown")),
    }
}

/* === Tests === */
