/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

pub type Result<T> = crate::Result<T>;

#[derive(Debug)]
pub enum DocumentFailure {
    Limit,
    Evaluation { message: &'static str },
    InvalidIdentifier { source: dokime_domain::Error },
    InvalidDefinition { issues: Vec<String> },
}

impl From<dokime_domain::Error> for DocumentFailure {
    fn from(source: dokime_domain::Error) -> Self {
        Self::InvalidIdentifier { source }
    }
}

impl std::fmt::Display for DocumentFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Limit => formatter.write_str("document materialization budget exceeded"),
            Self::Evaluation { message } => {
                write!(formatter, "document binding evaluation failed: {message}")
            }
            Self::InvalidIdentifier { .. } => formatter.write_str("invalid document identifier"),
            Self::InvalidDefinition { issues } => {
                write!(formatter, "invalid document definition: {issues:?}")
            }
        }
    }
}

impl std::error::Error for DocumentFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidIdentifier { source } => Some(source),
            Self::InvalidDefinition { .. } | Self::Evaluation { .. } | Self::Limit => None,
        }
    }
}
