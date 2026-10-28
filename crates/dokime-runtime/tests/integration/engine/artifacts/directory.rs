/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::{
    io,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{Duration, SystemTime},
};

use dokime_runtime::{
    artifacts::{ArtifactSink, Artifacts},
    definition::{Step, TemplateContext},
    execution::*,
    transport::recording::RecordingSink,
    transport::{MAX_ARTIFACT_BYTES, RunExecution, StepAdmission},
};

const GROWTH_FLOOR: u64 = 64 * 1024;
const TEMPLATE: &str = "artifacts.capture";
const WORKFLOW: &str = "artifacts";
const BARRIER: &str = "zz-scan-barrier/tick.bin";

#[derive(Default)]
struct State {
    grant: Mutex<Option<Artifacts>>,
    paths: Mutex<Vec<PathBuf>>,
    done: tokio::sync::Notify,
}

struct Capture;

struct DirectoryRun {
    workflow: Workflow<Capture>,
    state: Arc<State>,
    sink: Arc<RecordingSink>,
    token: String,
    cursor: u64,
    barrier: u64,
    observed_barrier: u64,
    notes: Vec<String>,
    terminal: bool,
}

impl SidecarTemplate for Capture {
    const RUN: &'static str = "artifacts.capture.run";
    const SIGNAL: &'static str = "artifacts.capture.signal";

    type Own = ();
    type Parameters = ();

    fn definition() -> TemplateDefinition {
        let mut template = TemplateContext::new();
        template.set_template_id(dokime_runtime::TemplateId::from_seed(TEMPLATE));
        template.set_metadata(dokime_runtime::definition::authoring::metadata(TEMPLATE, "Capture", None::<String>));
        template.set_steps(vec![Step::new("capture", "Capture")]);

        TemplateDefinition {
            template: template.into_test_case_template().unwrap(),
            records: Vec::new(),
            streams: Vec::new(),
        }
    }

    fn initialize<'a>(ctx: &'a mut InitializationContext<'_, Self, ()>) -> InitializationFuture<'a, (Self, ())> {
        Box::pin(async move {
            let state = ctx.environment().binding::<State>();
            state.paths.lock().unwrap().push(ctx.artifacts().directory("capture").unwrap());
            *state.grant.lock().unwrap() = Some(ctx.artifacts().clone());
            Ok((Self, ()))
        })
    }

    fn run_step<'a>(&'a mut self, _: &'a str, ctx: &'a mut StepContext<'_, Self, ()>) -> StepFuture<'a> {
        Box::pin(async move {
            let state = ctx.environment().binding::<State>();
            state.paths.lock().unwrap().push(ctx.artifacts().directory("capture").unwrap());
            state.done.notified().await;

            Ok(())
        })
    }

    fn cleanup<'a>(&'a mut self, ctx: &'a mut CleanupContext<'_, Self, ()>) -> StepFuture<'a> {
        Box::pin(async move {
            ctx.environment().binding::<State>().paths.lock().unwrap().push(ctx.artifacts().directory("capture").unwrap());

            Ok(())
        })
    }

    fn evaluate<'a>(_: &'a str, _: &'a CriterionContext<'_, Self>) -> EvaluationFuture<'a, bool> {
        Box::pin(async { Ok(None) })
    }
}

impl DirectoryRun {
    fn directory(&self, name: &str) -> io::Result<PathBuf> {
        self.state.grant.lock().unwrap().as_ref().unwrap().directory(name).map_err(|error| {
            let dokime_runtime::Error::Artifact(error) = error else { panic!("unexpected artifact error: {error}") };
            error
        })
    }

    fn drain(&mut self) {
        let reply = self.workflow.drain(&self.token, self.cursor).unwrap();
        for encoded in &reply.step_begins {
            let start: dokime_domain::model::execution::result::StepAttemptStart = serde_json::from_str(encoded).unwrap();
            self.workflow.admit(
                &self.token,
                &StepAdmission {
                    attempt_id: start.attempt_id.to_string(),
                    accepted: true,
                    message: String::new(),
                },
            );
        }
        self.cursor = reply.cursor;
        for chunk in reply.artifact_chunks {
            if chunk.name == BARRIER {
                self.observed_barrier += chunk.bytes.len() as u64;
            } else if chunk.name != "filler.bin" {
                self.sink.push(chunk);
            }
        }
        self.notes.extend(reply.diagnostics.into_iter().filter(|line| line.message.contains("capture/")).map(|line| line.message));
        if let Some(outcome) = reply.outcome {
            assert!(outcome.passed, "capture workflow failed: {outcome:?}");
            self.terminal = true;
        }
    }

