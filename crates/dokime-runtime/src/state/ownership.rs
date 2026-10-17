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

// Third Party Imports
use serde::{Deserialize, Serialize};
// Local Imports
use super::{InstalledPlugin, RuntimeState};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BundleCatalog {
    pub package: String,
    pub path: String,
    pub installation: Option<String>,
    #[serde(default)]
    pub content_digest: String,
    #[serde(alias = "artifacts")]
    pub plugins: Vec<InstalledPlugin>,
}

/* === Enums === */
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum PluginOwner {
    Host,
    Embedded {
        generation: String,
    },
    Bundle {
        package: String,
        path: String,
        installation: Option<String>,
        #[serde(default)]
        content_digest: String,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "catalog")]
pub enum ProviderCatalog {
    Bundle(BundleCatalog),
    Embedded {
        #[serde(alias = "artifact")]
        plugin: Box<InstalledPlugin>,
        generation: String,
    },
}

/* === Traits === */

/* === Implementations === */
impl RuntimeState {
    pub(crate) fn execution_identity(
        &self,
        source: &str,
    ) -> Option<dokime_domain::model::execution::snapshot::ExecutionPluginIdentity> {
        use dokime_domain::model::execution::snapshot::ExecutionImplementation;
        use dokime_domain::model::execution::snapshot::ExecutionPluginIdentity;

        let implementation = match self.plugin_owners.get(source)? {
            PluginOwner::Host => return None,
            PluginOwner::Bundle { content_digest, .. } => ExecutionImplementation::Bundle {
                content_digest: content_digest.clone(),
            },
            PluginOwner::Embedded { generation } => ExecutionImplementation::Embedded {
                generation: generation.clone(),
            },
        };

        Some(ExecutionPluginIdentity {
            source: source.to_owned(),
            version: self
                .installed_plugins
                .get(source)
                .map(|plugin| plugin.version.clone()),
            implementation,
        })
    }
}

impl From<BundleCatalog> for ProviderCatalog {
    fn from(bundle: BundleCatalog) -> Self {
        Self::Bundle(bundle)
    }
}

impl ProviderCatalog {
    pub fn id(&self) -> &str {
        match self {
            Self::Bundle(bundle) => &bundle.package,
            Self::Embedded { plugin, .. } => &plugin.source,
        }
    }

    pub fn plugins(&self) -> &[InstalledPlugin] {
        match self {
            Self::Bundle(bundle) => &bundle.plugins,
            Self::Embedded { plugin, .. } => std::slice::from_ref(plugin.as_ref()),
        }
    }

    fn owner(&self) -> PluginOwner {
        match self {
            Self::Bundle(bundle) => PluginOwner::Bundle {
                package: bundle.package.clone(),
                path: bundle.path.clone(),
                installation: bundle.installation.clone(),
                content_digest: bundle.content_digest.clone(),
            },
            Self::Embedded { generation, .. } => PluginOwner::Embedded {
                generation: generation.clone(),
            },
        }
    }

    fn allows_redefinition(&self) -> bool {
        match self {
            Self::Bundle(bundle) => bundle.installation.is_none(),
            Self::Embedded { .. } => true,
        }
    }
}

impl RuntimeState {
    pub fn install_host_plugin(&mut self, installation: InstalledPlugin) -> Result<(), String> {
        if matches!(
            self.plugin_owners.get(&installation.source),
            Some(PluginOwner::Bundle { .. } | PluginOwner::Embedded { .. })
        ) {
            return Err(format!(
                "source `{}` belongs to a registered provider",
                installation.source
            ));
        }
        let source = installation.source.clone();
        self.install_plugin(installation)?;
        self.plugin_owners.insert(source, PluginOwner::Host);
        Ok(())
    }

    /// Whether installing this host plugin would leave the state unchanged.
    /// Boot skips such commits: each one journals the whole installation.
    pub(crate) fn host_plugin_current(&self, installation: &InstalledPlugin) -> bool {
        self.installed_plugins.get(&installation.source) == Some(installation)
            && self.plugin_owners.get(&installation.source) == Some(&PluginOwner::Host)
    }

