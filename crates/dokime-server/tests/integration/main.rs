/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod appearance;
mod append;
mod authoring;
mod authorization;
mod catalog;
mod content;
mod diagnostics;
mod documents;
mod evidence;
mod file_watch;
mod history;
mod media;
mod notes;
mod planning;
mod plots;
mod plugins;
mod prompts;
mod query;
mod records;
mod server;
mod settings;
mod telemetry;
mod trace;
mod traceability;
mod workflow;

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
use dokime_app::{
    Runtime,
    core::{Identifier, Metadata},
};
use dokime_client::contracts;
use dokime_server::grpc::{
    Config, Server, TlsIdentity,
    access::{BearerGrant, Role},
};
use support::wire::*;

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