    // Steps the paused clock one scan interval at a time until the capture
    // reaches its public output boundary.
    async fn until(&mut self, ready: impl Fn(&Self) -> bool) {
        while !ready(self) {
            tokio::time::advance(Duration::from_millis(10)).await;
            tokio::task::yield_now().await;
            self.drain();
        }
    }

    async fn scan(&mut self) -> Vec<String> {
        // A last-sorted directory provides an observable scan barrier. Two
        // passes ensure diagnostics from the first pass have also been filed.
        // Tokio time is paused: no wall-clock sleeps or guessed scan delays.
        for _ in 0..2 {
            self.barrier += GROWTH_FLOOR;
            let directory = self.directory("zz-scan-barrier").unwrap();
            std::fs::write(directory.join("tick.bin"), vec![0; self.barrier as usize]).unwrap();
            self.until(|run| run.observed_barrier == run.barrier).await;
        }

        std::mem::take(&mut self.notes)
    }

    async fn sweep(&mut self) -> Vec<String> {
        self.state.done.notify_one();
        self.until(|run| run.terminal).await;
        self.workflow.shutdown().await;

        std::mem::take(&mut self.notes)
    }
}

async fn watching(token: &str) -> (DirectoryRun, Arc<RecordingSink>, PathBuf) {
    let state = Arc::new(State::default());
    let workflow = Workflow::<Capture>::new(WORKFLOW, "fixture").bound(state.clone());
    workflow
        .start(RunExecution {
            mode: "automatic".into(),
            execution_token: token.to_owned(),
            test_cycle_id: dokime_runtime::TestCycleId::from_seed(token).to_string(),
            test_execution_id: dokime_runtime::TestExecutionId::from_seed(token).to_string(),
            template_identifier: TEMPLATE.to_owned(),
            workflow: WORKFLOW.to_owned(),
            secrets_json: "{}".to_owned(),
            parameters_json: "{}".to_owned(),
        })
        .unwrap();
    let sink = Arc::new(RecordingSink::new());
    let mut run = DirectoryRun {
        workflow,
        state,
        sink: sink.clone(),
        token: token.to_owned(),
        cursor: 0,
        barrier: 0,
        observed_barrier: 0,
        notes: Vec::new(),
        terminal: false,
    };
    run.until(|run| run.state.paths.lock().unwrap().len() >= 2).await;
    let path = run.directory("capture").unwrap();

    (run, sink, path)
}

fn grown(length: usize, fill: u8) -> Vec<u8> {
    vec![fill; length]
}

#[tokio::test(start_paused = true)]
async fn file_sent_piece() {
    let (mut directories, sink, path) = watching("growth").await;

    std::fs::write(path.join("chamber.log"), b"one line\n").unwrap();
    assert!(directories.scan().await.is_empty());
    assert!(sink.chunks().is_empty(), "a few bytes are not worth a frame and a replay fingerprint");

    let first = grown(GROWTH_FLOOR as usize + 7, b'a');
    std::fs::write(path.join("chamber.log"), &first).unwrap();
    directories.scan().await;
    assert_eq!(sink.assembled("capture/chamber.log"), first);
    let opened = sink.chunks();
    assert_eq!(opened.len(), 1);
    assert_eq!(opened[0].name, "capture/chamber.log");
    assert_eq!(opened[0].mime, "text/plain");
    assert_eq!(opened[0].sequence, 0);
    assert!(!opened[0].last, "the file is still the run's to write");
    assert!(!opened[0].replace);

    let mut grown_file = first.clone();
    grown_file.extend(grown(GROWTH_FLOOR as usize, b'b'));
    std::fs::write(path.join("chamber.log"), &grown_file).unwrap();
    directories.scan().await;
    let appended = sink.chunks();
    assert_eq!(appended.len(), 2, "growth is the transfer's next piece");
    assert_eq!(appended[1].sequence, 1);
    assert_eq!(appended[1].bytes, grown(GROWTH_FLOOR as usize, b'b'), "only what the file grew by is sent again");
    assert_eq!(sink.assembled("capture/chamber.log"), grown_file);

    directories.sweep().await;
}

