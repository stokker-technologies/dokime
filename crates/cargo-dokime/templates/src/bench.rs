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
use std::collections::BTreeMap;
use std::sync::{Arc, LazyLock, Mutex, PoisonError};
// Third Party Imports

// Local Imports

/* === Constants === */
const RESPONSE: f64 = 0.5;

const MAXIMUM_C: f64 = 150.0;

static CHAMBERS: LazyLock<Mutex<BTreeMap<String, Arc<Mutex<Chamber>>>>> =
    LazyLock::new(|| Mutex::new(BTreeMap::new()));

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Chamber {
    celsius: f64,
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
    pub serial: String,
    pub firmware: String,
}

/* === Enums === */
#[derive(Debug, thiserror::Error)]
pub enum BenchError {
    #[error("the chamber cannot hold {celsius} °C")]
    Unreachable { celsius: f64 },
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
                celsius: ambient_c,
                target_c: ambient_c,
                powered: false,
            }))
        }));

        Self { port, chamber }
    }

                            pub fn read_celsius(&self) -> Result<f64, BenchError> {
        let mut chamber = self.chamber.lock().unwrap_or_else(PoisonError::into_inner);
        let target = if chamber.powered {
            chamber.target_c
        } else {
            chamber.celsius
        };
        chamber.celsius += (target - chamber.celsius) * RESPONSE;

        Ok(chamber.celsius)
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
        let chamber = registry()
            .get(&port)
            .map(Arc::clone)
            .ok_or_else(|| BenchError::Unopened { port: port.clone() })?;

        Ok(Self { port, chamber })
    }

        pub async fn identity(&self) -> Result<Identity, BenchError> {
        Ok(Identity {
            serial: format!(
                "SIM-{}",
                self.port.trim_start_matches('/').replace('/', "-")
            ),
            firmware: "sim-1.0".to_owned(),
        })
    }

                        pub async fn set_target(&self, celsius: f64) -> Result<(), BenchError> {
        if !celsius.is_finite() || celsius > MAXIMUM_C {
            return Err(BenchError::Unreachable { celsius });
        }
        let mut chamber = self.chamber.lock().unwrap_or_else(PoisonError::into_inner);
        chamber.target_c = celsius;
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
    chamber
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .powered = false;
}

/* === Tests === */
