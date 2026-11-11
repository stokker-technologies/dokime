/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// In-Process Charger Operation Fixtures

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::BTreeMap;
use std::sync::Arc;

// Third Party Imports

// Local Imports
use dokime::execution::Answering;
use dokime::harmos::operations::OperationDeclaration;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) fn charger_answers() -> BTreeMap<String, Answering> {
    use plugin_obc::operations::charger::*;
    use plugin_obc::operations::wait::Wait;
    let devices = Arc::new(Chargers::default());
    let mut answers = BTreeMap::new();
    let owner = devices.clone();
    let (id, answer) = answering_result::<InitializeCharger, _>(move |request| owner.initialize(request.supply, request.logging));
    answers.insert(id, answer);
    let owner = devices.clone();
    let (id, answer) = answering_result::<StartCharger, _>(move |request| owner.start(request.device_id));
    answers.insert(id, answer);
    let owner = devices.clone();
    let (id, answer) = answering_result::<ReadCharger, _>(move |request| owner.read(request.device_id));
    answers.insert(id, answer);
    let owner = devices.clone();
    let (id, answer) = answering_result::<SampleCharger, _>(move |request| owner.sample(request.device_id, request.elapsed_s));
    answers.insert(id, answer);
    let owner = devices.clone();
    let (id, answer) = answering_result::<SetChargerLoad, _>(move |request| owner.set_load(request.device_id, request.load_ohm));
    answers.insert(id, answer);
    let owner = devices.clone();
    let (id, answer) = answering_result::<InjectChargerFault, _>(move |request| owner.inject_fault(request.device_id, request.fault));
    answers.insert(id, answer);
    let owner = devices.clone();
    let (id, answer) = answering_result::<ClearChargerFault, _>(move |request| owner.clear_fault(request.device_id));
    answers.insert(id, answer);
    let owner = devices.clone();
    let (id, answer) = answering_result::<ProbeChargerRestart, _>(move |request| owner.probe_restart(request.device_id));
    answers.insert(id, answer);
    let owner = devices.clone();
    let (id, answer) = answering_result::<StopCharger, _>(move |request| owner.stop(request.device_id));
    answers.insert(id, answer);
    let owner = devices.clone();
    let (id, answer) = answering_result::<ReleaseCharger, _>(move |request| owner.release(request.device_id));
    answers.insert(id, answer);
    let owner = devices.clone();
    let (id, answer) = answering_result::<SampleChargerBatch, _>(move |request| request.sample(&owner));
    answers.insert(id, answer);
    let (id, answer) = answering_result::<EncodeChargerSummary, _>(|request| request.encode().map_err(plugin_obc::Error::from));
    answers.insert(id, answer);
    answers.insert(
        Wait::ID.into(),
        Arc::new(|payload: Vec<u8>| {
            Box::pin(async move {
                let request = <Wait as dokime::harmos::Message>::decode(payload.as_slice()).map_err(|error| error.to_string())?;
                tokio::time::sleep(std::time::Duration::from_secs_f64(request.duration_s)).await;
                Ok(dokime::harmos::Message::encode(&()))
            }) as std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<u8>, String>> + Send>>
        }),
    );
    answers
}

/* === Private Functions === */

fn answering_result<O, F>(answer: F) -> (String, Answering)
where
    O: OperationDeclaration + dokime::harmos::Message,
    O::Response: dokime::harmos::Message,
    F: Fn(O) -> Result<O::Response, plugin_obc::Error> + Send + Sync + 'static,
{
    let answer = Arc::new(answer);
    (
        O::ID.to_owned(),
        Arc::new(move |payload: Vec<u8>| {
            let answer = answer.clone();
            Box::pin(async move {
                let request = <O as dokime::harmos::Message>::decode(payload.as_slice()).map_err(|error| error.to_string())?;
                let response = answer(request).map_err(|error| error.to_string())?;
                Ok(dokime::harmos::Message::encode(&response))
            }) as std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<u8>, String>> + Send>>
        }),
    )
}

/* === Tests === */
