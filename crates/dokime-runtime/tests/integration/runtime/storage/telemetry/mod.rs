/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod stream_revisions_decode_on_one_root;

mod stream_revision_integrity_refuses_ambiguous_stored_definitions;

mod captured_sample_scope;

mod later_refusal_preserves_the_accepted_prefix_without_replay;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime_runtime::testing::telemetry_admission::*;

use dokime_runtime::{
    runtime::Error,
    storage::{
        Manager,
        telemetry::{self, TelemetryEntry},
    },
};

use harmos::{
    Identifier, Runtime,
    storage::{RecordInput, StorageBackends, StreamInput, query::StreamSelection},
};

use std::{num::NonZeroUsize, sync::Arc};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct TestApp;

#[derive(Default, serde::Deserialize)]
struct StorageConfig {
    #[serde(skip)]
    storage: Option<Manager>,
}

#[harmos::message]
struct Reading {
    value: String,
}

#[harmos::message]
struct CaptureReading {
    value: f64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl harmos::Application for TestApp {
    type Lanes = dokime_runtime::app::Lanes;
    type State = ();
    type Transaction = harmos::state::Never;
    type Record = harmos::state::Never;
    type Streams = ();
}

impl harmos::Lifecycle for TestApp {
    type Config = StorageConfig;
    type Error = harmos::Error;
    type Context = harmos::Startup<Self>;
    async fn prepare(config: Self::Config, setup: &mut harmos::Setup<Self>) -> harmos::Result<Self> {
        setup.settings().storage(config.storage.as_ref().unwrap());
        Ok(Self)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
