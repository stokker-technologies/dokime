/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Test Plugins Use The Ordinary Verified Bundle And SDK Lifecycle, With Duplex
// Channels In Place Of A Spawned Executable.

use std::{
    any::Any,
    collections::BTreeMap,
    future::Future,
    num::NonZeroUsize,
    path::Path,
    pin::Pin,
    sync::{Arc, Mutex, OnceLock, Weak},
};

use harmos::sidecar::{Attach, Attached, Executable, PluginNative, StreamDeclaration};

type Connect = dyn Fn() -> Pin<Box<dyn Future<Output = std::io::Result<Attached>> + Send>> + Send + Sync;

#[derive(Clone)]
pub(crate) struct NativeAttachment {
    connect: Arc<Connect>,
    binding: Option<(String, Arc<dyn Any + Send + Sync>)>,
}

#[derive(Clone)]
pub struct NativeSource {
    executable: Executable,
    attachment: Option<NativeAttachment>,
}

impl NativeSource {
    pub(crate) fn new(executable: Executable, attachment: Option<NativeAttachment>) -> Self {
        Self { executable, attachment }
    }

    pub(crate) fn declaration(&self) -> &harmos::plugin::inventory::Declaration {
        self.executable.declaration()
    }
}

impl Attach for NativeSource {
    type Error = String;

    fn owner(&self) -> &str {
        self.executable.owner()
    }

    fn publications(&self) -> Vec<StreamDeclaration> {
        self.executable.publications()
    }

    fn observers(&self) -> Vec<harmos::plugin::inventory::observers::Declaration> {
        self.executable.observers()
    }

    fn operations(&self) -> harmos::domain::Result<Vec<harmos::sidecar::operations::OperationDefinition>> {
        self.executable.operations()
    }

    fn operation_contracts(&self) -> harmos::domain::Result<Option<Vec<harmos::sidecar::operations::Contract>>> {
        self.executable.operation_contracts()
    }

    fn required_contracts(&self) -> harmos::domain::Result<Option<Vec<harmos::plugin::inventory::contracts::Required>>> {
        self.executable.required_contracts()
    }

    fn sdk_lifecycle(&self) -> harmos::domain::Result<Option<[harmos::domain::model::operations::lifecycle::Descriptor; 2]>> {
        self.executable.sdk_lifecycle()
    }

    fn observe_contract(&self) -> harmos::domain::Result<Option<harmos::sidecar::operations::Contract>> {
        self.executable.observe_contract()
    }

    fn listener_declarations(&self) -> harmos::domain::Result<Option<Vec<harmos::domain::listeners::Definition>>> {
        self.executable.listener_declarations()
    }

    async fn attach(&self) -> Result<Attached, String> {
        match &self.attachment {
            Some(attachment) => (attachment.connect)().await.map_err(|error| error.to_string()),
            None => self.executable.attach().await.map_err(|error| error.to_string()),
        }
    }
}

pub fn plugin<P>(mut config: crate::RuntimeConfig, directory: &Path) -> crate::runtime::Result<crate::RuntimeConfig>
where
    P: harmos::Plugin<Context = harmos::sidecar::Supervisor> + harmos::plugin::Declared,
{
    use harmos::plugin::inventory::{Declaration, Facility};
    use harmos_packaging::bundle::{Bundle, Identity, Limits, PackageMetadata, Source};

    let fail = |error: &dyn std::fmt::Display| crate::runtime::Error::Operation(error.to_string());
    let markers = P::markers_with_settings(&config.schema).map_err(|error| fail(&error))?.concat();
    let declaration = Declaration::inspect_with_settings(&markers, Facility::Sidecar, &config.schema).map_err(|error| fail(&error))?;
    let mut registrar = harmos::sidecar::Registrar::new();
    P::register(&mut registrar).map_err(|error| fail(&error))?;
    let bundle = Bundle::build_with_settings(
        Identity {
            name: format!("fixture-{}", P::ID),
            version: declaration.version,
            description: declaration.description,
            package: PackageMetadata::default(),
        },
        &[Source {
            path: "bin/fixture",
            bytes: &markers,
            executable: Some(harmos_packaging::bundle::Executable {
                facility: harmos_packaging::bundle::Facility::Sidecar,
                target: crate::plugins::NATIVE_TARGET.into(),
                registrations: registrar.finish(),
            }),
        }],
        Limits::default(),
        &config.schema,
    )
    .map_err(|error| fail(&error))?;
    std::fs::create_dir_all(directory)?;
    let path = directory.join(format!("{}.dokime", P::ID));
    std::fs::write(&path, bundle.bytes())?;
    let native = PluginNative::<P>::new_with_settings(NonZeroUsize::new(128).expect("positive capacity"), &config.schema)?;
    config.native_fixtures.insert(
        P::ID.into(),
        NativeAttachment {
            connect: Arc::new(move || {
                let native = native.clone();
                Box::pin(async move { native.attach().await })
            }),
            binding: None,
        },
    );
    config.plugins.push(crate::plugins::PluginBundle::new(path));
    Ok(config)
}

pub fn bound_plugin<P, B>(config: crate::RuntimeConfig, directory: &Path, binding: Arc<B>) -> crate::runtime::Result<crate::RuntimeConfig>
where
    P: harmos::Plugin<Context = harmos::sidecar::Supervisor> + harmos::plugin::Declared,
    B: Any + Send + Sync,
{
    let mut config = plugin::<P>(config, directory)?;
    let key = uuid::Uuid::now_v7().to_string();
    let binding: Arc<dyn Any + Send + Sync> = binding;
    let mut registry = bindings().lock().expect("fixture bindings lock");
    registry.retain(|_, value| value.strong_count() > 0);
    registry.insert(key.clone(), Arc::downgrade(&binding));
    config.native_fixtures.get_mut(P::ID).expect("fixture was inserted").binding = Some((key, binding));
    Ok(config)
}

pub fn resource<B: Any + Send + Sync>(config: &crate::plugin::Configuration) -> Result<Arc<B>, harmos::sidecar::Refusal> {
    let key: String = config.setting("__fixture_binding").map_err(harmos::sidecar::Refusal::of)?;
    bindings()
        .lock()
        .expect("fixture bindings lock")
        .get(&key)
        .and_then(Weak::upgrade)
        .ok_or_else(|| harmos::sidecar::Refusal::of("fixture binding was released"))?
        .downcast::<B>()
        .map_err(|_| harmos::sidecar::Refusal::of("fixture binding has another type"))
}

pub(crate) fn configure(runtime: &crate::RuntimeConfig, owner: &str, config: &mut crate::plugin::Configuration) {
    if let Some((key, _)) = runtime.native_fixtures.get(owner).and_then(|source| source.binding.as_ref()) {
        config.settings.insert("__fixture_binding".into(), key.clone().into());
    }
}

fn bindings() -> &'static Mutex<BTreeMap<String, Weak<dyn Any + Send + Sync>>> {
    static BINDINGS: OnceLock<Mutex<BTreeMap<String, Weak<dyn Any + Send + Sync>>>> = OnceLock::new();
    BINDINGS.get_or_init(Mutex::default)
}
