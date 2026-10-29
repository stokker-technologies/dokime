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
        let definition_revisions = {
            let state = self.state_read();
            cases
                .iter()
                .map(|case| state.case_definition_revision(case).map(|revision| (case.test_case_id.clone(), revision)))
                .collect::<Result<_, String>>()
                .map_err(crate::runtime::Error::Operation)?
        };
        self.register_cases_at(registration_id, cases, definition_revisions).await
    }

    pub async fn register_cases_at(
        &self, registration_id: dokime_domain::model::identity::CommandId, cases: Vec<crate::definition::TestCase>,
        definition_revisions: std::collections::BTreeMap<dokime_domain::model::identity::CaseId, String>,
    ) -> crate::runtime::Result<()> {
        if cases.is_empty() {
            return Err(crate::runtime::Error::Operation("registration requires at least one case".into()));
        }
        let permit = self.admit_owned_work().ok_or(crate::runtime::Error::ShuttingDown)?;
        let runtime = self.clone();
        tokio::spawn(async move {
            let _permit = permit;
            for case in &cases {
                prepare_case(case)?;
            }
            let ids = cases.iter().map(|case| case.test_case_id.clone()).collect::<Vec<_>>();
            runtime
                .commit(crate::transactions::RegisterCases {
                    registration_id,
                    cases,
                    definition_revisions,
                    max_cases: 100_000,
                })
                .await
                .map_err(|error| crate::runtime::Error::Operation(error.to_string()))?;

            for id in ids {
                let stored = runtime
                    .state_manager
                    .state()
                    .read(move |state| {
                        let case =
                            state.suites.case(&id).ok_or_else(|| crate::runtime::Error::Operation("registered case is absent".into()))?;
                        prepare_case(case)
                    })
                    .await
                    .map_err(|error| crate::runtime::Error::Operation(error.to_string()))?
                    .value?;
                runtime.storage().write_object(&*stored).await.map_err(|error| crate::runtime::Error::Operation(error.to_string()))?;
            }
            Ok(())
        })
        .await
        .map_err(|error| crate::runtime::Error::Operation(error.to_string()))?
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn prepare_case(
    case: &crate::definition::TestCase,
) -> crate::runtime::Result<harmos::storage::ReadBatch<crate::storage::objects::case::Case>> {
    use crate::storage::objects::case::Case;
    let seed = harmos::storage::ReadBatch::new(());
    seed.try_map(|_| Case::try_from(case).map_err(|error| crate::runtime::Error::Operation(error.to_string())))
}

async fn settings(
    runtime: &crate::RuntimeHandle, sources: impl IntoIterator<Item = String>,
) -> crate::runtime::Result<std::collections::BTreeMap<String, crate::settings::SettingValues>> {
    let sources = sources.into_iter().collect::<std::collections::BTreeSet<_>>();
    let state = runtime.state_read();
    let mut result = std::collections::BTreeMap::new();
    for source in sources {
        if let Some(artifact) = state.installed_plugins.get(&source) {
            result.insert(source, runtime.settings_snapshot(artifact).await?);
        }
    }
    Ok(result)
}

/* === Tests === */
