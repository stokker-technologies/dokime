/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Measure Admitted Native Column Publication

#![recursion_limit = "256"]

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
    time::{Instant, SystemTime, UNIX_EPOCH},
};

// Third Party Imports
use async_trait::async_trait;

// Local Imports
use dokime::{
    harmos::{
        self, Message,
        schema::{Described, graph::Schema},
    },
    runtime::{EmbeddedHost, EmbeddedProvider},
};
use dokime_server::host::{Args, NetworkArgs, Session};

/* === Constants === */

const WINDOW: usize = 10_000;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[dokime::message]
struct Reading {
    counter: i64,
    value: f64,
}

#[derive(Default)]
struct Provider {
    host: Mutex<Option<EmbeddedHost>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

#[async_trait]
impl EmbeddedProvider for Provider {
    fn catalog(&self) -> dokime::InstalledPlugin {
        let schema = Schema::from_declaration(Reading::DEFINITION, &Default::default()).unwrap();
        let columns: Vec<_> = [("counter", "integer"), ("value", "number")]
            .into_iter()
            .map(|(id, kind)| {
                serde_json::from_value::<dokime::telemetry::TelemetryColumn>(serde_json::json!({
                    "metadata": {"identifier": id, "label": id, "description": null},
                    "value_type": kind, "role": "value", "unit": null, "display_unit": null,
                    "required": true, "nullable": false, "default": null
                }))
                .unwrap()
            })
            .collect();
        dokime::InstalledPlugin {
            source: "batch".into(),
            version: "0.1.0".into(),
            enabled: true,
            telemetry_contracts: vec![
                dokime::telemetry::TelemetryContract::declared("batch.samples", "Samples")
                    .with_storage_schema(schema, WINDOW)
                    .columns(columns),
            ],
            ..Default::default()
        }
    }
    fn streams(&self) -> Vec<harmos::sidecar::StreamDeclaration> {
        vec![harmos::sidecar::StreamDeclaration {
            metadata: dokime::Metadata::new(dokime::Identifier::new("batch.samples").unwrap(), "Samples", None),
            row_metadata: vec![],
            id: "batch.samples".into(),
            window: WINDOW,
            row: Schema::from_declaration(Reading::DEFINITION, &Default::default()).unwrap(),
        }]
    }
    async fn initialize(&self, host: EmbeddedHost) -> dokime::runtime::Result<()> {
        *self.host.lock().unwrap() = Some(host);
        Ok(())
    }
    async fn start(&self) -> dokime::runtime::Result<Vec<Arc<dyn dokime::execution::Runnable>>> {
        Ok(vec![])
    }
    async fn stop(&self) -> dokime::runtime::Result<()> {
        self.host.lock().unwrap().take();
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn main() {
    std::thread::Builder::new()
        .name("native-authoring-proof".into())
        .stack_size(16 * 1024 * 1024)
        .spawn(story)
        .unwrap()
        .join()
        .unwrap()
        .unwrap();
}

#[tokio::main]
async fn story() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let root = tempfile::tempdir()?;
    let args = Args {
        shutdown_stdin: false,
        dev_control: false,
        network: NetworkArgs {
            host: std::net::Ipv4Addr::LOCALHOST.into(),
            port: Some(0),
            message_limit_mib: None,
        },
        title: None,
        label: None,
        output_root: None,
        application_root: Some(root.path().into()),
        env_file: None,
        plugins: vec![],
        plugins_root: None,
    };
    let provider = Arc::new(Provider::default());
    let mut session = Session::start(&args, args.configuration()?, args.network.config()?, vec![provider.clone()], |_| {}).await?;
    let host = provider.host.lock().unwrap().clone().ok_or("provider did not start")?;
    let input = (0..1000).map(|counter| Reading { counter, value: counter as f64 }).collect::<Vec<_>>();
    let stream = dokime::Identifier::new("batch.samples")?;
    let started = Instant::now();
    for _ in 0..10 {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_micros() as i64;
        let rows = input
            .iter()
            .map(|row| dokime::execution::StampedRow {
                values: BTreeMap::new(),
                storage_payload: Some(row.encode()),
                published_unix_micros: now,
            })
            .collect();
        host.publish(None, &stream, rows).await?;
    }
    let seconds = started.elapsed().as_secs_f64();
    println!(
        "AUTHORING_BATCH rows=10000 batch=1000 rust_rows_s={:.0} seconds={seconds:.6} writer_build={}",
        10000.0 / seconds,
        dokime::storage::format::BUILD,
    );
    session.shutdown().await?;
    Ok(())
}

/* === Tests === */
