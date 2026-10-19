/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Start One Execution And Drain Its Requested Observations

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::time::Duration;

// Third Party Imports

// Local Imports
use super::{ResponseStream, Result, TestExecutionHandle, poll_interval};
use dokime_domain::model::{evidence::wire::RecordPage, execution::ExecutionMode, telemetry::wire::TelemetrySamplesReply};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */
pub type RecordCallback<'a> = &'a mut (dyn FnMut(&RecordPage) -> Result<()> + Send);
pub type SampleCallback<'a> = &'a mut (dyn FnMut(&TelemetrySamplesReply) -> Result<()> + Send);

/* === Structs === */
// Listener Selection Affects Observation Only, Never Captured Evidence.
#[derive(Clone, Debug)]
pub struct RunOptions {
    pub mode: ExecutionMode,

    // None Disables Records; An Empty Vector Listens To All Record Types.
    pub record_types: Option<Vec<String>>,

    // None Disables Samples; An Empty Vector Listens To All Streams.
    pub streams: Option<Vec<String>>,
    pub timeout: Option<Duration>,
    pub interval: Option<Duration>,
    pub page_size: u32,
}

// Callbacks Run In The Caller's Task; Errors Stop Observation And Are Returned.
#[derive(Default)]
pub struct RunCallbacks<'a> {
    pub records: Option<RecordCallback<'a>>,
    pub samples: Option<SampleCallback<'a>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Default for RunOptions {
    fn default() -> Self {
        Self { mode: ExecutionMode::Automatic, record_types: None, streams: None, timeout: None, interval: None, page_size: 1000 }
    }
}

impl RunOptions {
    // Validate Every Local Option Before Any Execution Can Be Admitted.
    pub fn validate(&self) -> Result<()> {
        poll_interval(self.interval)?;
        super::listeners::sample_interval(self.page_size, self.interval)?;
        if let Some(patterns) = &self.record_types {
            super::listeners::validate_filters(patterns)?;
        }
        if let Some(patterns) = &self.streams {
            super::listeners::validate_filters(patterns)?;
        }
        Ok(())
    }
}

impl TestExecutionHandle {
    // Start Idempotently, Then Wait For The Saved Seal And Final Listener Drain.
    // Dropping This Future, Callback Failure Or Transport Failure Never Stops Work.
    pub async fn run(
        &self, options: RunOptions, mut callbacks: RunCallbacks<'_>,
    ) -> Result<dokime_domain::model::history::wire::CheckpointDocument> {
        options.validate()?;
        let operation = async {
            self.start(options.mode).await?;
            let records = options.record_types.map(|filters| self.records(filters, true, options.interval)).transpose()?;
            let samples = options.streams.map(|filters| self.samples(filters, true, options.page_size, options.interval)).transpose()?;
            let record_reader = drain(records.as_ref(), callbacks.records.take());
            let sample_reader = drain(samples.as_ref(), callbacks.samples.take());
            let result = tokio::try_join!(self.wait(None, options.interval), record_reader, sample_reader);
            if let Some(reader) = records {
                reader.close().await;
            }
            if let Some(reader) = samples {
                reader.close().await;
            }
            result.map(|(outcome, (), ())| outcome)
        };
        match options.timeout {
            Some(timeout) => tokio::time::timeout(timeout, operation).await.map_err(|source| super::Error::WaitTimedOut { source })?,
            None => operation.await,
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */
async fn drain<T: Send + 'static>(
    reader: Option<&ResponseStream<T>>, mut callback: Option<&mut (dyn FnMut(&T) -> Result<()> + Send)>,
) -> Result<()> {
    let Some(reader) = reader else { return Ok(()) };
    while let Some(page) = reader.message().await? {
        let Some(callback) = callback.as_mut() else { continue };
        callback(&page)?;
    }
    Ok(())
}

/* === Tests === */
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_observation_options_are_refused_before_start() {
        for options in [
            RunOptions { record_types: Some(vec!["bad[regex]".into()]), ..Default::default() },
            RunOptions { streams: Some(vec!["obc.*".into(); 65]), ..Default::default() },
            RunOptions { interval: Some(Duration::ZERO), ..Default::default() },
            RunOptions { page_size: 0, ..Default::default() },
            RunOptions { page_size: 5001, ..Default::default() },
        ] {
            let error = options.validate().unwrap_err();
            assert!(matches!(error, super::super::Error::Rpc(status) if status.code() == tonic::Code::InvalidArgument));
        }
        RunOptions { record_types: Some(Vec::new()), streams: Some(Vec::new()), ..Default::default() }.validate().unwrap();
        RunOptions { record_types: Some(vec!["obc.*".into(), "dokime.?".into()]), ..Default::default() }.validate().unwrap();
    }
}
