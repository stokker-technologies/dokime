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
#[cfg(feature = "native")]
pub use serving::{Handler, Served};

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
                || output.unit.len() > 128
                || output.axis_unit.len() > 128
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
        let definition = Self::definition()?;
        if request.parameters.len() > MAX_PARAMETER_BYTES {
            return Err(Error::Input("analysis configuration exceeds its budget".into()));
        }
        let samples = request.samples.ok_or_else(|| Error::Input("analysis samples are missing".into()))?;
        samples.check(definition.limits)?;
        let parameters: BTreeMap<String, Value> = serde_json::from_slice(&request.parameters)?;
        let selection = Selection { id: definition.id.clone(), version: definition.version, parameters };
        let values = definition.resolve(&selection).map_err(Error::Input)?;
        let parameters = A::Parameters::from_parameters(&values).map_err(Error::Parameters)?;
        let result = A::evaluate(&samples, &parameters).map_err(|error| Error::Evaluation(Box::new(error)))?;
        let response = result.into_response();
        response.check(&definition)?;
        Ok(response)
    }
}

impl crate::catalog::Visitor for Definitions {
    type Entry = fn() -> Result<Definition, Error>;
}

#[cfg(feature = "native")]
impl<A: Analysis> crate::catalog::Member<Handler<A>> for Definitions {
    const ENTRY: Self::Entry = Operation::<A>::definition;
}

#[cfg(feature = "native")]
impl<A: Analysis, R> crate::catalog::Member<Served<A, R>> for Definitions {
    const ENTRY: Self::Entry = Operation::<A>::definition;
}

impl<A: Analysis> crate::catalog::Member<Operation<A>> for Definitions {
    const ENTRY: Self::Entry = Operation::<A>::definition;
}

crate::transport::templated!(Operation, Request);

impl<A: Analysis> harmos::operations::OperationDeclaration for Operation<A> {
    type Response = Response;
    type Input = harmos::operations::NoStream;
    type Output = harmos::operations::NoStream;
    const ID: &'static str = A::ID;
    const VERSION: u32 = A::VERSION;
    const DESCRIPTION: &'static str = A::DESCRIPTION;
    const POLICY: harmos::operations::OperationPolicy = harmos::operations::OperationPolicy {
        timeout: A::TIMEOUT,
        attempt_timeout: A::TIMEOUT,
        retries: harmos::operations::RetryPolicy::Never,
        restart: harmos::operations::RestartPolicy::Never,
    };
    const VISIBILITY: harmos::operations::Visibility = harmos::operations::Visibility::Shared;
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
