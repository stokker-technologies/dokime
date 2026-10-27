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
use std::path::{Path, PathBuf};

// Third Party Imports
use tracing_subscriber::{EnvFilter, Layer, Registry, fmt, layer::SubscriberExt, util::SubscriberInitExt};

// Local Imports

/* === Constants === */

static ACTIVE_LOG: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();

/* === Macros === */

/* === Type Aliases === */

type BoxedLayer = Box<dyn Layer<Registry> + Send + Sync>;

/* === Structs === */

pub(crate) struct LoggingGuards {
    _stderr: tracing_appender::non_blocking::WorkerGuard,
    _file: Option<tracing_appender::non_blocking::WorkerGuard>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) fn active_log() -> Option<PathBuf> {
    ACTIVE_LOG.get().cloned()
}

pub(crate) fn init(identifier: &str, retained_files: usize) -> LoggingGuards {
    let (stderr_writer, stderr_guard) = tracing_appender::non_blocking(std::io::stderr());
    let stderr_layer: BoxedLayer = fmt::layer().with_writer(stderr_writer).with_filter(env_filter()).boxed();
    let mut layers: Vec<BoxedLayer> = vec![stderr_layer];

    let mut file_guard = None;
    let mut log_path = None;
    match app_log_dir(identifier).and_then(|dir| file_fmt_layer(&dir, retained_files)) {
        Ok((layer, guard, path)) => {
            layers.push(layer);
            file_guard = Some(guard);
            log_path = Some(path);
        }
        Err(reason) => {
            eprintln!("dokime desktop shell: continuing without a log file ({reason})");
        }
    }

    // A second `init` in the same process never happens in production (only
    // the real binary entrypoint calls this); tolerate it rather than panic
    // the shell over an already-installed subscriber.
    let _ = Registry::default().with(layers).try_init();

    if let Some(path) = log_path {
        let _ = ACTIVE_LOG.set(path.clone());
        tracing::info!(path = %path.display(), "dokime desktop shell log file");
    }

    LoggingGuards { _stderr: stderr_guard, _file: file_guard }
}

/* === Private Functions === */

fn env_filter() -> EnvFilter {
    EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"))
}

fn app_log_dir(identifier: &str) -> Result<PathBuf, String> {
    #[cfg(target_os = "macos")]
    {
        std::env::var_os("HOME")
            .map(|home| PathBuf::from(home).join("Library/Logs").join(identifier))
            .ok_or_else(|| "HOME is not set".to_owned())
    }
    #[cfg(target_os = "windows")]
    {
        std::env::var_os("LOCALAPPDATA")
            .map(|dir| PathBuf::from(dir).join(identifier).join("logs"))
            .ok_or_else(|| "%LOCALAPPDATA% is not set".to_owned())
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share")))
            .map(|dir| dir.join(identifier).join("logs"))
            .ok_or_else(|| "neither XDG_DATA_HOME nor HOME is set".to_owned())
    }
}

fn file_fmt_layer(dir: &Path, retained_files: usize) -> Result<(BoxedLayer, tracing_appender::non_blocking::WorkerGuard, PathBuf), String> {
    std::fs::create_dir_all(dir).map_err(|error| format!("failed to create log directory {}: {error}", dir.display()))?;
    let appender = tracing_appender::rolling::RollingFileAppender::builder()
        .rotation(tracing_appender::rolling::Rotation::DAILY)
        .filename_prefix("dokime-gui")
        .filename_suffix("log")
        .max_log_files(retained_files)
        .build(dir)
        .map_err(|error| format!("failed to open a log file under {}: {error}", dir.display()))?;
    let (writer, guard) = tracing_appender::non_blocking(appender);
    let layer = fmt::layer().with_ansi(false).with_writer(writer).with_filter(env_filter()).boxed();
    Ok((layer, guard, dir.join(format!("dokime-gui.{}.log", today()))))
}

fn today() -> String {
    static DATE_FORMAT: std::sync::LazyLock<Vec<time::format_description::BorrowedFormatItem<'static>>> = std::sync::LazyLock::new(|| {
        // Version 1: the same format-description syntax version `parse`
        // (deprecated) and `RollingFileAppender` itself use internally, so
        // this renders exactly the date its daily rotation names files with.
        time::format_description::parse_borrowed::<1>("[year]-[month]-[day]").expect("static date format is valid")
    });
    time::OffsetDateTime::now_utc().format(&DATE_FORMAT).unwrap_or_else(|_| "unknown-date".to_owned())
}

/* === Tests === */
