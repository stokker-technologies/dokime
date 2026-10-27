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
use std::{
    fmt::Display,
    io::{self, IsTerminal, Write},
    str::FromStr,
    sync::atomic::{AtomicU8, Ordering},
    time::Duration,
};

// Third Party Imports
use anstyle::{AnsiColor, Style};
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};

// Local Imports

/* === Constants === */

static COLOR_MODE: AtomicU8 = AtomicU8::new(0);

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug)]
pub struct Terminal {
    mode: ColorMode,
}

pub struct Progress {
    bar: ProgressBar,
    terminal: Terminal,
}

/* === Enums === */

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ColorMode {
    #[default]
    Auto,

    Always,
    Never,
}

/* === Traits === */

/* === Implementations === */

impl FromStr for ColorMode {
    type Err = super::Error;
    #[tracing::instrument(level = "debug", skip_all)]
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "auto" => Ok(Self::Auto),
            "always" => Ok(Self::Always),
            "never" => Ok(Self::Never),
            _ => Err(super::Error::InvalidColorMode),
        }
    }
}

impl Terminal {
    pub fn new(mode: ColorMode) -> Self {
        Self { mode: if mode == ColorMode::Auto { color_mode() } else { mode } }
    }
    pub fn color_enabled(&self) -> bool {
        let choice = match self.mode {
            ColorMode::Auto => anstream::ColorChoice::Auto,
            ColorMode::Always => anstream::ColorChoice::Always,
            ColorMode::Never => anstream::ColorChoice::Never,
        };
        matches!(
            anstream::AutoStream::new(io::stderr(), choice).current_choice(),
            anstream::ColorChoice::Always | anstream::ColorChoice::AlwaysAnsi
        )
    }
    fn write(&self, message: &str) {
        let choice = match self.mode {
            ColorMode::Auto => anstream::ColorChoice::Auto,
            ColorMode::Always => anstream::ColorChoice::Always,
            ColorMode::Never => anstream::ColorChoice::Never,
        };
        let _ = writeln!(anstream::AutoStream::new(io::stderr().lock(), choice), "{message}");
    }
    fn line(&self, label: &str, message: impl Display, color: AnsiColor) {
        let style = Style::new().fg_color(Some(color.into())).bold();
        let message = message.to_string().replace('\n', "\n            ");
        self.write(&format!("{style}{label:>10}{style:#}  {message}"));
    }
    pub fn header(&self, tool: &str, version: &str, command: &str) {
        let style = Style::new().bold();
        self.write(&format!("{style}{tool}{style:#} v{version} · {command}"));
    }
    pub fn field(&self, label: &str, value: impl Display) {
        self.line(label, value, AnsiColor::Cyan);
    }
    pub fn status(&self, message: impl Display) {
        self.line("Status", message, AnsiColor::Cyan);
    }
    pub fn success(&self, message: impl Display) {
        self.line("Success", message, AnsiColor::Green);
    }
    pub fn warn(&self, message: impl Display) {
        self.line("Warning", message, AnsiColor::Yellow);
    }
    pub fn error(&self, message: impl Display) {
        self.line("Error", message, AnsiColor::Red);
    }
    pub fn failure(&self, failure: impl Display) {
        let style = Style::new().fg_color(Some(AnsiColor::Red.into())).bold();
        let failure = failure.to_string();
        let mut lines = failure.lines();
        let Some(headline) = lines.next() else {
            return;
        };
        self.write(&format!("{style}{headline}{style:#}"));
        for line in lines {
            self.write(line);
        }
    }
    pub fn hint(&self, message: impl Display) {
        self.line("Hint", message, AnsiColor::Cyan);
    }
    pub fn progress(&self, message: impl Into<String>, tick: Duration) -> Progress {
        let message = message.into();
        let interactive = io::stderr().is_terminal();
        let bar =
            ProgressBar::with_draw_target(None, if interactive { ProgressDrawTarget::stderr() } else { ProgressDrawTarget::hidden() });
        bar.set_style(
            ProgressStyle::with_template("{spinner} {msg} [{elapsed_precise}]")
                .expect("valid progress template")
                .tick_strings(&["-", "\\", "|", "/"]),
        );
        bar.set_message(message.clone());
        if interactive {
            bar.enable_steady_tick(tick);
        } else {
            self.status(&message);
        }
        Progress { bar, terminal: self.clone() }
    }
}

impl Default for Terminal {
    fn default() -> Self {
        Self::new(ColorMode::Auto)
    }
}

impl Progress {
    pub fn log(&self, message: impl Display) {
        self.bar.suspend(|| self.terminal.write(&message.to_string()));
    }
    pub fn finish(self, message: impl Display) {
        self.bar.finish_and_clear();
        self.terminal.success(message);
    }
    pub fn fail(self, message: impl Display) {
        self.bar.finish_and_clear();
        self.terminal.error(message);
    }
}

impl Drop for Progress {
    fn drop(&mut self) {
        self.bar.finish_and_clear();
    }
}

/* === Public Functions === */

pub fn set_color_mode(mode: ColorMode) {
    COLOR_MODE.store(
        match mode {
            ColorMode::Auto => 0,
            ColorMode::Always => 1,
            ColorMode::Never => 2,
        },
        Ordering::Relaxed,
    );
}

pub fn color_mode() -> ColorMode {
    match COLOR_MODE.load(Ordering::Relaxed) {
        1 => ColorMode::Always,
        2 => ColorMode::Never,
        _ => ColorMode::Auto,
    }
}

/* === Private Functions === */

/* === Tests === */
