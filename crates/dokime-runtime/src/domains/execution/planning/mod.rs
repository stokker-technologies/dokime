/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub(crate) mod append;
pub(crate) mod copy;
pub(crate) mod create;
pub(crate) mod policy;

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl crate::RuntimeHandle {
    pub async fn register_cases(
        &self, registration_id: dokime_domain::model::identity::CommandId, cases: Vec<crate::definition::TestCase>,
    ) -> crate::runtime::Result<()> {
        if cases.is_empty() || cases.len() > 1000 {
            return Err(crate::runtime::Error::Operation("registration requires 1 to 1000 cases".into()));
        }
        let permit = self.admit_owned_work().ok_or(crate::runtime::Error::ShuttingDown)?;
        let runtime = self.clone();
        tokio::spawn(async move {
            let _permit = permit;
            for case in &cases {
                crate::storage::objects::case::Case::try_from(case).map_err(|error| crate::runtime::Error::Operation(error.to_string()))?;
            }
            let ids = cases.iter().map(|case| case.test_case_id.clone()).collect::<Vec<_>>();
            runtime
                .commit(crate::transactions::RegisterCases { registration_id, cases })
                .await
                .map_err(|error| crate::runtime::Error::Operation(error.to_string()))?;

            for id in ids {
                let case = runtime
                    .state_manager
                    .state()
                    .read(move |state| state.suites.case(&id).cloned())
                    .await
                    .map_err(|error| crate::runtime::Error::Operation(error.to_string()))?
                    .value
                    .ok_or_else(|| crate::runtime::Error::Operation("registered case is absent".into()))?;
                let stored = crate::storage::objects::case::Case::try_from(&case)
                    .map_err(|error| crate::runtime::Error::Operation(error.to_string()))?;
                runtime.storage().write_object(&stored).await.map_err(|error| crate::runtime::Error::Operation(error.to_string()))?;
            }
            Ok(())
        })
        .await
        .map_err(|error| crate::runtime::Error::Operation(error.to_string()))?
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn settings(
    runtime: &crate::RuntimeHandle, sources: impl IntoIterator<Item = String>,
) -> crate::runtime::Result<std::collections::BTreeMap<String, crate::settings::SettingValues>> {
    let sources = sources.into_iter().collect::<std::collections::BTreeSet<_>>();
    let state = runtime.state_read();
    sources
        .into_iter()
        .filter_map(|source| {
            state.installed_plugins.get(&source).map(|artifact| runtime.settings_snapshot(artifact).map(|values| (source, values)))
        })
        .collect()
}

/* === Tests === */
