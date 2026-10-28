/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::collections::BTreeMap;

use dokime_domain::model::identity::Identifier;
use dokime_domain::model::records::publication::RecordPublishRequest;
use dokime_domain::model::values::Value;

#[derive(Clone, Debug, PartialEq)]
pub enum Publication {
    Row { stream: Identifier, values: BTreeMap<String, Value>, payload: Vec<u8> },
    Record(Box<RecordPublishRequest>),
}

pub trait Publishable {
    type Payload: Into<Publication>;

    fn publication(self) -> Self::Payload;
}

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
