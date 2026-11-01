/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Opt-In Payload-Free Test Span Export

/* === Modules === */

mod error;

/* === Re-Exports === */

pub use dokime_macros::trace_test;
pub use error::Error;

/* === Imports === */

// Standard Library Imports
use std::{
    collections::HashMap,
    fs::File,
    io::{Seek, SeekFrom, Write},
    path::Path,
    sync::{Arc, Mutex, OnceLock, mpsc},
    time::{Instant, SystemTime, UNIX_EPOCH},
};

// Third Party Imports
use tracing::{
    Subscriber,
    span::{Attributes, Id},
};
use tracing_subscriber::{
    Layer, Registry,
    layer::{Context, SubscriberExt},
    registry::LookupSpan,
    util::SubscriberInitExt,
};

// Local Imports

/* === Constants === */

const TRACE_DIRECTORY: &str = "DOKIME_TEST_TRACE_DIR";
static STATE: OnceLock<State> = OnceLock::new();
static INSTALL: Mutex<bool> = Mutex::new(false);

/* === Macros === */

/* === Type Aliases === */

pub type Result<T> = std::result::Result<T, Error>;

pub type BoxedLayer = Box<dyn Layer<Registry> + Send + Sync>;

/* === Structs === */

struct State {
    trace: Option<Trace>,
}

#[derive(Clone)]
struct Trace {
    capture: Arc<Mutex<Capture>>,
    output: mpsc::SyncSender<Output>,
}

struct Capture {
    started: Instant,
    epoch: f64,
    next: u64,
    root: Option<u64>,
    spans: HashMap<u64, Span>,
}

struct Writer {
    file: File,
    count: usize,
    bytes: Vec<u8>,
}

struct Span {
    id: u64,
    parent: Option<u64>,
    started: f64,
    name: &'static str,
    target: &'static str,
}

#[derive(serde::Serialize)]
struct Event {
    args: Links,
    cat: &'static str,
    dur: f64,
    name: &'static str,
    ph: &'static str,
    pid: u32,
    tid: u64,
    ts: f64,
}

#[derive(serde::Serialize)]
struct Links {
    parent_id: Option<u64>,
    span_id: u64,
}

pub struct Guard {
    root: Option<Span>,
}

/* === Enums === */

enum Output {
    Event(Event),
    Flush(mpsc::Sender<std::io::Result<()>>),
}

/* === Traits === */

/* === Implementations === */

impl Drop for Guard {
    fn drop(&mut self) {
        if let Some(root) = self.root.take()
            && let Some(trace) = STATE.get().and_then(|state| state.trace.as_ref())
        {
            let event = {
                let mut capture = trace.capture.lock().expect("trace capture poisoned");
                capture.root = None;
                capture.complete(root)
            };
            trace.output.send(Output::Event(event)).expect("trace writer stopped");
        }
        flush();
    }
}

impl Capture {
    fn now(&self) -> f64 {
        self.epoch + self.started.elapsed().as_secs_f64() * 1_000_000.0
    }

    fn complete(&self, span: Span) -> Event {
        Event {
            args: Links { span_id: span.id, parent_id: span.parent },
            cat: span.target,
            dur: self.now() - span.started,
            name: span.name,
            ph: "X",
            pid: std::process::id(),
            tid: span.id,
            ts: span.started,
        }
    }
}

impl Writer {
    fn run(mut self, input: mpsc::Receiver<Output>) {
        let mut failure = None;
        for output in input {
            match output {
                Output::Event(event) => {
                    if let Err(error) = self.write(event) {
                        failure = Some(error);
                    }
                }
                Output::Flush(reply) => {
                    let flushed = self.file.flush();
                    let result = failure.take().map_or(flushed, Err);
                    let _ = reply.send(result);
                }
            }
        }
    }

    fn write(&mut self, event: Event) -> std::io::Result<()> {
        // Serialize Before File I/O, Keeping Each Span To One Native Write.
        self.bytes.clear();
        if self.count != 0 {
            self.bytes.extend_from_slice(b",\n");
        }
        serde_json::to_writer(&mut self.bytes, &event)?;
        self.bytes.extend_from_slice(b"]}");

        // Keep The File Valid Even If A Child Is Later Killed.
        self.file.seek(SeekFrom::End(-2))?;
        self.file.write_all(&self.bytes)?;
        self.count += 1;
        Ok(())
    }
}

