/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Audio input recording through cpal into a 16-bit PCM WAV file. The device
// stream is not `Send`, so one dedicated thread owns it from open to close; a
// supervisor task seals the file when asked, or when its execution closes.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
        mpsc,
    },
    thread::JoinHandle,
    time::{Duration, Instant},
};

// Third Party Imports
use tempfile::TempPath;
use tokio::sync::oneshot;

// Local Imports
use super::captures::SealRequest;
use crate::{Error, Result, RuntimeHandle};
use dokime::artifacts::ArtifactRecord;
use dokime::core::RecordScope;

/* === Constants === */
const WAV_HEADER_BYTES: u64 = 44;
const BYTES_PER_SAMPLE: u64 = 2;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
// One running recording. `finish` seals it as an artifact, and so does closing
// its execution; dropping it unsettled stops the device and discards the file.
pub struct AudioRecording {
    filename: String,
    started: Instant,
    samples: Arc<AtomicU64>,
    command: Option<oneshot::Sender<Command>>,
    outcome: Option<oneshot::Receiver<Result<ArtifactRecord>>>,
}

struct Recorder {
    stop: mpsc::Sender<()>,
    thread: JoinHandle<Result<()>>,
}

struct Supervisor {
    runtime: RuntimeHandle,
    scope: RecordScope,
    filename: String,
    recorder: Recorder,
    file: TempPath,
    command: oneshot::Receiver<Command>,
    seal: oneshot::Receiver<oneshot::Sender<()>>,
    outcome: oneshot::Sender<Result<ArtifactRecord>>,
    registration: u64,
}

/* === Enums === */
enum Command {
    Finish,
    Discard,
}

/* === Traits === */

/* === Implementations === */
impl AudioRecording {
    pub(super) async fn start(
        runtime: RuntimeHandle,
        scope: RecordScope,
        filename: String,
        input: Option<u32>,
    ) -> Result<Self> {
        let execution = scope
            .test_execution_id
            .clone()
            .ok_or_else(|| Error::Operation("audio recording needs an execution".into()))?;
        let file = tempfile::Builder::new()
            .prefix("dokime-audio-")
            .suffix(".wav")
            .tempfile()
            .map_err(|source| Error::Operation(format!("audio recording file: {source}")))?
            .into_temp_path();
        let samples = Arc::new(AtomicU64::new(0));

        // Open The Device On Its Own Thread
        let (ready, opened) = oneshot::channel();
        let (stop, stopped) = mpsc::channel();
        let path = file.to_path_buf();
        let counter = samples.clone();
        let thread = std::thread::Builder::new()
            .name("dokime-audio".into())
            .spawn(move || device::record(&path, input, counter, ready, stopped))
            .map_err(|source| Error::Operation(format!("audio recording thread: {source}")))?;
        let recorder = Recorder { stop, thread };
        let started = opened
            .await
            .unwrap_or_else(|_| Err(Error::Operation("audio recording thread exited".into())));
        if let Err(source) = started {
            recorder.close().await?;
            return Err(source);
        }

        // Supervise Until Finished, Discarded Or Sealed By The Execution
        let (seal_request, seal) = oneshot::channel();
        let registration = runtime
            .execution_storage
            .captures
            .register(&execution, seal_request as SealRequest);
        let (command_sender, command) = oneshot::channel();
        let (outcome_sender, outcome) = oneshot::channel();
        tokio::spawn(
            Supervisor {
                runtime,
                scope,
                filename: filename.clone(),
                recorder,
                file,
                command,
                seal,
                outcome: outcome_sender,
                registration,
            }
            .run(),
        );

        Ok(Self {
            filename,
            started: Instant::now(),
            samples,
            command: Some(command_sender),
            outcome: Some(outcome),
        })
    }

    pub fn filename(&self) -> &str {
        &self.filename
    }

    pub fn elapsed(&self) -> Duration {
        self.started.elapsed()
    }

    // The WAV file size the recording has reached so far.
    pub fn size_bytes(&self) -> u64 {
        WAV_HEADER_BYTES + self.samples.load(Ordering::Relaxed) * BYTES_PER_SAMPLE
    }

