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
use std::collections::BTreeMap;
// Third Party Imports

// Local Imports
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::values::Value;
use dokime_domain::model::records::publication::RecordPublishRequest;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */
#[derive(Clone, Debug, PartialEq)]
pub enum Publication {
    Row {
        stream: Identifier,
        values: BTreeMap<String, Value>,
        payload: Vec<u8>,
    },
    Record(Box<RecordPublishRequest>),
}

/* === Traits === */
pub trait Publishable {
    type Payload: Into<Publication>;

    fn publication(self) -> Self::Payload;
}

/* === Implementations === */
impl Publishable for RecordPublishRequest {
    type Payload = Self;

    fn publication(self) -> Self {
        self
    }
}

impl From<RecordPublishRequest> for Publication {
    fn from(request: RecordPublishRequest) -> Self {
        Self::Record(Box::new(request))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
