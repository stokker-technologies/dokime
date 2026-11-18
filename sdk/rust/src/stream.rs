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
use std::{marker::PhantomData, sync::Arc, time::Duration};

// Third Party Imports
use prost_reflect::{DescriptorPool, DynamicMessage};

// Local Imports
use crate::operation::{self, Action, FutureResult};
use crate::{Context, Message, Result, context::Executions};
use harmos;
use harmos::plugin;

use harmos::plugin::Publisher;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

type Producer<T> = Box<dyn FnOnce(plugin::Stream<T>, Executions) -> plugin::Stream<T> + Send>;
pub(crate) type Registration = Box<dyn FnOnce(plugin::Plugin, Executions, &str) -> plugin::Plugin + Send>;

/* === Structs === */

// An execution or plugin-wide stream with one protobuf message per sample.
pub struct Stream<T: Message> {
    pub(crate) identifier: String,
    description: String,
    marker: PhantomData<fn() -> T>,
    producer: Option<Producer<T>>,
}

pub struct Observer {
    register: Box<dyn FnOnce(Executions, &str) -> plugin::Observer + Send>,
    configure: Vec<Box<dyn FnOnce(plugin::Observer) -> plugin::Observer + Send>>,
    pub(crate) sampler: Sampler,
}

// The Same Observer Handler, Retained for In-Process Sampling Without a Host.
#[derive(Clone)]
pub(crate) struct Sampler {
    pub stream: String,
    pub interval: Option<Duration>,
    pub setting: Option<String>,
    pub types: prost_types::FileDescriptorSet,
    pub name: String,
    pub sample: Arc<dyn Sample>,
}

/// Answers One Granted Window With One Encoded Row per Sample.
pub(crate) trait Sample: Send + Sync {
    fn rows<'a>(&'a self, context: &'a Context, window: &'a plugin::Window) -> FutureResult<'a, Vec<Vec<u8>>>;
}

struct Encoded<T>(Action<T>);

struct Columns<T, C, H> {
    handler: Arc<H>,
    marker: PhantomData<fn() -> (T, C)>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl<T: Message> Stream<T> {
    pub fn of<H: Fn() -> Self>(_: H, label: impl Into<String>) -> Self {
        Self::new(crate::authoring::code_name::<H>(), label)
    }

    pub fn new(identifier: impl Into<String>, label: impl Into<String>) -> Self {
        let identifier = identifier.into();
        Self {
            description: label.into(),
            marker: PhantomData,
            identifier,
            producer: None,
        }
    }
    pub fn identifier(&self) -> &str {
        &self.identifier
    }
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }
    pub fn publish_on<H>(mut self, handler: H) -> Self
    where
        H: for<'a> AsyncFn(&'a Context, Publisher<T>) -> Result<()> + Send + Sync + 'static,
    {
        self.producer = Some(Box::new(move |stream, executions| {
            stream.publish_on(async move |transport: &plugin::Context, publisher| {
                let context = Context::publication(transport, &executions)?;
                handler(&context, publisher).await.map_err(harmos::Error::from)
            })
        }));
        self
    }
    pub(crate) fn registration(self) -> Registration {
        Box::new(move |plugin, executions, prefix| {
            let identifier = crate::authoring::build_identifier(prefix, &self.identifier, false);
            let stream = plugin::Stream::new(identifier).description(self.description);
            let stream = match self.producer {
                Some(producer) => producer(stream, executions),
                None => stream,
            };
            plugin.stream(stream)
        })
    }
}

impl<T: Message> AsRef<str> for Stream<T> {
    fn as_ref(&self) -> &str {
        &self.identifier
    }
}

