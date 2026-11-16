/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Charger Suites: One Folder Entry per Procedure Family

/* === Modules === */

pub mod benchmark;
pub mod steady_state;
pub mod transient;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use sha2::{Digest, Sha256};

// Local Imports
use crate::device::Bench;
use crate::device::Supply;
use crate::settings;
use dokime::Error;
use dokime::plugin::{Context, HostEvidenceTraceabilityGetOptions, Suite, Template};
use dokime::{FileMetadata, Metadata, Parameter, Result, TraceabilityEntry, Value};

/* === Constants === */

// Bench Bounds for Simulated Durations and Model Sample Intervals.
const SHORTEST_S: f64 = 0.0001;
const FINEST_STEP_S: f64 = 0.0001;
const COARSEST_STEP_S: f64 = 1.0;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn catalog() -> [Suite; 3] {
    [steady_state::suite(), transient::suite(), benchmark::suite()]
}

/// Publish the Execution Bench Before Reading Settings Because Sampling Can Start During the Read.
pub async fn open(ctx: &Context, supply: Supply, step_s: Option<f64>) -> Result<()> {
    ctx.set(Bench::new(supply, step_s)?);
    let interval = settings::sample_interval(ctx).await?;
    ctx.get::<Bench>()?.configure_interval(interval, step_s.is_none());
    Ok(())
}

/// Wait for Samples Due on the Host Clock; Each Sleep Settles Its Due Windows.
pub async fn wait(ctx: &Context, bench: &Bench, samples: u64) -> Result<()> {
    let target = bench.index() + samples;
    loop {
        let remaining = target.saturating_sub(bench.index());
        if remaining == 0 {
            return Ok(());
        }
        let count = u32::try_from(remaining).map_err(|_| Error::Invalid("Wait for Samples: Shorten the Run".into()))?;
        let duration = bench.interval().checked_mul(count).ok_or_else(|| Error::Invalid("Wait for Samples: Shorten the Run".into()))?;
        ctx.sleep(duration).await?;
    }
}

pub fn captures(ctx: &Context, stream: &str) -> bool {
    !ctx.execution_id().is_empty() && ctx.test_case().is_ok_and(|case| case.streams.iter().any(|captured| captured == stream))
}

/// Declare Parameters in Order on a Template.
pub fn parameters(mut template: Template, parameters: impl IntoIterator<Item = Parameter>) -> Template {
    for parameter in parameters {
        template = template.parameter(parameter);
    }
    template
}

/// The AC Supply Every Charger Procedure Applies, With Nominal Defaults.
pub fn supply() -> Vec<Parameter> {
    vec![
        Parameter::number("ac_voltage_v", "AC RMS Voltage").unit("V").default(230.0).range(0.0..=300.0).build(),
        Parameter::number("load_ohm", "Load Resistance").unit("ohm").default(100.0).range(0.1..).build(),
        Parameter::number("reactance_ohm", "Load Reactance").unit("ohm").default(0.0).build(),
        Parameter::number("frequency_hz", "Supply Frequency").unit("Hz").default(50.0).range(1.0..=1000.0).build(),
        Parameter::number("current_limit_a", "RMS Current Limit").unit("A").default(5.0).range(0.001..).build(),
    ]
}

/// The Observation Window and Model Sample Interval of a Charger Procedure.
pub fn timing(duration_label: &str, duration_s: f64) -> [Parameter; 2] {
    [
        Parameter::number("duration_s", duration_label).unit("s").default(duration_s).range(SHORTEST_S..).build(),
        Parameter::number("sample_interval_s", "Simulation Sample Interval")
            .unit("s")
            .default(0.001)
            .range(FINEST_STEP_S..=COARSEST_STEP_S)
            .build(),
    ]
}

/// Read the Supply Parameters of the Frozen Case.
pub fn read_supply(ctx: &Context) -> Result<Supply> {
    Ok(Supply {
        voltage_rms_v: ctx.parameter("ac_voltage_v")?,
        load_ohm: ctx.parameter("load_ohm")?,
        frequency_hz: ctx.parameter("frequency_hz")?,
        reactance_ohm: ctx.parameter("reactance_ohm")?,
        current_limit_a: ctx.parameter("current_limit_a")?,
        ..Supply::default()
    })
}

/// The Measured Sample Count of a Stream, From Saved Evidence.
pub async fn sample_count(ctx: &Context, stream: &str, step: &str) -> Result<u64> {
    let statistics = ctx.statistics(stream, "voltage_v", step).await?;
    Ok(statistics.statistics.map_or(0, |statistics| statistics.count))
}

/// Upload Bytes Into This Execution's Scope and Attach Them as a Named Artifact.
pub async fn attach(ctx: &Context, filename: &str, content_type: &str, bytes: &[u8]) -> Result<()> {
    let uploads = ctx.host().files.uploads;
    let sha256 = Sha256::digest(bytes).to_vec();
    let metadata = FileMetadata {
        scope: Some(ctx.execution_id().into()),
        media_type: content_type.into(),
        expected_length: Some(bytes.len() as u64),
        expected_sha256: Some(sha256.clone()),
        ..Default::default()
    };
    let created = uploads.create(metadata).await?;
    let upload = created.upload.ok_or_else(|| Error::Invalid(format!("Attach {filename}: Missing Upload Writer")))?;
    let chunk = usize::try_from(created.chunk_bytes.max(1)).unwrap_or(usize::MAX);
    for (index, part) in bytes.chunks(chunk).enumerate() {
        uploads.append(upload.clone(), (index * chunk) as u64, part.to_vec()).await?;
    }
    let published = uploads.publish(upload, bytes.len() as u64, sha256).await?;
    let file = published.upload.ok_or_else(|| Error::Invalid(format!("Attach {filename}: Missing Published File")))?;
    ctx.host().content.artifacts.create(file, filename.into(), content_type.into()).await?;
    Ok(())
}

/// Replace One Execution Traceability Entry at Its Current Revision.
pub async fn trace(ctx: &Context, id: &str, label: &str, value: Value, unit: &str, source: &str) -> Result<()> {
    let traceability = ctx.host().evidence.traceability;
    let options = HostEvidenceTraceabilityGetOptions {
        entry: Some(id.into()),
        ..Default::default()
    };
    let current = traceability.get_with(options).await?;
    let entry = TraceabilityEntry {
        metadata: Some(Metadata::new(id, label)),
        value: Some(value),
        source: source.into(),
        unit: unit.into(),
    };
    traceability.set(entry, current.revision, id.into()).await?;
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
