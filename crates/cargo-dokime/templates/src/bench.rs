/* ***
 *
 * {{package}}
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::collections::BTreeMap;
use std::sync::{Arc, LazyLock, Mutex, PoisonError};
// Third Party Imports

// Local Imports

/* === Constants === */
const RESPONSE: f64 = 0.5;

const MAXIMUM_C: f64 = 150.0;

static CHAMBERS: LazyLock<Mutex<BTreeMap<String, Arc<Mutex<Chamber>>>>> = LazyLock::new(|| Mutex::new(BTreeMap::new()));

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Chamber {
    temperature_c: f64,
    target_c: f64,
    powered: bool,
}

pub struct Sensor {
    port: String,
    chamber: Arc<Mutex<Chamber>>,
}

pub struct Heater {
    port: String,
    chamber: Arc<Mutex<Chamber>>,
}

pub struct Identity {
    pub serial_number: String,
    pub firmware_version: String,
}

/* === Enums === */
#[derive(Debug, thiserror::Error)]
pub enum BenchError {
    #[error("the chamber cannot hold {temperature_c} °C")]
    Unreachable { temperature_c: f64 },
    #[error("no chamber is open on {port}")]
    Unopened { port: String },
}

/* === Traits === */

/* === Implementations === */
impl Sensor {
    pub fn open(port: impl Into<String>, ambient_c: f64) -> Self {
        let port = port.into();
        let chamber = Arc::clone(registry().entry(port.clone()).or_insert_with(|| {
            Arc::new(Mutex::new(Chamber {
                temperature_c: ambient_c,
                target_c: ambient_c,
                powered: false,
            }))
        }));

        Self { port, chamber }
    }

    pub fn read_celsius(&self) -> Result<f64, BenchError> {
        let mut chamber = self.chamber.lock().unwrap_or_else(PoisonError::into_inner);
        let target = if chamber.powered { chamber.target_c } else { chamber.temperature_c };
        chamber.temperature_c += (target - chamber.temperature_c) * RESPONSE;

        Ok(chamber.temperature_c)
    }

    pub fn port(&self) -> &str {
        &self.port
    }

    pub fn describe(&self) -> String {
        format!("simulated sensor on {}", self.port)
    }

    pub fn close(&self) {
        rest(&self.chamber);
    }
}

impl Heater {
    pub async fn open(port: impl Into<String>) -> Result<Self, BenchError> {
        let port = port.into();
        let chamber = registry().get(&port).map(Arc::clone).ok_or_else(|| BenchError::Unopened { port: port.clone() })?;

        Ok(Self { port, chamber })
    }

    pub async fn identity(&self) -> Result<Identity, BenchError> {
        Ok(Identity {
            serial_number: format!("SIM-{}", self.port.trim_start_matches('/').replace('/', "-")),
            firmware_version: "sim-1.0".to_owned(),
        })
    }

    pub async fn set_target(&self, temperature_c: f64) -> Result<(), BenchError> {
        if !temperature_c.is_finite() || temperature_c > MAXIMUM_C {
            return Err(BenchError::Unreachable { temperature_c });
        }
        let mut chamber = self.chamber.lock().unwrap_or_else(PoisonError::into_inner);
        chamber.target_c = temperature_c;
        chamber.powered = true;

        Ok(())
    }

    pub async fn off(&self) -> Result<(), BenchError> {
        rest(&self.chamber);

        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn registry() -> std::sync::MutexGuard<'static, BTreeMap<String, Arc<Mutex<Chamber>>>> {
    CHAMBERS.lock().unwrap_or_else(PoisonError::into_inner)
}

fn rest(chamber: &Mutex<Chamber>) {
    chamber.lock().unwrap_or_else(PoisonError::into_inner).powered = false;
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn refused_target_unchanged() {
        let sensor = Sensor::open("unit/refused-target", 22.0);
        let heater = Heater::open(sensor.port()).await.unwrap();
        heater.set_target(80.0).await.unwrap();
        let before = {
            let chamber = sensor.chamber.lock().unwrap();
            (chamber.target_c, chamber.powered, chamber.temperature_c)
        };
        for refused in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 151.0] {
            assert!(matches!(heater.set_target(refused).await, Err(BenchError::Unreachable { .. })));
            let chamber = sensor.chamber.lock().unwrap();
            assert_eq!((chamber.target_c, chamber.powered, chamber.temperature_c), before);
        }
        heater.off().await.unwrap();
    }

    #[tokio::test]
    async fn closed_heater_steady() {
        let sensor = Sensor::open("unit/heater-off", 22.0);
        let heater = Heater::open(sensor.port()).await.unwrap();
        heater.set_target(80.0).await.unwrap();
        let warmed = sensor.read_celsius().unwrap();
        assert!(warmed > 22.0 && warmed < 80.0);
        heater.off().await.unwrap();
        assert_eq!(sensor.read_celsius().unwrap(), warmed);
        assert_eq!(sensor.read_celsius().unwrap(), warmed);
    }
}
