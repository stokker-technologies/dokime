/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

#[cfg(feature = "native")]
mod serving;

/* === Re-Exports === */

pub use dokime_domain::model::analysis::Axis;
pub use dokime_domain::model::analysis::Definition;
pub use dokime_domain::model::analysis::Limits;
pub use dokime_domain::model::analysis::MAX_PARAMETER_BYTES;
pub use dokime_domain::model::analysis::MAX_SAMPLES;
pub use dokime_domain::model::analysis::Output;
pub use dokime_domain::model::analysis::Selection;

/* === Imports === */

// Standard Library Imports
use std::collections::{BTreeMap, BTreeSet};
use std::marker::PhantomData;
use std::time::Duration;

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports
use crate::parameters::FromParameters;
use dokime_domain::model::values::Value;

/* === Constants === */

/* === Macros === */

crate::transport::templated!(Operation, Request);

/* === Type Aliases === */

/* === Structs === */

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Samples {
    pub axis: Vec<f64>,
    pub values: Vec<f64>,
    pub axis_unit: String,
    pub unit: String,
}

#[harmos::message]
#[derive(Clone, PartialEq)]
pub struct Request {
    pub samples: Option<Samples>,
    pub parameters: Vec<u8>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Values {
    pub key: String,
    pub unit: String,
    pub axis_unit: String,
    pub axis: Vec<f64>,
    pub values: Vec<f64>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Response {
    pub outputs: Vec<Values>,
}

// One analysis operation: the portable `Request` under the analysis identity.
pub struct Operation<A>(pub Request, PhantomData<fn() -> A>);

pub struct Definitions;

/* === Enums === */

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid analysis declaration: {0}")]
    Declaration(String),

    #[error("invalid analysis input: {0}")]
    Input(String),

    #[error("invalid analysis output: {0}")]
    Output(String),

    #[error("analysis parameters could not be decoded")]
    Parameters(#[source] crate::Error),

    #[error("analysis parameter encoding failed")]
    Encoding(#[from] serde_json::Error),

    #[error("analysis evaluation failed")]
    Evaluation(#[source] Box<dyn std::error::Error + Send + Sync>),
}

/* === Traits === */

pub trait Analysis: Send + Sync + 'static {
    type Parameters: FromParameters;
    type Output: ResultData;
    type Error: std::error::Error + Send + Sync + 'static;

    const ID: &'static str;
    const VERSION: u32;
    const LABEL: &'static str;
    const DESCRIPTION: &'static str;
    const TIMEOUT: Duration;
    const LIMITS: Limits;

    fn evaluate(samples: &Samples, parameters: &Self::Parameters) -> Result<Self::Output, Self::Error>;
}

pub trait ResultData: Send + 'static {
    fn outputs() -> Vec<Output>;
    fn into_response(self) -> Response;
}

/* === Implementations === */

impl Samples {
    pub fn check(&self, limits: Limits) -> Result<(), Error> {
        limits.check().map_err(Error::Declaration)?;
        if self.values.is_empty() || self.values.len() > limits.max_samples as usize || self.axis.len() != self.values.len() {
            return Err(Error::Input("samples require equally sized, nonempty bounded columns".into()));
        }
        if self.axis.iter().chain(&self.values).any(|value| !value.is_finite()) || self.axis.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Err(Error::Input("samples require finite values and a strictly increasing axis".into()));
        }
        Ok(())
    }
}

impl Response {
    pub fn check_limits(&self) -> Result<(), Error> {
        if self.outputs.iter().any(|output| {
            output.unit.len() as u64 > dokime_domain::model::analysis::MAX_UNIT_BYTES
                || output.axis_unit.len() as u64 > dokime_domain::model::analysis::MAX_UNIT_BYTES
        }) {
            return Err(Error::Output("response units exceed their configured byte budget".into()));
        }
        Ok(())
    }

