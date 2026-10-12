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

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
// `clap::Args` is derived here rather than mirrored by a second struct in the

// Cargo extension: `cargo dokime build`'s flags *are* these fields, and a

// hand-written mapper between two identical shapes has only drift to offer.

// `long_about = None` keeps this module's rationale out of `--help`.

#[derive(Clone, Debug, Default, clap::Args)]
#[command(
    about = "Compile a native plugin and freeze its catalog and assets into a .dokime bundle",
    long_about = None
)]
pub struct Options {
    #[arg(long)]
    pub manifest_path: Option<std::path::PathBuf>,
    #[arg(short, long)]
    pub package: Option<String>,
    #[arg(
        long = "bin",
        value_name = "BIN",
        help = "Native binaries to include; empty selects the package's default binary."
    )]
    pub binaries: Vec<String>,
    #[arg(
        short = 'F',
        long,
        help = "Features to activate; repeat or use comma/space-separated names."
    )]
    pub features: Vec<String>,
    #[arg(long, help = "Activate all available features.")]
    pub all_features: bool,
    #[arg(long, help = "Disable default features.")]
    pub no_default_features: bool,
    #[arg(long, help = "Require Cargo.lock to remain unchanged.")]
    pub locked: bool,
    #[arg(long, help = "Resolve and compile without accessing the network.")]
    pub offline: bool,
    #[arg(long)]
    pub target: Option<String>,
    #[arg(long, conflicts_with = "profile")]
    pub release: bool,
    #[arg(long)]
    pub profile: Option<String>,
    #[arg(long)]
    pub target_dir: Option<std::path::PathBuf>,
    #[arg(long, value_name = "SECONDS", value_parser = deadline, help = "Optional deadline in seconds for each Cargo invocation (metadata and compilation).")]
    pub cargo_timeout: Option<std::time::Duration>,
    #[arg(short, long)]
    pub output: Option<std::path::PathBuf>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Options {
    pub fn cargo_options(&self) -> Vec<String> {
        let mut options = Vec::new();
        for features in &self.features {
            options.extend(["--features".to_owned(), features.clone()]);
        }
        for (enabled, flag) in [
            (self.all_features, "--all-features"),
            (self.no_default_features, "--no-default-features"),
            (self.locked, "--locked"),
            (self.offline, "--offline"),
        ] {
            if enabled {
                options.push(flag.to_owned());
            }
        }
        options
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn deadline(value: &str) -> Result<std::time::Duration, String> {
    value
        .parse::<std::num::NonZeroU64>()
        .map(|seconds| std::time::Duration::from_secs(seconds.get()))
        .map_err(|error| error.to_string())
}

/* === Tests === */
