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
use crate::records::DeclaredRecord;
use dokime_domain::model::records::evidence::Record;

/* === Constants === */
pub const LISTENER_POLICY: harmos::operations::OperationPolicy = harmos::operations::OperationPolicy {
    timeout: std::time::Duration::from_secs(30),
    attempt_timeout: std::time::Duration::from_secs(30),
    retries: harmos::operations::RetryPolicy::Never,
    restart: harmos::operations::RestartPolicy::Never,
};

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[harmos::message]
#[derive(Clone, PartialEq)]
pub struct Delivery {
    #[row(description = "JSON-encoded [`Record`](dokime_domain::model::records::evidence::Record), as the host filed it.")]
    pub record_json: String,
}

// One listener's callback operation: the host delivers the filed record under
// the listener's own operation id.
pub struct Heard<L>(pub Delivery, std::marker::PhantomData<fn() -> L>);

/* === Enums === */

/* === Traits === */
pub trait Listener: 'static {
    const ID: &'static str;

    const DESCRIPTION: &'static str = "";

    type Record: DeclaredRecord;
}

/* === Implementations === */
crate::transport::templated!(Heard, Delivery);

impl<L: Listener> harmos::operations::OperationDeclaration for Heard<L> {
    type Response = ();
    type Input = harmos::operations::NoStream;
    type Output = harmos::operations::NoStream;
    const ID: &'static str = L::ID;
    const VERSION: u32 = 1;
    const DESCRIPTION: &'static str = L::DESCRIPTION;
    const POLICY: harmos::operations::OperationPolicy = LISTENER_POLICY;
    const VISIBILITY: harmos::operations::Visibility = harmos::operations::Visibility::Host;
}

impl Delivery {
    pub fn of(record: &Record) -> Self {
        Self { record_json: serde_json::to_string(record).expect("a filed record serializes") }
    }

    pub fn record<R: DeclaredRecord>(&self) -> crate::Result<R> {
        let record: Record = serde_json::from_str(&self.record_json)
            .map_err(|error| crate::Error::RecordDecode { record: R::RECORD_TYPE.to_owned(), reason: error.to_string() })?;
        R::from_record(&record)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
