/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use super::super::sampling::Source;
use super::{
    CpuLoadSettings, LoadControl, LoadHold, LoadSummary, LoadTarget,
    controller::{Controller, Summary},
    workers::Workers,
};
use std::{future::Future, time::Instant};
use tokio_util::sync::CancellationToken;

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

pub(super) async fn hold<F, P>(
    request: LoadHold, settings: CpuLoadSettings, source: Source, cancel: CancellationToken, mut publish: F,
) -> crate::Result<LoadSummary>
where
    F: FnMut(LoadControl) -> P,
    P: Future<Output = crate::Result<()>>,
{
    request.validate()?;
    settings.validate()?;
    let logical = std::thread::available_parallelism().map_err(|error| crate::Error::Probe(error.to_string()))?.get();
    let count = request.threads.map(usize::from).unwrap_or(logical);
    let capacity = source.read()?.capacity_gflops * 1_000_000_000.0;
    let scale = match request.target {
        LoadTarget::Percent(_) => 100.0 * count as f64 / logical as f64,
        LoadTarget::Flops(_) => capacity * count as f64 / logical as f64,
    };
    if !scale.is_finite() || scale <= 0.0 {
        return Err(crate::Error::Probe("CPU calibration capacity must be finite and positive".into()));
    }
    let mut summary = Summary::new(request.target);
    if cancel.is_cancelled() {
        return Ok(summary.finish());
    }
    let mut controller = Controller::new(settings.clone());
    let mut workers = Workers::start(count, &settings)?;
    let initial = measured(request.target, source.read()?.utilization_percent, 0.0);
    let mut duty = controller.update(normalized_error(request.target, initial, scale), settings.period().as_secs_f64());
    workers.duty(duty);
    let mut previous = Instant::now();
    let mut operations = workers.operations();
    let outcome = loop {
        tokio::select! {
            biased;
            () = cancel.cancelled() => break Ok(()),
            () = tokio::time::sleep_until((previous + settings.period()).into()) => {},
        }
        let now = Instant::now();
        let elapsed = now.duration_since(previous).as_secs_f64();
        let completed = workers.operations();
        let flops = completed.wrapping_sub(operations) as f64 / elapsed;
        let load = match source.read() {
            Ok(load) => load,
            Err(error) => break Err(error),
        };
        let value = measured(request.target, load.utilization_percent, flops);
        summary.record(value, elapsed);
        let row = LoadControl::measured(request.target, value, duty, flops);
        duty = controller.update(normalized_error(request.target, value, scale), elapsed);
        workers.duty(duty);
        previous = now;
        operations = completed;
        let published = tokio::select! {
            biased;
            () = cancel.cancelled() => break Ok(()),
            result = publish(row) => result,
        };
        if let Err(error) = published {
            break Err(error);
        }
    };
    let stopping = Instant::now();
    workers.stop()?;
    if stopping.elapsed() > settings.settlement() {
        return Err(crate::Error::Probe("CPU workers exceeded the configured settlement budget".into()));
    }
    outcome?;
    let elapsed = previous.elapsed().as_secs_f64();
    let flops = workers.operations().wrapping_sub(operations) as f64 / elapsed;
    summary.record(measured(request.target, source.read()?.utilization_percent, flops), elapsed);
    Ok(summary.finish())
}

/* === Private Functions === */

fn normalized_error(target: LoadTarget, measured: f64, scale: f64) -> f64 {
    (target.value() / scale - measured / scale).clamp(-1.0, 1.0)
}

fn measured(target: LoadTarget, utilization: f64, flops: f64) -> f64 {
    match target {
        LoadTarget::Percent(_) => utilization,
        LoadTarget::Flops(_) => flops,
    }
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::super::super::{Load, sampling::tests::fixed};
    use super::super::config::DEFAULT;
    use super::*;
    #[tokio::test]
    async fn cancellation_stops_workers_and_returns_summary_promptly() {
        for target in [LoadTarget::Percent(40.0), LoadTarget::Flops(1000.0)] {
            let sampler = fixed(Load::measured(1.0, 25.0));
            let cancel = CancellationToken::new();
            let stopping = cancel.clone();
            let (published, mut received) = tokio::sync::mpsc::unbounded_channel();
            let task = tokio::spawn(hold(LoadHold { target, threads: Some(1) }, DEFAULT, sampler.source(), cancel, move |row| {
                let sent = published.send(row).map_err(|error| crate::Error::Probe(error.to_string()));
                async { sent }
            }));
            let row = tokio::time::timeout(DEFAULT.settlement(), received.recv()).await.unwrap().unwrap();
            assert!(row.generator_flops.is_finite());
            stopping.cancel();
            let summary = tokio::time::timeout(DEFAULT.settlement(), task).await.unwrap().unwrap().unwrap();
            assert_eq!(summary.target, target);
            assert!(summary.duration_s > 0.0);
            assert!(summary.mean.is_finite() && summary.error_rms.is_finite());
            if matches!(target, LoadTarget::Percent(_)) {
                assert_eq!(summary.mean, 25.0);
                assert!((summary.error_rms - 15.0).abs() < 0.000_001);
            }
        }
    }
    #[tokio::test]
    async fn already_cancelled_hold_starts_no_workers() {
        let sampler = fixed(Load::measured(1.0, 25.0));
        let cancel = CancellationToken::new();
        cancel.cancel();
        let summary = hold(
            LoadHold {
                target: LoadTarget::Percent(40.0),
                threads: Some(1),
            },
            DEFAULT,
            sampler.source(),
            cancel,
            |_| async { panic!("cancelled hold published a period") },
        )
        .await
        .unwrap();
        assert_eq!(summary.duration_s, 0.0);
    }
}
