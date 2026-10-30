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

// Standard Library Imports
use std::{future::Future, time::Instant};

// Third Party Imports
use dokime::harmos::builtins::streams::Publish;
use dokime::harmos::{Context, Message, Operation, Response};

// Local Imports
use super::super::sampling::Source;
use super::{
    CpuLoadSettings, LoadControl, LoadHold, LoadSummary, LoadTarget,
    controller::{Controller, Summary},
    workers::Workers,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl LoadControl {
    pub(crate) fn measured(target: LoadTarget, measured: f64, duty: f64, generator_flops: f64) -> Self {
        let (mode, target_percent, measured_percent, target_flops, measured_flops) = match target {
            LoadTarget::Percent(value) => ("percent", Some(value), Some(measured), None, None),
            LoadTarget::Flops(value) => ("flops", None, None, Some(value), Some(measured)),
        };
        Self {
            mode: mode.into(),
            target_percent,
            measured_percent,
            target_flops,
            measured_flops,
            duty_percent: duty * 100.0,
            generator_flops,
        }
    }
}

impl Operation for LoadHold {
    type Error = dokime::Refusal;

    async fn run(self, ctx: Context<'_, Self>) -> Result<Response<LoadSummary>, Self::Error> {
        let logical = std::thread::available_parallelism().map_err(|error| dokime::Refusal::of(error.to_string()))?.get();
        dokime::plugin::refusal(self.worker_count(logical))?;
        let state = ctx.resource::<tokio::sync::Mutex<super::super::super::system::State>>();
        let source = tokio::select! {
            biased;
            () = ctx.cancelled() => return Ok(Response::new(Summary::new(self.target).finish())),
            state = state.lock() => dokime::plugin::refusal(state.collector.cpu_source())?,
        };
        let settings = ctx.resource::<CpuLoadSettings>().as_ref().clone();
        let summary = hold(self, settings, source, ctx.cancelled(), |row| {
            let request = Publish {
                stream: LoadControl::ID.to_owned(),
                schema: LoadControl::VERSION,
                sequence: None,
                published_unix_micros: None,
                context: None,
                payload: row.encode(),
                following: Vec::new(),
            };
            let ctx = &ctx;
            async move {
                ctx.operation(request).await.map_err(|error| crate::Error::Probe(error.to_string()))?;
                Ok(())
            }
        })
        .await;
        Ok(Response::new(dokime::plugin::refusal(summary)?))
    }
}

/* === Public Functions === */

pub(super) async fn hold<C, F, P>(
    request: LoadHold, settings: CpuLoadSettings, source: Source, cancel: C, mut publish: F,
) -> crate::Result<LoadSummary>
where
    C: Future<Output = ()>,
    F: FnMut(LoadControl) -> P,
    P: Future<Output = crate::Result<()>>,
{
    request.validate()?;
    settings.validate()?;
    let logical = std::thread::available_parallelism().map_err(|error| crate::Error::Probe(error.to_string()))?.get();
    let count = request.worker_count(logical)?;
    let capacity = source.read()?.capacity_gflops * 1_000_000_000.0;
    let scale = match request.target {
        LoadTarget::Percent(_) => 100.0 * count as f64 / logical as f64,
        LoadTarget::Flops(_) => capacity * count as f64 / logical as f64,
    };
    if !scale.is_finite() || scale <= 0.0 {
        return Err(crate::Error::Probe("CPU calibration capacity must be finite and positive".into()));
    }
    let mut summary = Summary::new(request.target);
    tokio::pin!(cancel);
    tokio::select! {
        biased;
        () = &mut cancel => return Ok(summary.finish()),
        () = std::future::ready(()) => {},
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
            () = &mut cancel => break Ok(()),
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
            () = &mut cancel => break Ok(()),
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
    use dokime::telemetry::TelemetryRow;
    use tokio_util::sync::CancellationToken;
    struct Publications(tokio::sync::mpsc::UnboundedSender<LoadControl>);
    impl harmos_sdk::operations::Caller for Publications {
        fn call(
            &self, target: harmos_sdk::operations::Target, request: Box<dyn std::any::Any + Send>,
        ) -> harmos_sdk::operations::CallFuture<'_> {
            use dokime::harmos::operations::OperationDeclaration;
            assert_eq!(target.id, Publish::ID);
            let request = request.downcast::<Publish>().unwrap();
            assert_eq!(request.stream, LoadControl::ID);
            assert_eq!(request.schema, LoadControl::VERSION);
            let row = LoadControl::decode(&request.payload).unwrap();
            self.0.send(row).unwrap();
            Box::pin(async { Ok(Response::new(()).into_erased()) })
        }
    }

    #[tokio::test]
    async fn public_dispatch_publishes_and_cancellation_returns_typed_summary() {
        use harmos_sdk::{
            __private::{RegistrationBatch, ResourceSet},
            ExecutionContext,
            operations::{CallContext, Calls},
        };
        use std::sync::Arc;
        let sampler = fixed(Load::measured(1.0, 25.0));
        let collector = crate::collectors::system::tests::with_cpu(sampler).unwrap();
        let state = crate::host::system::State { collector };
        let (published, mut received) = tokio::sync::mpsc::unbounded_channel();
        let mut resources = ResourceSet::new();
        resources.insert(Arc::new(tokio::sync::Mutex::new(state))).unwrap();
        resources.insert(Arc::new(DEFAULT)).unwrap();
        resources.insert(Arc::new(Calls(Arc::new(Publications(published))))).unwrap();
        let batch = RegistrationBatch::new().operation::<LoadHold>();
        let request = LoadHold {
            target: LoadTarget::Percent(40.0),
            threads: Some(1),
        };
        let mut instance = batch.operations[0].prepare_native(resources.clone(), Box::new(request.clone()), Box::new(())).unwrap();
        let context = ExecutionContext::new(resources, CallContext::default());
        let cancellation = context.cancellation.clone();
        instance.initialize(context.clone()).await.unwrap();
        let task = tokio::spawn(async move { instance.run(context).await });
        let row = received.recv().await.unwrap();
        assert_eq!(row.target_percent, Some(40.0));
        assert_eq!(row.measured_percent, Some(25.0));
        assert!(row.generator_flops.is_finite() && row.generator_flops >= 0.0);
        cancellation.request();
        let result = tokio::time::timeout(DEFAULT.settlement(), task).await.unwrap().unwrap().unwrap();
        let summary = result.restore::<LoadSummary>().unwrap();
        assert_eq!(summary.target, request.target);
        assert_eq!(summary.mean, 25.0);
        assert!(summary.samples > 0);
        assert!(summary.duration_s > 0.0);
    }

    #[tokio::test]
    async fn cancellation_stops_workers_and_returns_summary_promptly() {
        for target in [LoadTarget::Percent(40.0), LoadTarget::Flops(1000.0)] {
            let sampler = fixed(Load::measured(1.0, 25.0));
            let cancel = CancellationToken::new();
            let stopping = cancel.clone();
            let (published, mut received) = tokio::sync::mpsc::unbounded_channel();
            let task = tokio::spawn(hold(
                LoadHold { target, threads: Some(1) },
                DEFAULT,
                sampler.source(),
                cancel.cancelled_owned(),
                move |row| {
                    let sent = published.send(row).map_err(|error| crate::Error::Probe(error.to_string()));
                    async { sent }
                },
            ));
            let row = tokio::time::timeout(DEFAULT.settlement(), received.recv()).await.unwrap().unwrap();
            assert!(row.generator_flops.is_finite());
            stopping.cancel();
            let summary = tokio::time::timeout(DEFAULT.settlement(), task).await.unwrap().unwrap().unwrap();
            assert_eq!(summary.target, target);
            assert!(summary.duration_s > 0.0);
            assert!(summary.samples >= 1);
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
            cancel.cancelled_owned(),
            |_| async { panic!("cancelled hold published a period") },
        )
        .await
        .unwrap();
        assert_eq!(summary.duration_s, 0.0);
    }
    #[test]
    fn evidence_preserves_units_and_target_mode() {
        for target in [LoadTarget::Percent(30.0), LoadTarget::Flops(123.0)] {
            let row = LoadControl::measured(target, 20.0, 0.25, 123.0);
            assert_eq!(row.duty_percent, 25.0);
            assert_eq!(row.target_percent.is_some(), matches!(target, LoadTarget::Percent(_)));
            assert_eq!(row, LoadControl::decode(&row.encode()).unwrap());
        }
        for column in LoadControl::columns().into_iter().filter(|column| column.unit.is_some()) {
            let unit = column.unit.unwrap();
            assert!(column.metadata.label.ends_with(&format!("[{}]", unit.symbol)));
        }
        assert_eq!(LoadTarget::Percent(25.0).approximate_flops(4.0), 1_000_000_000.0);
        assert_eq!(LoadTarget::Flops(1_000_000_000.0).approximate_percent(4.0), 25.0);
    }
}
