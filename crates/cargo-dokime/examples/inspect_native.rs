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
use std::{hint::black_box, num::NonZeroUsize, path::PathBuf, time::Instant};
// Third Party Imports
use clap::Parser;
// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Parser)]
struct Args {
    executable: PathBuf,
    #[arg(long, default_value = "3")]
    samples: NonZeroUsize,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let bytes = std::fs::read(&args.executable)?;
    let mut durations = Vec::with_capacity(args.samples.get());
    for sample in 1..=args.samples.get() {
        let started = Instant::now();
        let declaration = harmos::artifact::inventory::Declaration::inspect(
            black_box(&bytes),
            harmos::artifact::inventory::Facility::Sidecar,
        )?;
        let elapsed = started.elapsed().as_secs_f64();
        durations.push(elapsed);
        println!(
            "Sample {sample}: {} bytes · {} · {elapsed:.3}s",
            bytes.len(),
            declaration.id
        );
    }
    durations.sort_by(f64::total_cmp);
    let middle = durations.len() / 2;
    let median = if durations.len().is_multiple_of(2) {
        (durations[middle - 1] + durations[middle]) / 2.0
    } else {
        durations[middle]
    };
    println!("Median inspection: {median:.3}s (file read excluded)");
    Ok(())
}

/* === Tests === */