    // Stops the device, finalizes the WAV file and stores it as an artifact.
    pub async fn finish(&mut self) -> Result<ArtifactRecord> {
        if let Some(command) = self.command.take() {
            let _ = command.send(Command::Finish);
        }
        self.settled().await
    }

    // Resolves once the recording is sealed, by `finish` or by its execution
    // closing. Cancel-safe; it yields the outcome exactly once.
    pub async fn settled(&mut self) -> Result<ArtifactRecord> {
        let outcome = self
            .outcome
            .as_mut()
            .ok_or_else(|| Error::Operation("audio recording already settled".into()))?;
        let settled = outcome
            .await
            .unwrap_or_else(|_| Err(Error::Operation("audio recording supervisor exited".into())));
        self.outcome = None;
        self.command = None;
        settled
    }
}

impl Drop for AudioRecording {
    fn drop(&mut self) {
        if let Some(command) = self.command.take() {
            let _ = command.send(Command::Discard);
        }
    }
}

impl Supervisor {
    async fn run(mut self) {
        // Wait For A Command Or The Execution Closing
        let mut sealed = None;
        let finish = tokio::select! {
            command = &mut self.command => matches!(command, Ok(Command::Finish)),
            request = &mut self.seal => match request {
                Ok(done) => {
                    sealed = Some(done);
                    true
                }
                Err(_) => matches!((&mut self.command).await, Ok(Command::Finish)),
            },
        };

        // Close The Device, Then Store Or Discard The File
        let closed = self.recorder.close().await;
        let outcome = match closed {
            Err(source) => Err(source),
            Ok(()) if !finish => Err(Error::Cancelled {
                code: "media_discarded".into(),
                message: "audio recording discarded".into(),
            }),
            Ok(()) => {
                self.runtime
                    .upload_artifact_file(
                        self.scope.clone(),
                        &self.file,
                        self.filename.clone(),
                        Some("audio/wav".into()),
                    )
                    .await
            }
        };
        drop(self.file);
        if let Some(execution) = &self.scope.test_execution_id {
            self.runtime
                .execution_storage
                .captures
                .release(execution, self.registration);
        }
        if let Err(source) = &outcome
            && finish
        {
            tracing::warn!(error = %source, filename = %self.filename, "audio recording failed to seal");
        }
        let _ = self.outcome.send(outcome);
        if let Some(done) = sealed {
            let _ = done.send(());
        }
    }
}

impl Recorder {
    async fn close(self) -> Result<()> {
        let _ = self.stop.send(());
        let thread = self.thread;
        tokio::task::spawn_blocking(move || thread.join())
            .await?
            .map_err(|_| Error::Operation("audio recording thread panicked".into()))?
    }
}

/* === Public Functions === */

/* === Private Functions === */
#[cfg(feature = "media-native")]
mod device {
    use std::{
        path::Path,
        sync::{
            Arc, Mutex,
            atomic::{AtomicU64, Ordering},
            mpsc,
        },
    };

    use cpal::{
        FromSample, SampleFormat, SizedSample,
        traits::{DeviceTrait, HostTrait, StreamTrait},
    };
    use tokio::sync::oneshot;

    use super::super::unavailable;
    use crate::{Error, Result};

    type Writer = Arc<Mutex<Option<hound::WavWriter<std::io::BufWriter<std::fs::File>>>>>;
    type Failure = Arc<Mutex<Option<String>>>;

    // Owns the device stream until stopped, then finalizes the WAV header.
    pub(super) fn record(
        path: &Path,
        input: Option<u32>,
        samples: Arc<AtomicU64>,
        ready: oneshot::Sender<Result<()>>,
        stop: mpsc::Receiver<()>,
    ) -> Result<()> {
        let writer: Writer = Arc::new(Mutex::new(None));
        let failure: Failure = Arc::new(Mutex::new(None));
        let stream = match open(path, input, &writer, &failure, samples) {
            Ok(stream) => stream,
            Err(source) => {
                let _ = ready.send(Err(source));
                return Ok(());
            }
        };
        let _ = ready.send(Ok(()));

        // Record Until Stopped Or Abandoned
        let _ = stop.recv();
        drop(stream);

        // Finalize The Header Over The Written Samples
        let finished = writer
            .lock()
            .map_err(|_| Error::Operation("audio writer lock poisoned".into()))?
            .take();
        if let Some(finished) = finished {
            finished
                .finalize()
                .map_err(|source| Error::Operation(format!("audio file finalization: {source}")))?;
        }
        let failed = failure
            .lock()
            .map_err(|_| Error::Operation("audio failure lock poisoned".into()))?
            .take();
        match failed {
            Some(message) => Err(unavailable(format!("audio recording failed: {message}"))),
            None => Ok(()),
        }
    }

