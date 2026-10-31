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
    sync::{Arc, Mutex, OnceLock},
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
struct Trace(Arc<Mutex<Writer>>);

struct Writer {
    file: File,
    started: Instant,
    epoch: f64,
    count: usize,
    next: u64,
    root: Option<u64>,
    spans: HashMap<u64, Span>,
    error: Option<std::io::Error>,
}

struct Span {
    id: u64,
    parent: Option<u64>,
    started: f64,
    name: String,
    target: &'static str,
}

pub struct Guard {
    root: Option<Span>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Drop for Guard {
    fn drop(&mut self) {
        if let Some(root) = self.root.take()
            && let Some(trace) = STATE.get().and_then(|state| state.trace.as_ref())
        {
            let mut writer = trace.0.lock().expect("trace writer poisoned");
            writer.root = None;
            if let Err(error) = writer.write(root) {
                writer.error = Some(error);
            }
        }
        flush();
    }
}

impl Writer {
    fn now(&self) -> f64 {
        self.epoch + self.started.elapsed().as_secs_f64() * 1_000_000.0
    }

    fn write(&mut self, span: Span) -> std::io::Result<()> {
        let event = serde_json::json!({
            "ph": "X", "name": span.name, "cat": span.target,
            "pid": std::process::id(), "tid": span.id,
            "ts": span.started, "dur": self.now() - span.started,
            "args": {"span_id": span.id, "parent_id": span.parent}
        });
        // Keep The File Valid Even If A Child Is Later Killed.
        self.file.seek(SeekFrom::End(-2))?;
        if self.count != 0 {
            self.file.write_all(b",\n")?;
        }
        serde_json::to_writer(&mut self.file, &event)?;
        self.file.write_all(b"]}")?;
        self.count += 1;
        Ok(())
    }
}

impl<S> Layer<S> for Trace
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_new_span(&self, attributes: &Attributes<'_>, id: &Id, context: Context<'_, S>) {
        let mut writer = self.0.lock().expect("trace writer poisoned");
        writer.next += 1;
        let parent =
            attributes.parent().cloned().or_else(|| attributes.is_contextual().then(|| context.current_span().id().cloned()).flatten());
        let parent = parent.and_then(|id| writer.spans.get(&id.into_u64()).map(|span| span.id)).or(writer.root);
        let span = Span {
            id: writer.next,
            parent,
            started: writer.now(),
            name: attributes.metadata().name().to_owned(),
            target: attributes.metadata().target(),
        };
        writer.spans.insert(id.into_u64(), span);
    }

    fn on_close(&self, id: Id, _context: Context<'_, S>) {
        let mut writer = self.0.lock().expect("trace writer poisoned");
        if let Some(span) = writer.spans.remove(&id.into_u64())
            && let Err(error) = writer.write(span)
        {
            writer.error = Some(error);
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
        let mut writer = trace.0.lock().expect("trace writer poisoned");
        writer.next += 1;
        writer.root = Some(writer.next);
        Span {
            id: writer.next,
            parent: None,
            started: writer.now(),
            name: name.to_owned(),
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
        let mut writer = trace.0.lock().expect("trace writer poisoned");
        writer.file.flush().expect("flush test trace");
        if let Some(error) = writer.error.take() {
            panic!("write test trace: {error}");
        }
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
    let path = directory.join(format!("{process}-{}.pftrace", std::process::id()));
    let mut file = File::create(path)?;
    file.write_all(b"{\"traceEvents\":[]}")?;
    let writer = Writer {
        file,
        started: Instant::now(),
        epoch: SystemTime::now().duration_since(UNIX_EPOCH).expect("system time").as_secs_f64() * 1_000_000.0,
        count: 0,
        next: 0,
        root: None,
        spans: HashMap::new(),
        error: None,
    };
    Ok(Some(Trace(Arc::new(Mutex::new(writer)))))
}

/* === Tests === */