    pub fn check(&self, definition: &Definition) -> Result<(), Error> {
        definition.check().map_err(Error::Declaration)?;
        if self.outputs.len() != definition.outputs.len() {
            return Err(Error::Output("response must contain every declared output exactly once".into()));
        }
        let mut seen = BTreeSet::new();
        let mut total = 0usize;
        for output in &self.outputs {
            let schema = definition
                .outputs
                .iter()
                .find(|schema| schema.key == output.key)
                .ok_or_else(|| Error::Output("response contains an undeclared output".into()))?;
            if !seen.insert(&output.key) {
                return Err(Error::Output("response repeats an output".into()));
            }
            total = total.checked_add(output.values.len()).ok_or_else(|| Error::Output("output size overflow".into()))?;
            if total > definition.limits.max_output_values as usize
                || output.values.iter().chain(&output.axis).any(|value| !value.is_finite())
            {
                return Err(Error::Output("response exceeds its budget or contains nonfinite values".into()));
            }
            if schema.unit.as_ref().is_some_and(|unit| unit != &output.unit)
                || schema.axis.as_ref().and_then(|axis| axis.unit.as_ref()).is_some_and(|unit| unit != &output.axis_unit)
            {
                return Err(Error::Output("response units do not match the declared output".into()));
            }
            match &schema.axis {
                None if output.axis.is_empty() && output.values.len() == 1 => {}
                Some(_) if output.axis.len() == output.values.len() && output.axis.windows(2).all(|pair| pair[0] < pair[1]) => {}
                _ => {
                    return Err(Error::Output("response does not match its declared scalar or series shape".into()));
                }
            }
        }
        Ok(())
    }
}

impl<A: Analysis> Operation<A> {
    pub fn definition() -> Result<Definition, Error> {
        let definition = Definition {
            id: A::ID.into(),
            version: A::VERSION,
            label: A::LABEL.into(),
            description: A::DESCRIPTION.into(),
            parameters: A::Parameters::declaration()
                .into_iter()
                .map(|parameter| parameter.try_into_definition().map_err(|error| Error::Declaration(error.to_string())))
                .collect::<Result<_, _>>()?,
            outputs: A::Output::outputs(),
            limits: A::LIMITS,
        };
        definition.check().map_err(Error::Declaration)?;
        if A::TIMEOUT.is_zero() {
            return Err(Error::Declaration("analysis timeout must be positive".into()));
        }
        Ok(definition)
    }

    pub fn evaluate(request: Request) -> Result<Response, Error> {
        let mut definition = Self::definition()?;
        definition.check_limits().map_err(Error::Declaration)?;
        definition.limits = definition.limits.constrained();
        if request.parameters.len() as u64 > dokime_domain::model::analysis::MAX_PARAMETER_BYTES as u64 {
            return Err(Error::Input("analysis configuration exceeds its budget".into()));
        }
        let samples = request.samples.ok_or_else(|| Error::Input("analysis samples are missing".into()))?;
        samples.check(definition.limits)?;
        let parameters: BTreeMap<String, Value> = serde_json::from_slice(&request.parameters)?;
        let selection = Selection {
            id: definition.id.clone(),
            version: definition.version,
            parameters,
        };
        let values = definition.resolve(&selection).map_err(Error::Input)?;
        let parameters = A::Parameters::from_parameters(&values).map_err(Error::Parameters)?;
        let result = A::evaluate(&samples, &parameters).map_err(|error| Error::Evaluation(Box::new(error)))?;
        let response = result.into_response();
        response.check(&definition)?;
        response.check_limits()?;
        Ok(response)
    }
}

impl crate::catalog::Visitor for Definitions {
    type Entry = fn() -> Result<Definition, Error>;
}

impl<A: Analysis> crate::catalog::Member<Operation<A>> for Definitions {
    const ENTRY: Self::Entry = Operation::<A>::definition;
}

impl<A: Analysis> harmos::operations::OperationDeclaration for Operation<A> {
    type Request = Self;
    type Response = Response;
    type Input = harmos::operations::NoStream;
    type Output = harmos::operations::NoStream;
    const ID: &'static str = A::ID;
    const DESCRIPTION: &'static str = A::DESCRIPTION;
    const POLICY: harmos::operations::OperationPolicy = harmos::operations::OperationPolicy {
        timeout: A::TIMEOUT,
        attempt_timeout: A::TIMEOUT,
        cancellation_timeout: std::time::Duration::ZERO,
        retries: harmos::operations::RetryPolicy::Never,
    };
    const VISIBILITY: harmos::operations::Visibility = harmos::operations::Visibility::Public;
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn result_units_output() {
        let definition = Definition {
            id: "analysis-fixture.scale".into(),
            version: 1,
            label: "Scale Signal".into(),
            description: "Multiply Original Samples By A Typed Factor".into(),
            parameters: Vec::new(),
            outputs: vec![Output {
                key: "scaled".into(),
                label: "Scaled Signal".into(),
                unit: Some("V".into()),
                axis: Some(Axis {
                    key: "time".into(),
                    unit: Some("s".into()),
                }),
            }],
            limits: Limits { max_samples: 64, max_output_values: 64 },
        };
        let mut response = Response {
            outputs: vec![Values {
                key: "scaled".into(),
                unit: "V".into(),
                axis_unit: "s".into(),
                axis: vec![0.0, 0.25, 0.5],
                values: vec![4.0, -2.0, 6.0],
            }],
        };
        assert!(response.check(&definition).is_ok());

        response.outputs[0].axis_unit = "Hz".into();
        assert!(matches!(response.check(&definition), Err(Error::Output(_))));
        response.outputs[0].axis_unit = "s".into();
        response.outputs[0].unit = "A".into();
        assert!(matches!(response.check(&definition), Err(Error::Output(_))));

        response.outputs[0].unit = "V".into();
        assert!(response.check(&definition).is_ok());
    }
}