    fn open(
        path: &Path,
        input: Option<u32>,
        writer: &Writer,
        failure: &Failure,
        samples: Arc<AtomicU64>,
    ) -> Result<cpal::Stream> {
        // Select The Input Device
        let host = cpal::default_host();
        let device = match input {
            None => host.default_input_device(),
            Some(index) => host
                .input_devices()
                .map_err(|source| unavailable(format!("audio inputs are unavailable: {source}")))?
                .nth(index as usize),
        }
        .ok_or_else(|| unavailable("no audio input device is available"))?;
        let supported = device
            .default_input_config()
            .map_err(|source| unavailable(format!("audio input configuration failed: {source}")))?;

        // Create The WAV File
        let spec = hound::WavSpec {
            channels: supported.channels(),
            sample_rate: supported.sample_rate(),
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let created = hound::WavWriter::create(path, spec)
            .map_err(|source| Error::Operation(format!("audio file creation: {source}")))?;
        *writer
            .lock()
            .map_err(|_| Error::Operation("audio writer lock poisoned".into()))? = Some(created);

        // Start The Stream In Its Native Sample Format
        let config = supported.config();
        let parts = (device, config, writer.clone(), failure.clone(), samples);
        let stream = match supported.sample_format() {
            SampleFormat::I8 => build::<i8>(parts),
            SampleFormat::I16 => build::<i16>(parts),
            SampleFormat::I32 => build::<i32>(parts),
            SampleFormat::I64 => build::<i64>(parts),
            SampleFormat::U8 => build::<u8>(parts),
            SampleFormat::U16 => build::<u16>(parts),
            SampleFormat::U32 => build::<u32>(parts),
            SampleFormat::U64 => build::<u64>(parts),
            SampleFormat::F32 => build::<f32>(parts),
            SampleFormat::F64 => build::<f64>(parts),
            other => Err(unavailable(format!(
                "unsupported audio sample format {other}"
            ))),
        }?;
        stream
            .play()
            .map_err(|source| unavailable(format!("audio stream failed to start: {source}")))?;
        Ok(stream)
    }

    fn build<T>(
        (device, config, writer, failure, samples): (
            cpal::Device,
            cpal::StreamConfig,
            Writer,
            Failure,
            Arc<AtomicU64>,
        ),
    ) -> Result<cpal::Stream>
    where
        T: SizedSample,
        i16: FromSample<T>,
    {
        let written = failure.clone();
        device
            .build_input_stream(
                config,
                move |data: &[T], _: &cpal::InputCallbackInfo| {
                    let Ok(mut guard) = writer.lock() else {
                        return;
                    };
                    let Some(output) = guard.as_mut() else {
                        return;
                    };
                    for sample in data {
                        if let Err(source) = output.write_sample(sample.to_sample::<i16>()) {
                            fail(&written, source);
                            *guard = None;
                            return;
                        }
                    }
                    samples.fetch_add(data.len() as u64, Ordering::Relaxed);
                },
                move |source| fail(&failure, source),
                None,
            )
            .map_err(|source| unavailable(format!("audio stream failed to open: {source}")))
    }

    fn fail(failure: &Failure, source: impl std::fmt::Display) {
        if let Ok(mut slot) = failure.lock() {
            slot.get_or_insert_with(|| source.to_string());
        }
    }
}

#[cfg(not(feature = "media-native"))]
mod device {
    use std::{
        path::Path,
        sync::{Arc, atomic::AtomicU64, mpsc},
    };

    use tokio::sync::oneshot;

    use crate::Result;

    pub(super) fn record(
        _path: &Path,
        _input: Option<u32>,
        _samples: Arc<AtomicU64>,
        ready: oneshot::Sender<Result<()>>,
        _stop: mpsc::Receiver<()>,
    ) -> Result<()> {
        let _ = ready.send(Err(super::super::not_compiled()));
        Ok(())
    }
}

/* === Tests === */
