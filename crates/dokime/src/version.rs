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
use std::{fmt, sync::LazyLock};

// Third Party Imports

// Local Imports

/* === Constants === */

/// Product version of this build.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

const COMMIT_DIGITS: std::ops::RangeInclusive<usize> = 7..=64;
const DIRTY: &str = "-dirty";
const SHORT_DIGITS: usize = 8;
const UNKNOWN: &str = "unknown";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Debug, clap::Args)]
pub struct Args {
    #[arg(long, help = "Emit CLI version metadata as JSON.")]
    json: bool,
}

/// The product version, source commit and publication channel one binary was built from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Build<'a> {
    pub version: &'a str,
    /// `None` when the build had neither a CI commit nor a Git checkout.
    pub revision: Option<Revision<'a>>,
    /// `nightly` or `release` for the publishers' builds; `None` otherwise.
    pub channel: Option<&'a str>,
}

/// A source commit, marked dirty when the checkout had uncommitted tracked changes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Revision<'a> {
    pub commit: &'a str,
    pub dirty: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl<'a> Build<'a> {
    /// Reads the embedded form: a revision of `<commit>[-dirty]` or `unknown`, and a possibly empty channel.
    pub fn parse(version: &'a str, revision: &'a str, channel: &'a str) -> Self {
        let channel = Some(channel).filter(|channel| !channel.is_empty());
        Self {
            version,
            revision: Revision::parse(revision),
            channel,
        }
    }
}

impl fmt::Display for Build<'_> {
    /// `0.1.2 (766330b7 nightly)`, `0.1.2 (766330b7-dirty)` or `0.1.2 (unknown)`.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} (", self.version)?;
        match self.revision {
            Some(revision) => write!(formatter, "{revision}")?,
            None => formatter.write_str(UNKNOWN)?,
        }
        if let Some(channel) = self.channel {
            write!(formatter, " {channel}")?;
        }
        formatter.write_str(")")
    }
}

impl<'a> Revision<'a> {
    pub fn parse(value: &'a str) -> Option<Self> {
        let (commit, dirty) = value.strip_suffix(DIRTY).map_or((value, false), |commit| (commit, true));
        let digit = |byte: u8| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte);
        let valid = COMMIT_DIGITS.contains(&commit.len()) && commit.bytes().all(digit);
        valid.then_some(Self { commit, dirty })
    }

    /// The abbreviated commit, without the dirty mark.
    pub fn short(&self) -> &'a str {
        &self.commit[..self.commit.len().min(SHORT_DIGITS)]
    }

    /// The full commit with its dirty mark, as identity responses carry it.
    pub fn full(&self) -> String {
        let mark = if self.dirty { DIRTY } else { "" };
        format!("{}{mark}", self.commit)
    }
}

impl fmt::Display for Revision<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mark = if self.dirty { DIRTY } else { "" };
        write!(formatter, "{}{mark}", self.short())
    }
}

/* === Public Functions === */

/// The build this binary was compiled from.
pub fn current() -> Build<'static> {
    Build::parse(VERSION, env!("DOKIME_SOURCE_REVISION"), env!("DOKIME_BUILD_CHANNEL"))
}

/// [`current`] as one line, for `--version` and terminal headers.
pub fn label() -> &'static str {
    static LABEL: LazyLock<String> = LazyLock::new(|| current().to_string());
    LABEL.as_str()
}

pub fn run(args: Args) {
    let build = current();
    if args.json {
        let revision = build.revision.map(|revision| revision.full());
        println!("{}", serde_json::json!({ "version": build.version, "revision": revision, "channel": build.channel }));
    } else {
        println!("{}", label());
    }
}

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    const COMMIT: &str = "766330b73c0b0e5a4c1f9d2e8a7b6c5d4e3f2a1b";

    #[test]
    fn formats_a_published_build() {
        let build = Build::parse("0.1.2", COMMIT, "nightly");
        assert_eq!(build.revision, Some(Revision { commit: COMMIT, dirty: false }));
        assert_eq!(build.to_string(), "0.1.2 (766330b7 nightly)");
    }

    #[test]
    fn marks_a_dirty_checkout() {
        let embedded = format!("{COMMIT}-dirty");
        let build = Build::parse("0.1.2", &embedded, "");
        let revision = build.revision.unwrap();
        assert!(revision.dirty);
        assert_eq!(revision.full(), embedded);
        assert_eq!(build.to_string(), "0.1.2 (766330b7-dirty)");
    }

    #[test]
    fn reports_an_unknown_source() {
        for embedded in ["unknown", "", "not-a-commit", "766330", &"a".repeat(65)] {
            let build = Build::parse("0.1.2", embedded, "");
            assert_eq!(build.revision, None, "{embedded}");
            assert_eq!(build.to_string(), "0.1.2 (unknown)");
        }
    }

    #[test]
    fn keeps_a_short_commit_whole() {
        let revision = Revision::parse("766330b").unwrap();
        assert_eq!(revision.short(), "766330b");
        assert_eq!(revision.full(), "766330b");
    }

    #[test]
    fn embeds_a_parsable_revision() {
        let embedded = env!("DOKIME_SOURCE_REVISION");
        assert_eq!(current().revision.is_some(), embedded != UNKNOWN, "{embedded}");
    }
}