impl Observer {
    /// Observe One Sample per Call; Each Granted Window Calls It Once per Sample, in Order.
    pub fn new<T, H>(stream: &Stream<T>, handler: H) -> Self
    where
        T: Message,
        H: for<'a> AsyncFn(&'a Context) -> Result<T> + Send + Sync + 'static,
    {
        let identifier = stream.identifier.clone();
        let action = operation::action(handler);
        let sampler = Sampler::new::<T>(&identifier, Arc::new(Encoded(action.clone())));
        Self {
            register: Box::new(move |executions, prefix| {
                let identifier = crate::authoring::build_identifier(prefix, &identifier, false);
                let stream = plugin::Stream::<T>::new(identifier);
                plugin::Observer::new(&stream, async move |transport: &plugin::Context| {
                    let context = Context::publication(transport, &executions)?;
                    action.execute(&context).await.map_err(harmos::Error::from)
                })
            }),
            configure: Vec::new(),
            sampler,
        }
    }

    /// Observe a Whole Window per Call; `C` Repeats Every Field of `T` With Its Number and Type.
    ///
    /// Sample `i` of the window is due at `window.start + i * window.period`; the returned
    /// columns hold one value per sample in every field.
    pub fn window<T, C, H>(stream: &Stream<T>, handler: H) -> Self
    where
        T: Message,
        C: Message,
        H: for<'a> AsyncFn(&'a Context, &'a plugin::Window) -> Result<C> + Send + Sync + 'static,
    {
        let identifier = stream.identifier.clone();
        let handler = Arc::new(handler);
        let sampler = Sampler::new::<T>(
            &identifier,
            Arc::new(Columns::<T, C, H> {
                handler: handler.clone(),
                marker: PhantomData,
            }),
        );
        Self {
            register: Box::new(move |executions, prefix| {
                let identifier = crate::authoring::build_identifier(prefix, &identifier, false);
                let stream = plugin::Stream::<T>::new(identifier);
                plugin::Observer::window(&stream, async move |transport: &plugin::Context, window: &plugin::Window| {
                    let context = Context::publication(transport, &executions)?;
                    handler(&context, window).await.map_err(harmos::Error::from)
                })
            }),
            configure: Vec::new(),
            sampler,
        }
    }

    /// Sample at a Fixed Interval on the Host Clock.
    pub fn every(mut self, interval: Duration) -> Self {
        self.sampler.interval = Some(interval);
        self.configure(move |observer| {
            observer.every(prost_types::Duration {
                seconds: i64::try_from(interval.as_secs()).unwrap_or(i64::MAX),
                nanos: interval.subsec_nanos() as i32,
            })
        })
    }

    /// Take the Interval From a Plugin Duration Setting.
    pub fn interval_setting(mut self, identifier: impl Into<String>) -> Self {
        let identifier = identifier.into();
        self.sampler.setting = Some(identifier.clone());
        self.configure(move |observer| observer.interval_setting(identifier))
    }

    /// Take This Stream's Window Length From a Plugin Duration Setting Instead of the Host's.
    pub fn window_setting(self, identifier: impl Into<String>) -> Self {
        let identifier = identifier.into();
        self.configure(move |observer| observer.window_setting(identifier))
    }

    fn configure(mut self, configure: impl FnOnce(plugin::Observer) -> plugin::Observer + Send + 'static) -> Self {
        self.configure.push(Box::new(configure));
        self
    }

    pub(crate) fn registration(self) -> Registration {
        Box::new(move |plugin, executions, prefix| {
            let mut observer = (self.register)(executions, prefix);
            for configure in self.configure {
                observer = configure(observer);
            }
            plugin.observer(observer)
        })
    }
}

impl Sampler {
    fn new<T: Message>(stream: &str, sample: Arc<dyn Sample>) -> Self {
        Self {
            stream: stream.to_owned(),
            interval: None,
            setting: None,
            types: T::types(),
            name: T::name(),
            sample,
        }
    }
}

impl<T: Message> Sample for Encoded<T> {
    fn rows<'a>(&'a self, context: &'a Context, window: &'a plugin::Window) -> FutureResult<'a, Vec<Vec<u8>>> {
        Box::pin(async move {
            let mut rows = Vec::with_capacity(usize::try_from(window.count).unwrap_or_default());
            for _ in 0..window.count {
                rows.push(self.0.execute(context).await?.encode_to_vec());
            }
            Ok(rows)
        })
    }
}

impl<T, C, H> Sample for Columns<T, C, H>
where
    T: Message,
    C: Message,
    H: for<'a> AsyncFn(&'a Context, &'a plugin::Window) -> Result<C> + Send + Sync + 'static,
{
    fn rows<'a>(&'a self, context: &'a Context, window: &'a plugin::Window) -> FutureResult<'a, Vec<Vec<u8>>> {
        Box::pin(async move {
            let columns = (self.handler)(context, window).await?.encode_to_vec();
            columns_rows::<T, C>(&columns, window.count)
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */

/// Split Encoded Columns Into One Encoded Row per Sample, Refusing a Column of Another Length.
fn columns_rows<T: Message, C: Message>(columns: &[u8], count: u64) -> Result<Vec<Vec<u8>>> {
    let invalid = |error: String| crate::Error::Invalid(format!("Answer Sample Window: {error}"));
    let descriptor = |types, name: String| -> Result<prost_reflect::MessageDescriptor> {
        let pool = DescriptorPool::from_file_descriptor_set(types).map_err(|error| invalid(error.to_string()))?;
        pool.get_message_by_name(&name).ok_or_else(|| invalid(format!("declare the message {name}")))
    };
    let (row, layout) = (descriptor(T::types(), T::name())?, descriptor(C::types(), C::name())?);
    let columns = DynamicMessage::decode(layout.clone(), columns).map_err(|error| invalid(error.to_string()))?;
    let count = usize::try_from(count).map_err(|error| invalid(error.to_string()))?;
    let mut rows = vec![DynamicMessage::new(row); count];
    for field in layout.fields() {
        let value = columns.get_field(&field);
        let Some(values) = value.as_list().filter(|values| values.len() == count) else {
            return Err(invalid(format!("send {count} values in column `{}`", field.name())));
        };
        for (row, value) in rows.iter_mut().zip(values) {
            row.try_set_field_by_number(field.number(), value.clone()).map_err(|error| invalid(error.to_string()))?;
        }
    }
    let rows = rows.iter().map(prost::Message::encode_to_vec).collect();
    Ok(rows)
}

/* === Tests === */
