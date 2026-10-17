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
use std::marker::PhantomData;

// Third Party Imports
use harmos::plugin::inventory::Section;
use harmos::plugin::{Registrar, RegistrationError};

// Local Imports
use crate::catalog::{self, Catalog};
use crate::plugin::registration::{
    ANALYSES_NAMESPACE, LISTENERS_NAMESPACE, OPTIONS_NAMESPACE, RECORDS_NAMESPACE, SETTINGS_NAMESPACE, SUITES_NAMESPACE,
    SettingsRegistration, TELEMETRY_NAMESPACE, domain_schema,
};
use crate::plugin::settings::PluginSettings;

/* === Constants === */

/* === Macros === */
macro_rules! section {
    ($namespace:expr, $payload:literal) => {
        Section::new($namespace, $payload)
    };
}

/* === Type Aliases === */

/* === Structs === */
pub struct Suites<S>(PhantomData<fn() -> S>);

pub struct Records<R, L = ()>(PhantomData<fn() -> (R, L)>);

pub struct Evidence<S, A = ()>(PhantomData<fn() -> (S, A)>);

pub struct Settings<S>(PhantomData<fn() -> S>);

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl<S> harmos::plugin::Contribution for Suites<S>
where
    S: Catalog<catalog::Suites> + 'static,
{
    const NAMESPACE: &'static str = SUITES_NAMESPACE;
    const SECTION: Section = section!(SUITES_NAMESPACE, r#"{"schema":2}"#);

    fn register(registrar: &mut Registrar) -> Result<(), RegistrationError> {
        let definitions = catalog::entries::<S, catalog::Suites>().into_iter().map(|definition| definition()).collect::<Vec<_>>();
        if definitions.is_empty() {
            return Ok(());
        }
        register(registrar, SUITES_NAMESPACE, &crate::definition::SuiteSet::new(definitions))
    }
}

impl<R, L> harmos::plugin::Contribution for Records<R, L>
where
    R: Catalog<catalog::Records> + 'static,
    L: Catalog<catalog::Listeners> + 'static,
{
    const NAMESPACE: &'static str = RECORDS_NAMESPACE;
    const SECTION: Section = section!(RECORDS_NAMESPACE, r#"{"schema":1}"#);

    fn register(registrar: &mut Registrar) -> Result<(), RegistrationError> {
        let records = catalog::entries::<R, catalog::Records>().into_iter().map(|contract| contract()).collect::<Vec<_>>();
        let mut seen = std::collections::BTreeSet::new();
        for record in &records {
            if !seen.insert(record.record_type.clone()) {
                return Err(invalid(RECORDS_NAMESPACE, format!("record `{}` is registered more than once", record.record_type)));
            }
        }
        if !records.is_empty() {
            register(registrar, RECORDS_NAMESPACE, &records)?;
        }
        let listeners = catalog::entries::<L, catalog::Listeners>()
            .into_iter()
            .map(|entry| crate::plugin::registration::ListenerRegistration {
                operation: entry.operation.to_owned(),
                record: entry.record.to_owned(),
            })
            .collect::<Vec<_>>();
        if !listeners.is_empty() {
            register(registrar, LISTENERS_NAMESPACE, &listeners)?;
        }
        Ok(())
    }
}

impl<S, A> harmos::plugin::Contribution for Evidence<S, A>
where
    S: Catalog<catalog::StreamContracts> + 'static,
    A: Catalog<crate::analysis::Definitions> + 'static,
{
    const NAMESPACE: &'static str = TELEMETRY_NAMESPACE;
    const SECTION: Section = section!(TELEMETRY_NAMESPACE, r#"{"schema":1}"#);

    fn register(registrar: &mut Registrar) -> Result<(), RegistrationError> {
        let streams = catalog::entries::<S, catalog::StreamContracts>().into_iter().map(|contract| contract()).collect::<Vec<_>>();
        if !streams.is_empty() {
            register(registrar, TELEMETRY_NAMESPACE, &streams)?;
        }
        let analyses = catalog::entries::<A, crate::analysis::Definitions>()
            .into_iter()
            .map(|definition| definition())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| invalid(ANALYSES_NAMESPACE, error))?;
        if !analyses.is_empty() {
            register(registrar, ANALYSES_NAMESPACE, &analyses)?;
        }
        Ok(())
    }
}

impl<S> harmos::plugin::Contribution for Settings<S>
where
    S: PluginSettings,
{
    const NAMESPACE: &'static str = SETTINGS_NAMESPACE;
    const SECTION: Section = section!(SETTINGS_NAMESPACE, r#"{"schema":3}"#);

    fn register(registrar: &mut Registrar) -> Result<(), RegistrationError> {
        let settings = SettingsRegistration {
            catalog: S::catalog(),
            status_entrypoint: S::status_operation().map(|name| crate::domains::PluginEntrypoint { name: name.to_owned() }),
            discoveries: S::discoveries(),
        };
        settings.catalog.validate().map_err(|error| invalid(SETTINGS_NAMESPACE, error))?;
        if !settings.catalog.contributions.is_empty() || settings.status_entrypoint.is_some() {
            register(registrar, SETTINGS_NAMESPACE, &settings)?;
        }
        let options = S::option_providers();
        if !options.is_empty() {
            register(registrar, OPTIONS_NAMESPACE, &options)?;
        }
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn register<T: serde::Serialize>(registrar: &mut Registrar, namespace: &str, value: &T) -> Result<(), RegistrationError> {
    registrar.register(namespace, domain_schema(namespace), value)
}

fn invalid(namespace: &str, error: impl std::fmt::Display) -> RegistrationError {
    RegistrationError::Encode { namespace: namespace.into(), source: <serde_json::Error as serde::ser::Error>::custom(error.to_string()) }
}

/* === Tests === */
