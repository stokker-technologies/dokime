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

mod authoring_catalog;
mod execution_artifacts;
mod execution_run;
mod execution_seal_report;
mod plugin_lifecycle;
mod server_diagnostics;
mod server_pairing_remote;
mod settings_backends;
mod telemetry_watch_plot;
mod watch_liveness;

#[path = "../support/mod.rs"]
mod support;

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

/* === Tests === */
