/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Stream

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Context, Message, Publisher, Result, context::Executions};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

type Producer<T> = Box<dyn FnOnce(harmos::Stream<T>, Executions) -> harmos::Stream<T> + Send>;
pub(crate) type Registration = Box<dyn FnOnce(harmos::Plugin, Executions) -> harmos::Plugin + Send>;

/* === Structs === */

// An execution or plugin-wide stream with one protobuf message per sample.
pub struct Stream<T: Message> {
    pub(crate) identifier: String,
    pub(crate) inner: harmos::Stream<T>,
    producer: Option<Producer<T>>,
}

pub struct Observer {
    register: Box<dyn FnOnce(Executions) -> harmos::Observer + Send>,
    configure: Vec<Box<dyn FnOnce(harmos::Observer) -> harmos::Observer + Send>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl<T: Message> Stream<T> {
    pub fn new(identifier: impl Into<String>, label: impl Into<String>) -> Self {
        let identifier = identifier.into();
        Self {
            inner: harmos::Stream::new(identifier.clone()).description(label),
            identifier,
            producer: None,
        }
    }
    pub fn identifier(&self) -> &str {
        &self.identifier
    }
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.inner = self.inner.description(description);
        self
    }
    pub fn publish_on<H>(mut self, handler: H) -> Self
    where
        H: for<'a> AsyncFn(&'a Context, Publisher<T>) -> Result<()> + Send + Sync + 'static,
    {
        self.producer = Some(Box::new(move |stream, executions| {
            stream.publish_on(async move |transport: &harmos::Context, publisher| {
                let context = Context::publication(transport, &executions)?;
                handler(&context, publisher).await.map_err(harmos::Error::from)
            })
        }));
        self
    }
    pub(crate) fn registration(self) -> Registration {
        Box::new(move |plugin, executions| {
            let stream = match self.producer {
                Some(producer) => producer(self.inner, executions),
                None => self.inner,
            };
            plugin.stream(stream)
        })
    }
}

impl Observer {
    pub fn new<T, H>(stream: &Stream<T>, handler: H) -> Self
    where
        T: Message,
        H: for<'a> AsyncFn(&'a Context) -> Result<T> + Send + Sync + 'static,
    {
        let identifier = stream.identifier.clone();
        Self {
            register: Box::new(move |executions| {
                let stream = harmos::Stream::<T>::new(identifier);
                let observer = harmos::Observer::new(&stream, async move |transport: &harmos::Context| {
                    let context = Context::publication(transport, &executions)?;
                    handler(&context).await.map_err(harmos::Error::from)
                });
                observer
            }),
            configure: Vec::new(),
        }
    }

    pub fn every_ms(self, interval_ms: u64) -> Self {
        self.configure(move |observer| observer.every_ms(interval_ms))
    }

    pub fn interval_setting(self, identifier: impl Into<String>) -> Self {
        let identifier = identifier.into();
        self.configure(move |observer| observer.interval_setting(identifier))
    }

    fn configure(mut self, configure: impl FnOnce(harmos::Observer) -> harmos::Observer + Send + 'static) -> Self {
        self.configure.push(Box::new(configure));
        self
    }

    pub(crate) fn registration(self) -> Registration {
        Box::new(move |plugin, executions| {
            let mut observer = (self.register)(executions);
            for configure in self.configure {
                observer = configure(observer);
            }
            plugin.observer(observer)
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
