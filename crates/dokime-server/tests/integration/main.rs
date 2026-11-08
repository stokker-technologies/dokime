/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

#![recursion_limit = "256"]

/* === Modules === */

mod grpc;
mod host;

mod finite_queries;
mod query_roundtrip;
mod watch_liveness;

#[path = "../support/mod.rs"]
mod support;

mod storage_format;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::time::Duration;
// Third Party Imports
use tonic::{
    Code, Request,
    transport::{Certificate, ClientTlsConfig, Endpoint},
};
// Local Imports
use dokime::client::contracts;
pub use dokime_server::Error;
use dokime_server::grpc::{Config, Server, TlsIdentity};
use support::wire::*;
use {dokime::Identifier, dokime::Metadata, dokime::Runtime};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

async fn stage(future: impl std::future::Future<Output = ()>) {
    let name = std::any::type_name_of_val(&future);
    let started = std::time::Instant::now();
    eprintln!("Stage: {name}");
    future.await;
    eprintln!("Completed: {name} ({:?})", started.elapsed());
}

/* === Tests === */