#[tokio::test(start_paused = true)]
async fn file_shrank_had() {
    let (mut directories, sink, path) = watching("shrink").await;
    std::fs::write(path.join("chamber.log"), grown(GROWTH_FLOOR as usize, b'a')).unwrap();
    directories.scan().await;

    std::fs::write(path.join("chamber.log"), b"rewritten\n").unwrap();
    directories.scan().await;
    assert_eq!(sink.chunks().len(), 1, "a small rewrite waits for the sweep like any other small change");

    directories.sweep().await;
    let chunks = sink.chunks();
    let replacing = chunks.iter().filter(|chunk| chunk.replace).collect::<Vec<_>>();
    assert_eq!(replacing.len(), 1);
    assert_eq!(replacing[0].bytes, b"rewritten\n");
    assert!(replacing[0].last);
    assert_eq!(
        chunks.iter().map(|chunk| chunk.sequence).collect::<Vec<_>>(),
        [0, 1],
        "sequences carry on rather than restarting with the content"
    );
    assert_eq!(sink.assembled("capture/chamber.log"), b"rewritten\n");
}

#[tokio::test(start_paused = true)]
async fn rewrite_same_time() {
    let (mut directories, sink, path) = watching("in-place").await;
    let file = path.join("chamber.log");
    std::fs::write(&file, grown(GROWTH_FLOOR as usize, b'a')).unwrap();
    directories.scan().await;

    std::fs::write(&file, grown(GROWTH_FLOOR as usize, b'b')).unwrap();
    // Stated rather than waited for: whether two writes a millisecond apart
    // carry different times is the filesystem's business, and the rule
    // being tested is what the scan does when they do.
    std::fs::File::options().write(true).open(&file).unwrap().set_modified(SystemTime::now() + Duration::from_secs(2)).unwrap();
    directories.scan().await;

    let chunks = sink.chunks();
    assert_eq!(chunks.len(), 2);
    assert!(chunks[1].replace, "the file no longer says what it said");
    assert_eq!(sink.assembled("capture/chamber.log"), grown(GROWTH_FLOOR as usize, b'b'));

    directories.sweep().await;
}

#[tokio::test(start_paused = true)]
async fn file_past_skipped() {
    let (mut directories, sink, path) = watching("too-large").await;
    let file = std::fs::File::create(path.join("capture.bin")).unwrap();
    // Sparse: the rule is about the size the file claims, and writing
    // sixteen megabytes to prove it would only measure the disk.
    file.set_len(MAX_ARTIFACT_BYTES + 1).unwrap();
    drop(file);

    let notes = directories.scan().await;
    assert_eq!(notes.len(), 1);
    assert!(notes[0].contains("capture/capture.bin") && notes[0].contains("limit"), "the diagnostic names the file: {}", notes[0]);
    assert!(directories.scan().await.is_empty(), "refused once, then skipped");
    assert!(directories.sweep().await.is_empty());
    assert!(sink.chunks().is_empty(), "a refused file never opens a transfer for the sweep to finalize");
}

#[tokio::test(start_paused = true)]
async fn file_outgrows_opened() {
    let (mut directories, sink, path) = watching("outgrown").await;
    let file = path.join("chamber.log");
    std::fs::write(&file, grown(GROWTH_FLOOR as usize, b'a')).unwrap();
    directories.scan().await;
    std::fs::File::options().write(true).open(&file).unwrap().set_len(MAX_ARTIFACT_BYTES + 1).unwrap();

    let notes = directories.scan().await;

    assert_eq!(notes.len(), 1);
    let chunks = sink.chunks();
    assert_eq!(chunks.len(), 2);
    assert!(
        chunks[1].aborted && chunks[1].last && chunks[1].bytes.is_empty(),
        "the host is told the prefix it holds will never be completed"
    );

    directories.sweep().await;
}

#[tokio::test(start_paused = true)]
async fn name_refuses_directory() {
    let (mut directories, sink, path) = watching("unnamed").await;
    std::fs::write(path.join("notes"), b"no extension\n").unwrap();

    let notes = directories.scan().await;

    assert_eq!(notes.len(), 1);
    assert!(notes[0].contains("capture/notes"), "{}", notes[0]);
    assert!(directories.scan().await.is_empty(), "reported once, then skipped");
    assert!(sink.chunks().is_empty());

    directories.sweep().await;
}

