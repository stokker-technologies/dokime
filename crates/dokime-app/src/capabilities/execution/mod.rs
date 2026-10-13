/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub mod authoring;
mod handle;
pub(crate) mod lifecycle;
pub(crate) mod outputs;
pub(crate) mod planning;
pub mod progress;
pub mod provider;
pub(crate) mod provider_finish;
pub(crate) mod registry;
pub mod requests;
pub(crate) mod run;

/* === Re-Exports === */
pub use registry::SuiteSnapshot;

/* === Imports === */
// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{
    Error, RuntimeHandle, RuntimeState, transactions::Transactions, unavailable_rejection,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl RuntimeHandle {
    pub(crate) async fn commit_with_key<T>(
        &self,
        idempotency_key: Option<String>,
        transaction: T,
    ) -> Result<harmos::state::Receipt, Error>
    where
        T: harmos::state::Transaction<RuntimeState> + Into<Transactions>,
        T::Error: From<String> + std::fmt::Display,
    {
        match idempotency_key.filter(|key| !key.is_empty()) {
            Some(key) => {
                self.commit_keyed(harmos::state::RequestKey::new(key), transaction)
                    .await
            }
            None => self.commit(transaction).await,
        }
        .map_err(|error| unavailable_rejection(error.to_string()))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
