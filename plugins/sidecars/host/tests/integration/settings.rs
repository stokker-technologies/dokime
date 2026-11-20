/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Host Setting Stories: Declared Defaults, Accepted Ranges and CPU Load Relations

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::BTreeSet;
use std::time::Duration;

// Third Party Imports

// Local Imports
use dokime::{Error, SettingValue, setting_value};
use plugin_host::settings::{self, CpuLoad, Kind, SETTINGS};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn value(value: setting_value::Value) -> SettingValue {
    SettingValue { value: Some(value) }
}

fn milliseconds(value: u64) -> SettingValue {
    let value = Duration::from_millis(value);
    self::value(setting_value::Value::Duration(prost_types::Duration {
        seconds: value.as_secs() as i64,
        nanos: value.subsec_nanos() as i32,
    }))
}

fn defaults() -> CpuLoad {
    CpuLoad {
        period: Duration::from_millis(250),
        sample_period: Duration::from_millis(250),
        slice: Duration::from_millis(10),
        proportional_gain: 0.5,
        integral_gain: 0.5,
        integral_min: -1.0,
        integral_max: 1.0,
        settlement: Duration::from_secs(2),
        batch_iterations: 4_096,
    }
}

/* === Tests === */

#[test]
fn every_declared_default_is_unique_and_within_its_own_range() -> dokime::Result<()> {
    let names = SETTINGS.iter().map(|setting| setting.name).collect::<BTreeSet<_>>();
    assert_eq!(names.len(), SETTINGS.len());
    for setting in SETTINGS {
        let declaration = setting.declaration();
        setting.check(declaration.default.as_ref().unwrap())?;
    }
    Ok(())
}

#[test]
fn saved_values_outside_their_range_or_of_another_kind_are_refused() -> dokime::Result<()> {
    // Each Kind Accepts Its Bounds and Refuses One Step Beyond Them.
    for setting in SETTINGS {
        let (inside, outside) = match setting.kind {
            Kind::Milliseconds { min, max, .. } => (vec![milliseconds(min), milliseconds(max)], vec![milliseconds(max + 1)]),
            Kind::Integer { min, max, .. } => (
                vec![value(setting_value::Value::Integer(min)), value(setting_value::Value::Integer(max))],
                vec![value(setting_value::Value::Integer(min - 1)), value(setting_value::Value::Integer(max + 1))],
            ),
            Kind::Number { min, max, default } => {
                let mut outside = vec![value(setting_value::Value::Number(f64::NAN))];
                outside.extend(min.map(|min| value(setting_value::Value::Number(min - 1.0))));
                outside.extend(max.map(|max| value(setting_value::Value::Number(max + 1.0))));
                (vec![value(setting_value::Value::Number(min.unwrap_or(default)))], outside)
            }
            Kind::Text { .. } => (vec![value(setting_value::Value::Text(String::new()))], vec![value(setting_value::Value::Flag(true))]),
        };
        for accepted in inside {
            setting.check(&accepted)?;
        }
        for refused in outside {
            assert!(matches!(setting.check(&refused), Err(Error::Invalid(_))), "{}", setting.name);
        }
        assert!(matches!(setting.check(&SettingValue::default()), Err(Error::Invalid(_))), "{}", setting.name);
    }
    assert!(settings::setting("retired_setting").is_err());
    Ok(())
}

#[test]
fn cpu_load_settings_must_fit_one_controller_period_and_the_cancellation_grace() -> dokime::Result<()> {
    defaults().validate()?;
    let refused = [
        CpuLoad {
            sample_period: Duration::from_millis(251),
            ..defaults()
        },
        CpuLoad {
            slice: Duration::from_millis(251),
            ..defaults()
        },
        CpuLoad {
            settlement: Duration::from_millis(9),
            ..defaults()
        },
        CpuLoad {
            settlement: settings::CPU_LOAD_CANCELLATION_GRACE + Duration::from_millis(1),
            ..defaults()
        },
        CpuLoad { batch_iterations: 0, ..defaults() },
        CpuLoad { proportional_gain: -0.1, ..defaults() },
        CpuLoad {
            integral_gain: f64::INFINITY,
            ..defaults()
        },
        CpuLoad { integral_min: 0.5, ..defaults() },
        CpuLoad { integral_max: -0.5, ..defaults() },
        CpuLoad {
            integral_min: 0.0,
            integral_max: 0.0,
            ..defaults()
        },
    ];
    for load in refused {
        assert!(matches!(load.validate(), Err(Error::Invalid(_))), "{load:?}");
    }
    Ok(())
}
