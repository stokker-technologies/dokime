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

pub mod client;
mod context;
mod data;
mod error;
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

/* === Re-Exports === */

pub use client::{Client, ClientSettings};
use context::Context;
pub use data::FromValue;
pub use dokime_derive::{Message, Parameters, Record, Row, Settings};
pub use error::{Error, Result};
pub use exports::*;
pub use harmos::OperationRequest;

use plugin::Plugin;
use record::Record;
use stream::{Observer, Stream};

/* === Imports === */

// Standard Library Imports

// Third Party Imports
extern crate self as dokime;

// Local Imports
use harmos::Message as HarmosMessage;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

// Protobuf Data with the Harmos Descriptor Contract.
pub trait Message: HarmosMessage {}

/* === Implementations === */

impl<T: HarmosMessage> Message for T {}

/* === Public Functions === */

/* === Private Functions === */

// Dependencies used by the Harmos data derives, including when renamed by a consumer.
#[doc(hidden)]
pub mod encoding {
    /* === Modules === */

    /* === Re-Exports === */

    pub use harmos::Message as ContractMessage;
    pub use harmos::encoding::*;

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