#[tokio::test(start_paused = true)]
async fn sweep_finalizes_directory() {
    let (mut directories, sink, path) = watching("sweep").await;
    std::fs::create_dir_all(path.join("frames")).unwrap();
    std::fs::write(path.join("frames/first.txt"), b"nested\n").unwrap();
    std::fs::write(path.join("empty.log"), b"").unwrap();

    assert!(directories.sweep().await.is_empty());

    let chunks = sink.chunks();
    assert_eq!(
        chunks.iter().map(|chunk| chunk.name.as_str()).collect::<Vec<_>>(),
        ["capture/empty.log", "capture/frames/first.txt"],
        "a file the scan never saw is still swept up, named by its path"
    );
    assert!(chunks.iter().all(|chunk| chunk.last));
    assert!(chunks[0].bytes.is_empty(), "an empty file the run meant to produce is one terminal piece");
    assert_eq!(chunks[1].bytes, b"nested\n");
    assert_eq!(
        chunks.iter().map(|chunk| chunk.transfer_id.as_str()).collect::<std::collections::BTreeSet<_>>().len(),
        2,
        "one transfer identity per file"
    );
    assert!(!path.exists(), "the directory is removed after the sweep");
}

#[tokio::test(start_paused = true)]
async fn one_name_refused() {
    let (mut directories, _sink, path) = watching("identity").await;

    assert_eq!(directories.directory("capture").unwrap(), path);
    assert_ne!(directories.directory("other").unwrap(), path);
    for refused in ["", "..", "capture/../..", "/absolute", "back\\slash"] {
        assert_eq!(
            directories.directory(refused).unwrap_err().kind(),
            io::ErrorKind::InvalidInput,
            "a directory named {refused:?} could name a place of its own"
        );
    }

    directories.sweep().await;
}

#[tokio::test(start_paused = true)]
async fn one_directory_cleanup() {
    let (mut run, sink, opened) = watching("directory-context").await;
    std::fs::write(opened.join("evidence.txt"), b"written by anything at all").unwrap();

    assert!(run.sweep().await.is_empty());
    assert_eq!(*run.state.paths.lock().unwrap(), [opened.clone(), opened.clone(), opened.clone()]);
    assert_eq!(sink.assembled("capture/evidence.txt"), b"written by anything at all");
    assert!(!opened.exists(), "the directory is removed once swept");
}

#[tokio::test(start_paused = true)]
async fn closed_backlog_tick() {
    // A regression that blocks the only runtime worker hangs this story
    // until the lane deadline ends it.
    let (mut run, sink, path) = watching("scanning-fixture-gate").await;
    let artifacts = run.state.grant.lock().unwrap().as_ref().unwrap().clone();
    // The public writer fills the current 16 MiB backlog budget using valid
    // bounded chunks. Keep the finalizer open until the host drains the gate.
    let mut writer = tokio::task::spawn_blocking(move || {
        let mut writer = artifacts.create("filler.bin").unwrap();
        let chunk = vec![0; dokime_runtime::transport::MAX_ARTIFACT_CHUNK_BYTES];
        for _ in 0..(16 * 1024 * 1024 / chunk.len()) {
            writer.write_all(&chunk).unwrap();
        }
        writer
    })
    .await
    .unwrap();
    std::fs::write(path.join("evidence.bin"), vec![7u8; 70 * 1024]).unwrap();

    let finishing = tokio::task::spawn_blocking(move || {
        writer.write_all(b"blocked").unwrap();
        writer.finish().unwrap();
    });
    tokio::time::advance(Duration::from_millis(300)).await;
    tokio::task::yield_now().await;
    assert!(!finishing.is_finished(), "the undrained backlog applies backpressure");

    // The runtime remains responsive while the scanner waits on its blocking thread.
    let responsive = tokio::spawn(async { true });
    assert!(responsive.await.unwrap());

    run.drain();
    // A subsequent cursor acknowledges the first reply and releases backlog.
    run.drain();
    run.until(|_| finishing.is_finished()).await;
    finishing.await.unwrap();
    assert!(run.scan().await.is_empty());
    assert_eq!(sink.assembled("capture/evidence.bin"), vec![7u8; 70 * 1024]);
    assert!(run.sweep().await.is_empty());
}
