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
use std::path::Path;
// Third Party Imports
use tracing_subscriber::{
    EnvFilter, Layer, Registry, fmt, layer::SubscriberExt, util::SubscriberInitExt,
};
// Local Imports
use crate::host::{Error, Result};

/* === Constants === */
const LOG_FORMAT_ENV: &str = "DOKIME_LOG_FORMAT";

const KEEP_LOG_FILES: usize = 14;

/* === Macros === */

/* === Type Aliases === */
pub(crate) type BoxedLayer = Box<dyn Layer<Registry> + Send + Sync>;

/* === Structs === */
pub struct LoggingGuards {
    _appender: tracing_appender::non_blocking::WorkerGuard,
    _file_appender: tracing_appender::non_blocking::WorkerGuard,
    _hub_log_appender: tracing_appender::non_blocking::WorkerGuard,
    #[cfg(debug_assertions)]
    _profile: Option<super::profiles::DevProfileGuards>,
}

struct CompactConsole;

#[derive(Default)]
struct ConsoleFields {
    message: String,
    line: Option<String>,
    plugin: Option<String>,
    execution: Option<String>,
    timestamp: Option<String>,
    details: Vec<String>,
}

/* === Enums === */
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LogFormat {
    Pretty,
    Json,
}

/* === Traits === */

/* === Implementations === */
impl LogFormat {
    fn from_env() -> Self {
        match std::env::var(LOG_FORMAT_ENV) {
            Ok(value) if value.eq_ignore_ascii_case("json") => Self::Json,
            _ => Self::Pretty,
        }
    }
}

impl tracing::field::Visit for ConsoleFields {
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        match field.name() {
            "message" => self.message = value.to_owned(),
            "line" => self.line = Some(value.to_owned()),
            "artifact_id" | "plugin" => self.plugin = Some(value.to_owned()),
            "execution_id" | "execution" => self.execution = Some(value.to_owned()),
            "timestamp" => self.timestamp = Some(value.to_owned()),
            name => self.details.push(format!("{name}={value}")),
        }
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        self.record_str(field, &format!("{value:?}"));
    }
}

impl<S, N> fmt::FormatEvent<S, N> for CompactConsole
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
    N: for<'a> fmt::FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        _context: &fmt::FmtContext<'_, S, N>,
        mut writer: fmt::format::Writer<'_>,
        event: &tracing::Event<'_>,
    ) -> std::fmt::Result {
        let mut fields = ConsoleFields::default();
        event.record(&mut fields);
        let timestamp = fields
            .timestamp
            .filter(|value| !value.is_empty())
            .unwrap_or_else(dokime::execution::log::timestamp);
        let time = timestamp.get(11..23).unwrap_or(&timestamp);
        let plugin = fields.plugin.as_deref().unwrap_or("host");
        let location = match fields.execution.as_deref() {
            Some(run) => format!("{plugin}/{}", run.chars().take(8).collect::<String>()),
            None => plugin.to_owned(),
        };
        let message = fields.line.as_deref().unwrap_or(&fields.message);
        let level = event.metadata().level();
        let (color, reset) = if writer.has_ansi_escapes() {
            let color = match *level {
                tracing::Level::ERROR => "\x1b[31m",
                tracing::Level::WARN => "\x1b[33m",
                tracing::Level::INFO => "\x1b[32m",
                tracing::Level::DEBUG => "\x1b[34m",
                tracing::Level::TRACE => "\x1b[35m",
            };

            (color, "\x1b[0m")
        } else {
            ("", "")
        };

        for line in message.split('\n') {
            write!(writer, "{time} {color}{level}{reset} {location} {line}")?;
            if fields.line.is_none() && !fields.details.is_empty() {
                write!(writer, " {}", fields.details.join(" "))?;
            }
            writeln!(writer)?;
        }
        Ok(())
    }
}

