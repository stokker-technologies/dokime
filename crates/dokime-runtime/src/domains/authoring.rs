/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use crate::domains::Permission;
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::identity::Metadata;

pub fn identifier(value: &str) -> Identifier {
    Identifier::new(value).unwrap_or_else(|error| panic!("invalid artifact identifier {value}: {error}"))
}

pub fn permission(atom: &str) -> Permission {
    atom.parse().unwrap_or_else(|error| panic!("invalid artifact permission {atom}: {error}"))
}

pub fn metadata(identifier_value: &str, label: &str, description: &str) -> Metadata {
    Metadata::new(identifier(identifier_value), label, Some(description.to_owned()))
}

pub fn bounded(value: u32, default: u32, maximum: u32) -> u32 {
    if value == 0 { default } else { value.min(maximum) }
}
