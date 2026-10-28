/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

pub(crate) mod commands;
pub(crate) mod config_application;
pub(crate) mod identity;
pub(crate) mod lifecycle;
pub(crate) mod queries;
pub(crate) mod query_records;
pub(crate) mod secrets;
pub(crate) mod settings;
pub(crate) mod storage;

pub(crate) use crate::service_support::prelude::*;

use dokime_domain::model;

async fn record_health(runtime: &RuntimeHandle, message: &str) -> dokime_runtime::runtime::Result<()> {
    runtime.add_health_item(RecordScope::default(), model::execution::HealthSeverity::Info, message.to_owned(), None).await
}
