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

// Local Imports
use dokime_domain::model::identity::Identifier;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

pub type Result<T> = crate::Result<T>;

/* === Structs === */

/* === Enums === */

#[derive(Debug)]
pub enum DocumentFailure {
    Limit,
    Budget { budget: &'static str, limit: usize, reached: usize, card: String },
    Evaluation { message: &'static str },
    InvalidIdentifier { source: dokime_domain::Error },
    InvalidDefinition { issues: Vec<String> },
    // A repeated section must select row-local values; `time_between` reads
    // across rows, so it cannot name the instances of a section.
    CorrelatedRepeat { section: Identifier },
}

/* === Traits === */

/* === Implementations === */

impl From<dokime_domain::Error> for DocumentFailure {
    fn from(source: dokime_domain::Error) -> Self {
        Self::InvalidIdentifier { source }
    }
}

impl std::fmt::Display for DocumentFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Limit => formatter.write_str("document materialization budget exceeded"),
            Self::Budget { budget, limit, reached, card } => {
                write!(formatter, "card {card}: document budget {budget} exceeded (limit {limit}, reached {reached})")
            }
            Self::Evaluation { message } => {
                write!(formatter, "document binding evaluation failed: {message}")
            }
            Self::InvalidIdentifier { .. } => formatter.write_str("invalid document identifier"),
            Self::InvalidDefinition { issues } => {
                write!(formatter, "invalid document definition: {issues:?}")
            }
            Self::CorrelatedRepeat { section } => {
                write!(formatter, "repeated section {section} cannot select on time_between; repeat selectors must be row-local")
            }
        }
    }
}

impl std::error::Error for DocumentFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidIdentifier { source } => Some(source),
            Self::Budget { .. }
            | Self::InvalidDefinition { .. }
            | Self::Evaluation { .. }
            | Self::Limit
            | Self::CorrelatedRepeat { .. } => None,
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
