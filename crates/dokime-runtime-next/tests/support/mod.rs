/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution Host

/* === Modules === */

mod settings;

/* === Re-Exports === */

pub use settings::{SYSTEM_HANG_GUARD, settings};

/* === Imports === */

// Standard Library Imports
use std::path::PathBuf;

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, PartialEq, prost::Message)]
pub struct Reading {
    #[prost(double, tag = "1")]
    pub value: f64,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct Attempt {
    #[prost(double, tag = "1")]
    pub value: f64,

    #[prost(uint32, tag = "2")]
    pub attempt: u32,

    #[prost(bool, tag = "3")]
    pub scope_refused: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn thermal_fixture() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let path = std::env::var_os("DOKIME_THERMAL_PLUGIN").map(PathBuf::from).unwrap_or_else(|| {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/debug")
            .join(format!("dokime-rust-thermal{}", std::env::consts::EXE_SUFFIX))
    });
    if !path.is_file() {
        return Err("Build the Shared Thermal Example Before Running the Host Stories".into());
    }
    Ok(path)
}
pub fn execution_fixture() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/debug")
        .join(format!("dokime-rust-execution{}", std::env::consts::EXE_SUFFIX));
    if !path.is_file() {
        return Err("Build the Execution Example with the SDK Task Before Running the Host Stories".into());
    }
    Ok(path)
}
pub fn parameters(value: f64, fail_attempts: u32, evaluate: bool) -> std::collections::HashMap<String, dokime_protocol::v1::Value> {
    use dokime_protocol::v1::{Value, value::Kind};
    std::collections::HashMap::from([
        ("value".into(), Value { kind: Some(Kind::Number(value)) }),
        (
            "fail_attempts".into(),
            Value {
                kind: Some(Kind::UnsignedInteger(fail_attempts as u64)),
            },
        ),
        ("evaluate".into(), Value { kind: Some(Kind::Boolean(evaluate)) }),
    ])
}
pub fn answer(
    host: &dokime_runtime_next::Host, execution: &str, prompt: dokime_protocol::v1::OperatorQuestion,
) -> dokime_runtime_next::Result<()> {
    host.answer(
        execution,
        dokime_protocol::v1::OperatorAnswer {
            prompt: prompt.prompt,
            accepted: true,
            action: "continue".into(),
            ..Default::default()
        },
    )
}

pub fn host_fixture() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let executable = std::env::var_os("NEXTEST_BIN_EXE_dokime-host-next")
        .or_else(|| option_env!("CARGO_BIN_EXE_dokime-host-next").map(Into::into))
        .ok_or("Build the New Host Binary Before Running Its Client Stories")?;
    Ok(PathBuf::from(executable))
}

/* === Private Functions === */

/* === Tests === */