    /// Whether reconciling these catalogs would leave the state unchanged.
    /// Boot skips such commits: each one journals every catalog in full.
    pub(crate) fn providers_current<C: Clone + Into<ProviderCatalog>>(
        &self,
        catalogs: &[C],
    ) -> bool {
        self.provider_candidate(catalogs, None)
            .is_ok_and(|candidate| candidate == *self)
    }

    pub fn reconcile_providers<C: Clone + Into<ProviderCatalog>>(
        &mut self,
        catalogs: &[C],
    ) -> Result<(), String> {
        *self = self.provider_candidate(catalogs, None)?;
        Ok(())
    }

    pub fn validate_providers<C: Clone + Into<ProviderCatalog>>(
        &self,
        catalogs: &[C],
    ) -> Result<(), String> {
        self.provider_candidate(catalogs, None).map(|_| ())
    }

    pub(crate) fn validate_provider_reload<C: Clone + Into<ProviderCatalog>>(
        &self,
        catalogs: &[C],
        package: &str,
    ) -> Result<(), String> {
        self.provider_candidate(catalogs, Some(package)).map(|_| ())
    }

    pub(crate) fn reload_provider_catalog<C: Clone + Into<ProviderCatalog>>(
        &mut self,
        catalogs: &[C],
        package: &str,
    ) -> Result<(), String> {
        *self = self.provider_candidate(catalogs, Some(package))?;
        Ok(())
    }

    fn provider_candidate<C: Clone + Into<ProviderCatalog>>(
        &self,
        catalogs: &[C],
        redefine: Option<&str>,
    ) -> Result<Self, String> {
        let providers: Vec<ProviderCatalog> = catalogs.iter().cloned().map(Into::into).collect();
        if let Some(package) = redefine
            && !providers
                .iter()
                .any(|provider| provider.id() == package && provider.allows_redefinition())
        {
            return Err("record redefinition requires a configured or embedded provider".into());
        }
        let mut candidate = self.clone();
        let old: Vec<_> = candidate
            .installed_plugins
            .keys()
            .filter(|source| {
                !matches!(
                    candidate.plugin_owners.get(*source),
                    Some(PluginOwner::Host)
                )
            })
            .cloned()
            .collect();
        for source in old {
            candidate.remove_plugin(&source)?;
            candidate.plugin_owners.remove(&source);
        }
        let mut sources = std::collections::BTreeSet::new();
        let mut identities = std::collections::BTreeSet::new();
        for provider in &providers {
            if !identities.insert(provider.id()) {
                return Err(format!("duplicate provider `{}`", provider.id()));
            }
            if let ProviderCatalog::Embedded { generation, .. } = provider
                && generation.trim().is_empty()
            {
                return Err("an embedded provider requires a registration generation".into());
            }
            for plugin in provider.plugins() {
                if !sources.insert(&plugin.source)
                    || candidate.plugin_owners.contains_key(&plugin.source)
                {
                    return Err(format!(
                        "provider source `{}` is already owned",
                        plugin.source
                    ));
                }
                candidate.install_plugin_with_redefinition(
                    plugin.clone(),
                    if redefine == Some(provider.id()) {
                        super::Redefinition::Supersede
                    } else {
                        super::Redefinition::Refuse
                    },
                )?;
                candidate
                    .plugin_owners
                    .insert(plugin.source.clone(), provider.owner());
            }
        }
        // Asked once, of the finished catalog. A template may name a stream
        // another plugin publishes, so the question has no answer until every
        // plugin in the desired set is in — and the removals above mean the
        // states in between are not ones to judge. Asking inside the rebuild
        // instead would refuse every reconcile of two plugins that read each
        // other's streams, and make a plugin another one reads from
        // impossible to remove.
        candidate.admit_requested_streams()?;

        Ok(candidate)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
