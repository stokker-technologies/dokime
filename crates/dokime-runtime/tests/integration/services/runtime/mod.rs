/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub(crate) mod commands;
pub(crate) mod config_application;
pub(crate) mod config_settings;
pub(crate) mod identity;
pub(crate) mod lifecycle;
pub(crate) mod queries;
pub(crate) mod query_records;
pub(crate) mod settings;
pub(crate) mod storage;

/* === Re-Exports === */
pub(crate) use crate::service_support::prelude::*;

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime_domain::model;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
async fn record_health(runtime: &RuntimeHandle, message: &str) -> dokime_runtime::runtime::Result<()> {
    runtime
        .add_health_item(
            RecordScope::default(),
            model::execution::HealthSeverity::Info,
            message.to_owned(),
            None,
        )
        .await
}

/* === Tests === */