/* === Public Functions === */
pub fn init(root: &Path, ansi: bool) -> Result<LoggingGuards> {
    let (layer, appender_guard) = stderr_fmt_layer(ansi)?;
    let (file_layer, file_appender_guard) = file_fmt_layer(root)?;
    let (hub_log_layer, hub_log_appender_guard) = super::hub_log::hub_log_layer(root)?;
    #[cfg_attr(not(debug_assertions), allow(unused_mut))]
    let mut layers: Vec<BoxedLayer> = vec![layer, file_layer, hub_log_layer];

    #[cfg(debug_assertions)]
    let profile = match super::profiles::dev_profile_layers(root)? {
        Some((mut profile_layers, guards)) => {
            layers.append(&mut profile_layers);
            Some(guards)
        }
        None => None,
    };

    Registry::default()
        .with(layers)
        // Not a logging layer: `ErrorLayer` is what lets a captured
        // `dokime::plugin::Diagnostic` read back the spans it happened inside.
        // Without it in this subscriber every host-side span trace is empty, and
        // nothing says so.
        .with(tracing_error::ErrorLayer::default())
        .try_init()
        .map_err(|error| Error::Setup(format!("failed to install tracing subscriber: {error}")))?;

    Ok(LoggingGuards {
        _appender: appender_guard,
        _file_appender: file_appender_guard,
        _hub_log_appender: hub_log_appender_guard,
        #[cfg(debug_assertions)]
        _profile: profile,
    })
}

/* === Private Functions === */
fn stderr_fmt_layer(
    ansi: bool,
) -> Result<(BoxedLayer, tracing_appender::non_blocking::WorkerGuard)> {
    let (writer, guard) = tracing_appender::non_blocking(std::io::stderr());
    Ok((fmt_layer(LogFormat::from_env(), writer, ansi)?, guard))
}

fn file_fmt_layer(
    root: &Path,
) -> Result<(BoxedLayer, tracing_appender::non_blocking::WorkerGuard)> {
    let directory = root.join("logs");
    std::fs::create_dir_all(&directory).map_err(|error| {
        Error::Setup(format!(
            "failed to create log directory {}: {error}",
            directory.display()
        ))
    })?;
    let appender = tracing_appender::rolling::RollingFileAppender::builder()
        .rotation(tracing_appender::rolling::Rotation::DAILY)
        .filename_prefix("dokime-serve")
        .filename_suffix("log")
        .max_log_files(KEEP_LOG_FILES)
        .build(&directory)
        .map_err(|error| {
            Error::Setup(format!(
                "failed to open a log file under {}: {error}",
                directory.display()
            ))
        })?;
    let (writer, guard) = tracing_appender::non_blocking(appender);
    Ok((fmt_layer(LogFormat::from_env(), writer, false)?, guard))
}

fn log_filter(host: Option<&str>, plugin: Option<&str>) -> Result<EnvFilter> {
    let mut filter = host
        .and_then(|value| EnvFilter::try_new(value).ok())
        .unwrap_or_else(|| EnvFilter::new("info"));
    if let Some(plugin) = plugin {
        let plugin = EnvFilter::try_new(plugin)
            .map_err(|error| Error::Setup(format!("invalid DOKIME_PLUGIN_LOG: {error}")))?;
        let level = plugin
            .max_level_hint()
            .unwrap_or(tracing::level_filters::LevelFilter::TRACE);
        let stderr = if level == tracing::level_filters::LevelFilter::OFF {
            "off"
        } else {
            "trace"
        };

        filter = filter
            .add_directive(format!("sidecar={level}").parse().expect("valid level"))
            .add_directive(
                format!("sidecar::stderr={stderr}")
                    .parse()
                    .expect("valid level"),
            );
    }
    Ok(filter)
}

fn fmt_layer(
    format: LogFormat,
    writer: tracing_appender::non_blocking::NonBlocking,
    ansi: bool,
) -> Result<BoxedLayer> {
    let filter = log_filter(
        std::env::var("RUST_LOG").ok().as_deref(),
        std::env::var("DOKIME_PLUGIN_LOG").ok().as_deref(),
    )?;
    Ok(match format {
        LogFormat::Json => fmt::layer()
            .json()
            .with_writer(writer)
            .with_target(true)
            .with_filter(filter)
            .boxed(),
        LogFormat::Pretty => fmt::layer()
            .event_format(CompactConsole)
            .with_ansi(ansi)
            .with_writer(writer)
            .with_filter(filter)
            .boxed(),
    })
}

/* === Tests === */