impl<S> Layer<S> for Trace
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_new_span(&self, attributes: &Attributes<'_>, id: &Id, context: Context<'_, S>) {
        let mut writer = self.capture.lock().expect("trace capture poisoned");
        writer.next += 1;
        let parent =
            attributes.parent().cloned().or_else(|| attributes.is_contextual().then(|| context.current_span().id().cloned()).flatten());
        let parent = parent.and_then(|id| writer.spans.get(&id.into_u64()).map(|span| span.id)).or(writer.root);
        let span = Span {
            id: writer.next,
            parent,
            started: writer.now(),
            name: attributes.metadata().name(),
            target: attributes.metadata().target(),
        };
        writer.spans.insert(id.into_u64(), span);
    }

    fn on_close(&self, id: Id, _context: Context<'_, S>) {
        let event = {
            let mut capture = self.capture.lock().expect("trace capture poisoned");
            capture.spans.remove(&id.into_u64()).map(|span| capture.complete(span))
        };
        if let Some(event) = event {
            self.output.send(Output::Event(event)).expect("trace writer stopped");
        }
    }
}

/* === Public Functions === */

// Install Application Logging And Export Together, So Filtered Layers Register Normally.
pub fn configure(layers: Vec<BoxedLayer>) -> Result<Guard> {
    let mut installed = INSTALL.lock().expect("trace installation poisoned");
    install(&mut installed, layers)?;
    Ok(Guard { root: None })
}

// Application Entrypoints Can Open Their File Before Their Logging Owner Runs.
pub fn pending() -> Option<Guard> {
    std::env::var_os(TRACE_DIRECTORY)?;
    let _installed = INSTALL.lock().expect("trace installation poisoned");
    state().expect("open process trace");
    Some(Guard { root: None })
}

pub fn process() -> Option<Guard> {
    std::env::var_os(TRACE_DIRECTORY)?;
    let mut installed = INSTALL.lock().expect("trace installation poisoned");
    if !*installed {
        install(&mut installed, Vec::new()).expect("install process trace subscriber");
    }
    Some(Guard { root: None })
}

pub fn test(name: &'static str, logging: bool) -> Option<Guard> {
    std::env::var_os(TRACE_DIRECTORY)?;
    let mut installed = INSTALL.lock().expect("trace installation poisoned");
    let trace = state().expect("open test trace").trace.as_ref().expect("trace directory is set");
    let root = {
        let mut writer = trace.capture.lock().expect("trace capture poisoned");
        writer.next += 1;
        writer.root = Some(writer.next);
        Span {
            id: writer.next,
            parent: None,
            started: writer.now(),
            name,
            target: "dokime_test",
        }
    };
    if !logging && !*installed {
        install(&mut installed, Vec::new()).expect("install test trace subscriber");
    }
    Some(Guard { root: Some(root) })
}

pub fn flush() {
    if let Some(trace) = STATE.get().and_then(|state| state.trace.as_ref()) {
        let (reply, completed) = mpsc::channel();
        trace.output.send(Output::Flush(reply)).expect("trace writer stopped");
        completed.recv().expect("trace writer stopped before flush").expect("flush test trace");
    }
}

/* === Private Functions === */

// Callers Hold INSTALL While Initializing The Process Owner.
fn state() -> std::io::Result<&'static State> {
    if STATE.get().is_none() {
        let _ = STATE.set(State { trace: open()? });
    }
    Ok(STATE.get().expect("trace owner initialized"))
}

fn install(installed: &mut bool, mut layers: Vec<BoxedLayer>) -> Result<()> {
    if *installed {
        return Err(Error::AlreadyInstalled);
    }
    if let Some(trace) = &state()?.trace {
        layers.push(Box::new(trace.clone()));
    }
    let subscriber = Registry::default().with(layers).with(tracing_error::ErrorLayer::default());
    subscriber.try_init()?;
    *installed = true;
    Ok(())
}

fn open() -> std::io::Result<Option<Trace>> {
    let Some(directory) = std::env::var_os(TRACE_DIRECTORY) else {
        return Ok(None);
    };
    let directory = Path::new(&directory);
    std::fs::create_dir_all(directory)?;
    let executable = std::env::current_exe()?;
    let process = executable.file_stem().and_then(|name| name.to_str()).unwrap_or("process");
    let epoch = SystemTime::now().duration_since(UNIX_EPOCH).expect("system time");
    let path = directory.join(format!("{process}-{}-{}.pftrace", std::process::id(), epoch.as_nanos()));
    let mut file = File::options().write(true).create_new(true).open(path)?;
    file.write_all(b"{\"traceEvents\":[]}")?;
    // Bound Pending Metadata And Keep File I/O Off Instrumented Runtime Workers.
    let (output, input) = mpsc::sync_channel(4096);
    let writer = Writer { file, count: 0, bytes: Vec::new() };
    std::thread::Builder::new().name("dokime-trace".into()).spawn(move || writer.run(input))?;
    let capture = Capture {
        started: Instant::now(),
        epoch: epoch.as_secs_f64() * 1_000_000.0,
        next: 0,
        root: None,
        spans: HashMap::new(),
    };
    Ok(Some(Trace {
        capture: Arc::new(Mutex::new(capture)),
        output,
    }))
}

/* === Tests === */
