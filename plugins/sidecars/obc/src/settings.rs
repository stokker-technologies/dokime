/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Bench Settings: Deployment Shape That Does Not Vary From Case to Case

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::path::PathBuf;
use std::time::Duration;

// Third Party Imports

// Local Imports
use dokime::templates::Context;
use dokime::{Error, Result, SettingDeclaration, SettingValue, setting_value};

/* === Constants === */

pub const SAMPLE_INTERVAL: &str = "sample_interval";
pub const IMAGES_DIRECTORY: &str = "images_directory";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn catalog() -> [SettingDeclaration; 2] {
    [
        SettingDeclaration {
            minimum: Some(SettingValue {
                value: Some(setting_value::Value::Duration(prost_types::Duration { seconds: 0, nanos: 1 })),
            }),
            maximum: None,
            choices: Vec::new(),
            required: false,
            name: SAMPLE_INTERVAL.into(),
            title: "Sample Interval".into(),
            description: "Host time between two observer samples; the simulated model advances one step per sample.".into(),
            default: Some(SettingValue {
                value: Some(setting_value::Value::Duration(prost_types::Duration { seconds: 0, nanos: 1_000_000 })),
            }),
        },
        SettingDeclaration {
            minimum: None,
            maximum: None,
            choices: Vec::new(),
            required: false,
            name: IMAGES_DIRECTORY.into(),
            title: "Reference Images Directory".into(),
            description: "Directory containing the optional reference.png the start-up evidence step attaches.".into(),
            default: Some(SettingValue {
                value: Some(setting_value::Value::Text(".".into())),
            }),
        },
    ]
}

/// The Effective Observer Cadence for This Plugin Entry.
pub async fn sample_interval(ctx: &Context) -> Result<Duration> {
    match value(ctx, SAMPLE_INTERVAL).await? {
        setting_value::Value::Duration(interval) => Duration::try_from(interval)
            .map_err(|error| Error::Invalid(format!("Read Sample Interval: Configure a Valid Duration: {error}"))),
        _ => Err(Error::Invalid("Read Sample Interval: Configure a Duration in the Plugin Settings".into())),
    }
}

pub async fn images_directory(ctx: &Context) -> Result<PathBuf> {
    match value(ctx, IMAGES_DIRECTORY).await? {
        setting_value::Value::Text(directory) => Ok(directory.into()),
        _ => Err(Error::Invalid("Read Reference Images Directory: Configure a Path in the Plugin Settings".into())),
    }
}

/* === Private Functions === */

async fn value(ctx: &Context, name: &str) -> Result<setting_value::Value> {
    ctx.setting(name).await?.value.ok_or_else(|| Error::Invalid(format!("Read Setting {name}: Restore Its Value in the Plugin Settings")))
}

/* === Tests === */
