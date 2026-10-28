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
use std::fmt;
use std::str::FromStr;

// Third Party Imports
use serde::Deserialize;
use serde::Serialize;

// Local Imports

/* === Constants === */

/* === Macros === */

product_error!(PermissionError, formatter {
    Self::WrongSegmentCount => formatter.write_str("permission atom must have `family:action:target` segments"),
    Self::EmptySegment { segment } => write!(formatter, "permission atom `{segment}` segment is empty"),
    Self::PartialWildcard { segment } => write!(formatter, "permission atom `{segment}` wildcard must occupy the whole segment"),
});

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Permission {
    family: String,
    action: String,
    target: String,
}

/* === Enums === */

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PermissionError {
    WrongSegmentCount,
    EmptySegment { segment: &'static str },
    PartialWildcard { segment: &'static str },
}

/* === Traits === */

/* === Implementations === */

impl Permission {
    pub fn new(family: impl Into<String>, action: impl Into<String>, target: impl Into<String>) -> crate::Result<Self> {
        let permission = Self {
            family: family.into(),
            action: action.into(),
            target: target.into(),
        };
        permission.validate()?;
        Ok(permission)
    }

    pub fn family(&self) -> &str {
        &self.family
    }

    pub fn action(&self) -> &str {
        &self.action
    }

    pub fn target(&self) -> &str {
        &self.target
    }

    pub fn covers(&self, required: &Self) -> bool {
        segment_covers(&self.family, &required.family)
            && segment_covers(&self.action, &required.action)
            && segment_covers(&self.target, &required.target)
    }

    fn validate(&self) -> Result<(), PermissionError> {
        validate_segment("family", &self.family)?;
        validate_segment("action", &self.action)?;
        validate_segment("target", &self.target)
    }
}

impl fmt::Display for Permission {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}:{}:{}", self.family, self.action, self.target)
    }
}

impl From<Permission> for String {
    fn from(permission: Permission) -> Self {
        permission.to_string()
    }
}

impl FromStr for Permission {
    type Err = crate::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut segments = value.splitn(3, ':');
        let family = segments.next().ok_or_else(|| crate::Error::from(PermissionError::WrongSegmentCount))?;
        let action = segments.next().ok_or_else(|| crate::Error::from(PermissionError::WrongSegmentCount))?;
        let target = segments.next().ok_or_else(|| crate::Error::from(PermissionError::WrongSegmentCount))?;
        Self::new(family, action, target)
    }
}

impl TryFrom<String> for Permission {
    type Error = crate::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn segment_covers(grant: &str, required: &str) -> bool {
    grant == "*" || grant == required
}

fn validate_segment(segment: &'static str, value: &str) -> Result<(), PermissionError> {
    if value.is_empty() {
        return Err(PermissionError::EmptySegment { segment });
    }
    if value.contains('*') && value != "*" {
        return Err(PermissionError::PartialWildcard { segment });
    }
    Ok(())
}

/* === Tests === */
