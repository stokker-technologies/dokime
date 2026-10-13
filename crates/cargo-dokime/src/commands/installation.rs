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

// Local Imports
use crate::error::Error;

/* === Constants === */
#[cfg(any(windows, target_os = "macos"))]
const PRODUCT: &str = "Dokime";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Debug)]
pub(super) struct Installation {
    executable: PathBuf,
    origin: Origin,
}

struct Search {
    home: Option<PathBuf>,
    path: Vec<PathBuf>,
    installed: Vec<PathBuf>,
}

/* === Enums === */
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Origin {
    Explicit,
    Home,
    Path,
    Installed,
}

/* === Traits === */

/* === Implementations === */
impl Origin {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Explicit => "selected with --dokime",
            Self::Home => "found through DOKIME_HOME",
            Self::Path => "found on PATH",
            Self::Installed => "found in the desktop installation",
        }
    }
}

impl Installation {
    pub(super) fn new(executable: PathBuf, origin: Origin) -> Self {
        Self { executable, origin }
    }

    pub(super) fn executable(&self) -> &Path {
        &self.executable
    }

    pub(super) fn origin(&self) -> Origin {
        self.origin
    }

    pub(super) fn gui(&self) -> Result<PathBuf, Error> {
        let directory = self.executable.parent().ok_or_else(|| {
            Error::Invalid(format!(
                "Dokime executable {} has no directory",
                self.executable.display()
            ))
        })?;
        executable_file(&directory.join(gui_name())).ok_or_else(|| {
            Error::Invalid(format!(
                "no Dokime GUI is installed beside {}; install Dokime with the desktop installer, select a complete installation with --dokime PATH, or pass --gui PATH",
                self.executable.display()
            ))
        })
    }
}

impl Search {
    fn from_environment() -> Self {
        Self {
            home: std::env::var_os("DOKIME_HOME").map(PathBuf::from),
            path: std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default())
                // An empty PATH entry means the working directory; a plugin
                // project must never select a Dokime out of its own checkout.
                .filter(|directory| !directory.as_os_str().is_empty())
                .collect(),
            installed: installed_directories(),
        }
    }

    fn find(&self) -> Option<Installation> {
        self.find_with(executable_file)
    }

    fn find_with(
        &self,
        mut executable: impl FnMut(&Path) -> Option<PathBuf>,
    ) -> Option<Installation> {
        let name = backend_name();
        if let Some(home) = &self.home
            && let Some(path) = executable(&home.join(&name))
        {
            return Some(Installation::new(path, Origin::Home));
        }
        for directory in &self.path {
            if let Some(path) = executable(&directory.join(&name)) {
                return Some(Installation::new(path, Origin::Path));
            }
        }
        for directory in &self.installed {
            if let Some(path) = executable(&directory.join(&name)) {
                return Some(Installation::new(path, Origin::Installed));
            }
        }
        None
    }
}

/* === Public Functions === */
pub(super) fn gui_override(path: &Path) -> Result<PathBuf, Error> {
    executable_file(path).ok_or_else(|| {
        Error::Invalid(format!(
            "Dokime GUI {} is not a regular file",
            path.display()
        ))
    })
}

pub(super) fn discover() -> Result<Installation, Error> {
    Search::from_environment().find().ok_or_else(not_installed)
}

/* === Private Functions === */
fn not_installed() -> Error {
    Error::Invalid(
        "no Dokime installation was found; install Dokime with the desktop installer, set DOKIME_HOME to its directory, or pass --dokime PATH".into(),
    )
}

#[cfg(windows)]
fn installed_directories() -> Vec<PathBuf> {
    ["LOCALAPPDATA", "ProgramFiles"]
        .into_iter()
        .filter_map(std::env::var_os)
        .map(|root| PathBuf::from(root).join(PRODUCT))
        .collect()
}

#[cfg(target_os = "macos")]
fn installed_directories() -> Vec<PathBuf> {
    vec![PathBuf::from(format!(
        "/Applications/{PRODUCT}.app/Contents/MacOS"
    ))]
}

#[cfg(not(any(windows, target_os = "macos")))]
fn installed_directories() -> Vec<PathBuf> {
    vec![PathBuf::from("/usr/bin"), PathBuf::from("/usr/local/bin")]
}

fn backend_name() -> String {
    format!("dokime-server{}", std::env::consts::EXE_SUFFIX)
}

fn gui_name() -> String {
    format!("dokime-gui{}", std::env::consts::EXE_SUFFIX)
}

fn executable_file(path: &Path) -> Option<PathBuf> {
    if !path.is_file() {
        return None;
    }
    std::path::absolute(path).ok()
}

/* === Tests === */
