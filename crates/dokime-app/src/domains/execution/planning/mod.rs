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
        &self,
        registration_id: dokime::core::CommandId,
        cases: Vec<crate::definition::TestCase>,
    ) -> crate::Result<()> {
        if cases.is_empty() || cases.len() > 1000 {
            return Err(crate::Error::Operation(
                "registration requires 1 to 1000 cases".into(),
            ));
        }
        let permit = self
            .owned_admissions
            .clone()
            .try_acquire_owned()
            .map_err(|_| {
                crate::Error::overload(
                    "case_admission_busy",
                    "case admission is at capacity; retry later",
                )
            })?;
        let runtime = self.clone();
        tokio::spawn(async move {
            let _permit = permit;
            for case in &cases {
                dokime::storage::objects::case::Case::try_from(case)
                    .map_err(|error| crate::Error::Operation(error.to_string()))?;
            }
            let ids = cases
                .iter()
                .map(|case| case.test_case_id.clone())
                .collect::<Vec<_>>();
            runtime
                .commit(crate::transactions::RegisterCases {
                    registration_id,
                    cases,
                })
                .await
                .map_err(|error| crate::Error::Operation(error.to_string()))?;

            for id in ids {
                let case = runtime
                    .state_manager
                    .state()
                    .read(move |state| state.suites.case(&id).cloned())
                    .await
                    .map_err(|error| crate::Error::Operation(error.to_string()))?
                    .value
                    .ok_or_else(|| crate::Error::Operation("registered case is absent".into()))?;
                let stored = dokime::storage::objects::case::Case::try_from(&case)
                    .map_err(|error| crate::Error::Operation(error.to_string()))?;
                runtime
                    .storage()
                    .write_object(&stored)
                    .await
                    .map_err(|error| crate::Error::Operation(error.to_string()))?;
            }
            Ok(())
        })
        .await
        .map_err(|error| crate::Error::Operation(error.to_string()))?
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn settings(
    runtime: &crate::RuntimeHandle,
    sources: impl IntoIterator<Item = String>,
) -> crate::Result<std::collections::BTreeMap<String, dokime::settings::SettingValues>> {
    let sources = sources
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>();
    let state = runtime.state_read();
    sources
        .into_iter()
        .filter_map(|source| {
            state.installed_plugins.get(&source).map(|artifact| {
                runtime
                    .settings_snapshot(artifact)
                    .map(|values| (source, values))
            })
        })
        .collect()
}

/* === Tests === */
