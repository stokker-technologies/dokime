/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Typed Plugin Activation Resources

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::future::Future;
use std::mem;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

use tokio::sync;

use crate::operation::FutureResult;
use crate::{Error, Result, operations};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

type Shared = Arc<dyn Any + Send + Sync>;
type Open = for<'a> fn(&'a operations::Context) -> FutureResult<'a, Shared>;
type Close = fn(Shared) -> FutureResult<'static, ()>;

/* === Structs === */

struct Entry {
    value: sync::Mutex<Option<Shared>>,
    open: Open,
    close: Close,
}

#[derive(Default)]
pub(crate) struct Resources {
    entries: HashMap<TypeId, Entry>,
    opened: Mutex<Vec<TypeId>>,
    closed: AtomicBool,
}

/* === Enums === */

/* === Traits === */

/// A Lazily Opened Resource Shared by One Plugin Activation.
pub trait Resource: Send + Sync + Sized + 'static {
    fn open(context: &operations::Context) -> impl Future<Output = Result<Self>> + Send;
    fn close(&self) -> impl Future<Output = Result<()>> + Send {
        async { Ok(()) }
    }
}

/* === Implementations === */

impl Resources {
    pub(crate) fn register<T: Resource>(&mut self) -> Result<()> {
        if self.entries.contains_key(&TypeId::of::<T>()) {
            return Err(Error::Invalid("Declare Resources: List Each Type Once".into()));
        }
        self.entries.insert(
            TypeId::of::<T>(),
            Entry {
                value: sync::Mutex::new(None),
                open: open::<T>,
                close: close::<T>,
            },
        );
        Ok(())
    }

    pub(crate) fn activate(&self) {
        self.closed.store(false, Ordering::SeqCst);
    }

    pub(crate) async fn get<T: Resource>(&self, context: &operations::Context) -> Result<Arc<T>> {
        let kind = TypeId::of::<T>();
        let entry = self.entries.get(&kind).ok_or_else(|| Error::Invalid("Open Resource: Declare the Type on the Plugin".into()))?;
        let mut value = entry.value.lock().await;
        if self.closed.load(Ordering::SeqCst) {
            return Err(Error::Invalid("Open Resource: Restart the Deactivating Plugin".into()));
        }
        if value.is_none() {
            *value = Some((entry.open)(context).await?);
            self.opened
                .lock()
                .map_err(|_| Error::Invalid("Open Resource: Restart the Plugin After a Poisoned Resource Store".into()))?
                .push(kind);
        }
        value
            .as_ref()
            .expect("Opened Resource")
            .clone()
            .downcast()
            .map_err(|_| Error::Invalid("Open Resource: Register the Matching Resource Type".into()))
    }

    pub(crate) async fn close(&self) -> Result<()> {
        self.closed.store(true, Ordering::SeqCst);
        // Wait for Every In-Flight Open Before Taking the Release Order.
        for entry in self.entries.values() {
            drop(entry.value.lock().await);
        }
        let opened = mem::take(
            &mut *self
                .opened
                .lock()
                .map_err(|_| Error::Invalid("Close Resources: Restart the Plugin After a Poisoned Resource Store".into()))?,
        );
        let mut failures = Vec::new();
        for kind in opened.into_iter().rev() {
            let entry = &self.entries[&kind];
            let value = entry.value.lock().await.take();
            if let Some(value) = value
                && let Err(error) = (entry.close)(value).await
            {
                failures.push(error.to_string());
            }
        }
        if failures.is_empty() {
            return Ok(());
        }
        Err(Error::Invalid(format!("Close Resources: Resolve the Release Failures, Then Restart: {}", failures.join("; "))))
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn open<T: Resource>(context: &operations::Context) -> FutureResult<'_, Shared> {
    Box::pin(async move { Ok(Arc::new(T::open(context).await?) as Shared) })
}

fn close<T: Resource>(value: Shared) -> FutureResult<'static, ()> {
    Box::pin(async move {
        let resource = value.downcast::<T>().map_err(|_| Error::Invalid("Close Resource: Register the Matching Resource Type".into()))?;
        resource.close().await
    })
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::{Resource, Resources};
    use crate::context::{Context, Execution, Local, LocalHost};
    use crate::operation::FutureResult;
    use crate::{Error, Origin, Result, TestCase, operations};
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    #[derive(Default)]
    struct Probe {
        opened: AtomicUsize,
        closed: AtomicUsize,
    }
    struct Connection(Arc<Probe>);
    struct Host;

    impl LocalHost for Host {
        fn call<'a>(&'a self, _: &'static str, _: Vec<u8>) -> FutureResult<'a, Vec<u8>> {
            Box::pin(async { Err(Error::Invalid("Test Resource: No Host Call Was Expected".into())) })
        }
    }

    impl Resource for Connection {
        async fn open(context: &operations::Context) -> Result<Self> {
            let probe = context.plugin::<Arc<Probe>>()?.clone();
            probe.opened.fetch_add(1, Ordering::SeqCst);
            Ok(Self(probe))
        }
        async fn close(&self) -> Result<()> {
            self.0.closed.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    #[tokio::test]
    async fn resources_share_one_instance_and_release_it_per_activation() -> Result<()> {
        let probe = Arc::new(Probe::default());
        let mut resources = Resources::default();
        resources.register::<Connection>()?;
        assert!(resources.register::<Connection>().is_err());
        let resources = Arc::new(resources);
        let mut inner = Context::local(
            Local {
                host: Arc::new(Host),
                scope: String::new(),
                attempt: 1,
            },
            Execution::new(TestCase::default(), Origin::default()),
            Arc::new(probe.clone()),
        );
        inner.resources = resources.clone();
        let context = operations::Context::from_inner(inner);
        let first = context.resource::<Connection>().await?;
        let second = context.resource::<Connection>().await?;
        assert!(Arc::ptr_eq(&first, &second));
        assert_eq!(probe.opened.load(Ordering::SeqCst), 1);
        resources.close().await?;
        assert_eq!(probe.closed.load(Ordering::SeqCst), 1);
        assert!(context.resource::<Connection>().await.is_err());
        resources.activate();
        let third = context.resource::<Connection>().await?;
        assert!(!Arc::ptr_eq(&first, &third));
        resources.close().await?;
        assert_eq!(probe.opened.load(Ordering::SeqCst), 2);
        assert_eq!(probe.closed.load(Ordering::SeqCst), 2);
        Ok(())
    }
}
