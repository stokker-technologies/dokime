/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Arrow Telemetry Ownership

/* === Modules === */

pub(crate) mod aggregates;
pub(crate) mod catalog;
pub(crate) mod codec;
pub(crate) mod config;
pub(crate) mod previews;
mod reader;
mod writer;

/* === Re-Exports === */

pub use reader::BatchSelection;

/* === Imports === */

// Standard Library Imports
use std::{collections::BTreeMap, sync::Arc};

// Third Party Imports
use tokio::time::Instant;

// Local Imports
use crate::{Error, Result, Sample, ipc};
use codec::Codec;
use config::StorageSettings;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(crate) struct Telemetry {
    settings: StorageSettings,
    pending: BTreeMap<(String, String), Pending>,
    writers: BTreeMap<String, ipc::Writer>,
    codecs: BTreeMap<(String, String), Arc<Codec>>,
    running: bool,
    closed: bool,
    workers: Vec<tokio::task::JoinHandle<()>>,
}
struct Pending {
    codec: Arc<Codec>,
    started: Instant,
    samples: Vec<Sample>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Telemetry {
    pub(crate) fn new(settings: StorageSettings) -> Result<Self> {
        if settings.previews.bucket_interval.is_zero()
            || settings.previews.bucket_interval.as_nanos() > i128::MAX as u128
            || settings.flush_interval.is_zero()
            || Instant::now().checked_add(settings.flush_interval).is_none()
        {
            return Err(Error::Data("set a positive representable storage flush interval"));
        }
        Ok(Self {
            settings,
            pending: BTreeMap::new(),
            writers: BTreeMap::new(),
            codecs: BTreeMap::new(),
            running: false,
            closed: false,
            workers: Vec::new(),
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
