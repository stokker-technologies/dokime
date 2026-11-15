/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// The Execution-Owned Testbench: One Charger, Its Model Clock and Its Observers

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::{Mutex, MutexGuard};
use std::time::Duration;

// Third Party Imports
use tokio::sync::Notify;

// Local Imports
use crate::device::{Charger, Power, Supply};
use crate::measurement::{Measurement, Survey};
use crate::settings;
use dokime::plugin::Context;
use dokime::{Error, Result};

/* === Constants === */

// Every Charger Run Stays Within This Many Samples.
const MOST_SAMPLES: f64 = 100_000.0;

// The Thermal Survey Visits Four Modules With Eight Sensors Each.
const SURVEY_POINTS: u64 = 32;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// One Execution's Bench. Observers Sample It on Host Ticks; Steps Drive It.
pub struct Testbench {
    state: Mutex<State>,
    tick: Mutex<Duration>,
    published: Notify,
}

pub struct State {
    pub charger: Charger,
    /// Samples Published So Far; Model Time Is `origin_s + index * step_s`.
    pub index: u64,
    pub step_s: f64,
    pub origin_s: f64,
    /// The Phase Label Steps Assign to the Samples That Follow.
    pub label: &'static str,
    /// Start-Up Checkpoint and Total Sample Counts, When the Sequence Labels Phases.
    pub sequence: Option<(u64, u64)>,
    survey: u64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Testbench {
    /// Open the Bench for This Execution With the Plugin's Configured Sample Interval.
    ///
    /// `step_s` is the model time one sample advances; `None` follows the host clock.
    pub async fn open(ctx: &Context, supply: Supply, step_s: Option<f64>) -> Result<()> {
        if step_s.is_some_and(|step_s| !step_s.is_finite() || step_s <= 0.0) {
            return Err(Error::Invalid("Open Testbench: Choose a Positive Sample Interval".into()));
        }
        // Observers May Tick Before the Settings Read Returns, So the Bench Exists First
        let tick = Duration::from_millis(1);
        ctx.set(Self {
            state: Mutex::new(State {
                charger: Charger::new(supply)?,
                index: 0,
                step_s: step_s.unwrap_or(tick.as_secs_f64()),
                origin_s: 0.0,
                label: "idle",
                sequence: None,
                survey: 0,
            }),
            tick: Mutex::new(tick),
            published: Notify::new(),
        });
        let tick = settings::sample_interval(ctx).await?;
        let bench = ctx.get::<Self>()?;
        *bench.tick.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = tick;
        if step_s.is_none() {
            bench.state().step_s = tick.as_secs_f64();
        }
        Ok(())
    }

    pub fn state(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// The Number of Samples a Model Duration Spans on This Bench.
    pub fn samples_for(&self, duration_s: f64) -> Result<u64> {
        let samples = (duration_s / self.state().step_s).ceil();
        if !samples.is_finite() || !(1.0..=MOST_SAMPLES).contains(&samples) {
            return Err(Error::Invalid(format!(
                "Size the Run: {duration_s} s Spans {samples} Samples; Choose a Duration and Interval Within 1 to {MOST_SAMPLES} Samples"
            )));
        }
        Ok(samples as u64)
    }

    /// Advance Host Time Until the Observers Have Published `samples` More Ticks.
    pub async fn wait(&self, ctx: &Context, samples: u64) -> Result<()> {
        if samples == 0 {
            return Ok(());
        }
        let target = self.state().index + samples;
        let tick = *self.tick.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let total = tick.checked_mul(u32::try_from(samples).map_err(|_| Error::Invalid("Wait: Too Many Samples".into()))?);
        ctx.sleep(total.ok_or_else(|| Error::Invalid("Wait: Duration Overflow".into()))?).await?;
        // The Last Tick Fires With the Sleep; Await Its Sample Without Advancing Time
        loop {
            let published = self.published.notified();
            if self.state().index >= target {
                return Ok(());
            }
            published.await;
        }
    }

    /// Count One Published Sample and Wake Any Step Waiting for It.
    pub fn advance(&self) {
        self.state().index += 1;
        self.published.notify_waiters();
    }
}

impl State {
    pub fn now_s(&self) -> f64 {
        self.origin_s + self.index as f64 * self.step_s
    }
}

/* === Public Functions === */

/// Whether This Execution's Case Captures a Stream; the Host Ticks Every Plugin Observer.
pub fn captures(ctx: &Context, stream: &str) -> bool {
    !ctx.execution_id().is_empty() && ctx.test_case().is_ok_and(|case| case.streams.iter().any(|captured| captured == stream))
}

/// Observe One Charger Sample at the Bench's Model Time.
pub async fn measure(ctx: &Context, stream: &str) -> Result<Measurement> {
    if !captures(ctx, stream) {
        return Ok(Measurement::default());
    }
    let bench = ctx.get::<Testbench>()?;
    let mut state = bench.state();
    let index = state.index;
    let elapsed_s = state.now_s();
    let reading = state.charger.sample(elapsed_s);
    let phase = match state.sequence {
        _ if state.charger.power == Power::Faulted => "fault",
        Some(_) if index == 0 => "start",
        Some((_, total)) if index + 1 == total => "complete",
        Some((checkpoint, _)) if index + 1 == checkpoint => "checkpoint",
        Some((checkpoint, _)) if index < checkpoint => "before_prompt",
        Some(_) => "after_prompt",
        None => state.label,
    };
    drop(state);
    bench.advance();
    Ok(Measurement::new(index, reading, phase))
}

/// Observe One Point of the Thermal Channel Survey; Powered Modules Warm Up Sweep by Sweep.
pub async fn survey(ctx: &Context, stream: &str) -> Result<Survey> {
    if !captures(ctx, stream) {
        return Ok(Survey::default());
    }
    let bench = ctx.get::<Testbench>()?;
    let mut state = bench.state();
    let point = state.survey;
    state.survey += 1;
    let sweep = (point / SURVEY_POINTS) as f64;
    let module = ((point % SURVEY_POINTS) / 8) as f64;
    let sensor = (point % 8) as f64;
    let running = state.charger.power == Power::Running;
    let load = if running { state.charger.reading(state.now_s()).input_power_w / 600.0 } else { 0.0 };
    Ok(Survey {
        sweep,
        module,
        sensor,
        rise_k: module * 1.5 + sensor * 0.4 + 12.0 * load * (1.0 - (-sweep / 20.0).exp()),
    })
}

/* === Private Functions === */

/* === Tests === */
