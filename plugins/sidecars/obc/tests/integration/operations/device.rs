/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Manual device state and refusal behavior

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use obc_runtime::{
    Client, Error, Stop, Testbench, app,
    device::{Device, Fault, Power},
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[dokime_tracing::trace_test]
#[tokio::test]
async fn manual_power_shared() {
    let runtime = app::launch(Testbench::default()).await.unwrap();
    let writer = Client::new(runtime.handle());
    let reader = Client::new(runtime.handle());
    assert_eq!(reader.device().await.unwrap().power, Power::Off);

    writer.set_power(true).await.unwrap();
    let powered = reader.device().await.unwrap();
    assert_eq!(powered.power, Power::Running);
    assert_eq!(powered.supply.voltage_rms_v, 230.0);
    assert_eq!(powered.fault, None);

    writer.set_power(false).await.unwrap();
    assert_eq!(reader.device().await.unwrap().power, Power::Off);
    runtime.shutdown().await.unwrap();
}

#[dokime_tracing::trace_test]
#[tokio::test]
async fn stopped_mutations_refused() {
    let runtime = app::launch(Testbench::default()).await.unwrap();
    let client = Client::new(runtime.handle());
    client.set_power(true).await.unwrap();

    runtime.state.commit(Stop).await.unwrap();
    let stopped = client.device().await.unwrap();
    assert_eq!(stopped.power, Power::Off);
    assert!(client.stopped().await.unwrap());
    assert!(matches!(client.set_power(true).await, Err(dokime::harmos::Error::Rejected(Error::Stopped))));
    assert!(matches!(client.set_power(false).await, Err(dokime::harmos::Error::Rejected(Error::Stopped))));
    assert_eq!(client.device().await.unwrap(), stopped);
    runtime.shutdown().await.unwrap();
}

#[dokime_tracing::trace_test]
#[tokio::test]
async fn latched_fault_preserved() {
    let mut device = Device::default();
    device.inject(Fault::Overcurrent);
    let runtime = app::launch(Testbench {
        device: device.clone(),
        ..Default::default()
    })
    .await
    .unwrap();
    let client = Client::new(runtime.handle());

    assert!(matches!(client.set_power(true).await, Err(dokime::harmos::Error::Rejected(Error::FaultLatched))));
    assert_eq!(client.device().await.unwrap(), device);
    client.set_power(false).await.unwrap();
    assert_eq!(client.device().await.unwrap(), device);
    runtime.state.commit(Stop).await.unwrap();
    assert_eq!(client.device().await.unwrap().power, Power::Faulted);
    runtime.shutdown().await.unwrap();
}
