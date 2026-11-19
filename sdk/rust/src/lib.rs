/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Rust Plugin Authoring and Typed Protocol Clients.

/* === Modules === */

#[rustfmt::skip]
mod exports;
mod wire {
    pub(crate) use dokime_protocol::{host, model, plugin};
}

mod authoring;
pub mod client;
mod configuration;
mod context;
mod data;
mod error;
pub mod operations;
mod payload;
pub mod templates;
#[rustfmt::skip]
mod host;
mod message;
mod operation;
pub mod plugin;
pub mod prelude;
mod prompt;
mod record;
mod stream;
mod suite;
mod template;
pub mod testing;
mod transfers;

/* === Re-Exports === */

pub use ::harmos::plugin::Lane;
pub use client::{
    Attribution, Case, Client, ConnectionSettings, Cycle, Execution, Outcome, Progress, ProgressStream, connect, connect_with,
};
pub use client::{RegisteredPlugin, ResultStatus};
use context::Context;
pub use data::FromValue;
pub use dokime_derive::{Message, Parameters, Record, Row, Settings};
pub use error::{Error, Result};
pub use exports::*;
pub use payload::Payload;

pub use plugin::Plugin;
use record::Record;
use stream::{Observer, Stream};
pub use suite::Suite;

/* === Imports === */

// Standard Library Imports

// Third Party Imports
extern crate self as dokime;

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

// Protobuf Data with the Harmos Descriptor Contract.
pub trait Message: ::harmos::Message {}

/* === Implementations === */

impl Message for () {}

/* === Public Functions === */

/* === Private Functions === */

// Dependencies used by the Harmos data derives, including when renamed by a consumer.
#[doc(hidden)]
pub mod encoding {
    /* === Modules === */

    /* === Re-Exports === */

    pub mod contract {
        pub use ::harmos::Message;
    }
    pub use crate::message::Wire;
    pub use ::harmos::encoding::*;

    /* === Imports === */

    // Standard Library Imports

    // Third Party Imports

    // Local Imports

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
}

/* === Tests === */
