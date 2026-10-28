/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

#![recursion_limit = "256"]

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
mod plugin_admission;
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

use std::time::Duration;

use tonic::{
    Code, Request,
    transport::{Certificate, ClientTlsConfig, Endpoint},
};

use dokime::client::contracts;
use dokime_server::grpc::{Config, Server, TlsIdentity};
use support::wire::*;
use {dokime::Identifier, dokime::Metadata, dokime::Runtime};
