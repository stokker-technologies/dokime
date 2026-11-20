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
use std::sync::{Arc, Mutex, PoisonError};
use std::{marker::PhantomData, time::Duration};

// Third Party Imports
use prost_reflect::{DescriptorPool, DynamicMessage, MessageDescriptor};
use prost_types::{FileDescriptorSet, Timestamp};

// Local Imports
use crate::authoring;
use crate::operation::{self, Action, FutureResult};
use crate::operations;
use crate::{Context, Error, Message, Result, context::Executions};
use ::harmos::plugin;

use ::harmos::plugin::Publisher;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

type ObserverFactory = Box<dyn FnOnce(Executions, &str) -> plugin::Observer + Send>;
type Producer<T> = Box<dyn FnOnce(plugin::Stream<T>, Executions) -> plugin::Stream<T> + Send>;
pub(crate) type Registration = Box<dyn FnOnce(plugin::Plugin, Executions, &str) -> Result<plugin::Plugin> + Send>;
type Fed = Box<dyn FnOnce(Arc<Feed>) -> Registration + Send>;
type Hook = Arc<dyn Fn(&operations::Context) -> Result<()> + Send + Sync>;

/* === Structs === */

// An execution or plugin-wide stream with one protobuf message per sample.
pub struct Stream<T: Message> {
    pub(crate) identifier: String,
    description: String,
    marker: PhantomData<fn() -> T>,
    producer: Option<Producer<T>>,
}

pub struct Observer {
    register: ObserverFactory,
    configure: Vec<Box<dyn FnOnce(plugin::Observer) -> plugin::Observer + Send>>,
    pub(crate) sampler: Sampler,
    pub(crate) execution: bool,
}

// The Same Observer Handler, Retained for In-Process Sampling Without a Host.
#[derive(Clone)]
pub(crate) struct Sampler {
    pub stream: String,
    pub interval: Option<Duration>,
    pub setting: Option<String>,
    pub types: FileDescriptorSet,
    pub name: String,
    pub sample: Arc<dyn Sample>,
}

/// Answers One Granted Window With One Encoded Row per Sample.
pub(crate) trait Sample: Send + Sync {
    fn rows<'a>(&'a self, context: &'a Context, window: &'a plugin::Window) -> FutureResult<'a, Vec<Vec<u8>>>;
}

struct Encoded<T>(Action<T>);

/// Feeds Several Declared Streams From One Measurement per Tick; Every Row of a Tick Shares Its Timestamp.
///
/// The collection runs only while at least one of its streams has a listener. `on_start` runs when
/// the first listener arrives, before the first tick, and `on_stop` when the last one leaves; no
/// reading is kept after it stops. Each tick may publish zero or more rows to each stream, so a
/// stream whose fields do not exist at that moment stays silent.
///
/// ```
/// use std::time::Duration;
///
/// use dokime::plugin::{Collection, Plugin, Rows, Stream};
/// use dokime::{Result, operations};
///
/// #[derive(dokime::Row)]
/// struct RadioState {
///     connected: bool,
/// }
///
/// #[derive(dokime::Row)]
/// struct RadioLink {
///     signal_percent: f64,
/// }
///
/// async fn collect(_: &operations::Context, rows: &mut Rows) -> Result<()> {
///     let (connected, signal_percent) = (true, 71.0);
///     rows.publish("radio.state", RadioState { connected })?;
///     // The Link Exists Only While Connected, So Its Stream Is Silent Otherwise.
///     if connected {
///         rows.publish("radio.link", RadioLink { signal_percent })?;
///     }
///     Ok(())
/// }
///
/// let collection = Collection::operation(collect)
///     .stream(Stream::<RadioState>::new("radio.state", "Radio State"))
///     .stream(Stream::<RadioLink>::new("radio.link", "Radio Link"))
///     .every(Duration::from_secs(1))
///     .on_start(|_: &operations::Context| Ok(()))
///     .on_stop(|_: &operations::Context| Ok(()));
/// let plugin = Plugin::of::<()>("1.0.0").identifier("radio").collection(collection);
/// assert!(plugin.declarations().is_ok());
/// ```
pub struct Collection {
    collect: Arc<dyn Collect>,
    streams: Vec<Fed>,
    rows: Vec<FedRow>,
    interval: Option<Duration>,
    setting: Option<String>,
    start: Option<Hook>,
    stop: Option<Hook>,
}

/// The Rows One Tick of a Collection Publishes, Each to One of Its Streams.
pub struct Rows {
    rows: Arc<[FedRow]>,
    published: Vec<(String, Vec<u8>)>,
}

/// One Stream a Collection Feeds: Its Identifier and Row Type.
#[derive(Clone)]
pub(crate) struct FedRow {
    pub stream: String,
    pub types: FileDescriptorSet,
    pub name: String,
}

/// The Shared Demand of One Collection: Its Listeners, Its Latest Tick and the Lock Serializing Ticks.
pub(crate) struct Feed {
    collect: Arc<dyn Collect>,
    pub rows: Arc<[FedRow]>,
    pub interval: Option<Duration>,
    setting: Option<String>,
    start: Option<Hook>,
    stop: Option<Hook>,
    listeners: Mutex<usize>,
    latest: Mutex<Option<Arc<Tick>>>,
    generation: Mutex<u64>,
    ticking: tokio::sync::Mutex<()>,
}

/// One Measurement Fanned Out to the Collection's Streams.
pub(crate) struct Tick {
    pub generation: u64,
    pub at: Timestamp,
    pub rows: Vec<(String, Vec<u8>)>,
}

/// One Listener of a Collection; Dropping the Last One Stops the Collection.
pub(crate) struct Listener {
    feed: Arc<Feed>,
    context: operations::Context,
}

struct Collector<H>(H);

struct Columns<T, C, H> {
    handler: Arc<H>,
    marker: PhantomData<fn() -> (T, C)>,
}

/* === Enums === */

/* === Traits === */

trait Collect: Send + Sync {
    fn collect<'a>(&'a self, context: &'a operations::Context, rows: &'a mut Rows) -> FutureResult<'a, ()>;
}

/* === Implementations === */

impl<T: Message> Stream<T> {
    pub fn of<H: Fn() -> Self>(_: H, label: impl Into<String>) -> Self {
        Self::new(authoring::code_name::<H>(false), label)
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
                handler(&context, publisher).await.map_err(::harmos::Error::from)
            })
        }));
        self
    }
    pub(crate) fn registration(self) -> Registration {
        Box::new(move |plugin, executions, prefix| {
            let identifier = authoring::build_identifier(prefix, &self.identifier, false);
            let stream = plugin::Stream::new(identifier).description(self.description);
            let stream = match self.producer {
                Some(producer) => producer(stream, executions),
                None => stream,
            };
            plugin.register(|registrar| registrar.register(stream)).map_err(Error::from)
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
        Self::sample(stream, handler, true)
    }

    /// Observe Plugin-Owned State Without Exposing Template Capabilities.
    pub fn operation<T, H>(stream: &Stream<T>, handler: H) -> Self
    where
        T: Message,
        H: for<'a> AsyncFn(&'a operations::Context) -> Result<T> + Send + Sync + 'static,
    {
        Self::sample(
            stream,
            async move |inner: &Context| {
                let context = operations::Context::from_inner(inner.clone());
                handler(&context).await
            },
            false,
        )
    }

    fn sample<T, H>(stream: &Stream<T>, handler: H, execution: bool) -> Self
    where
        T: Message,
        H: for<'a> AsyncFn(&'a Context) -> Result<T> + Send + Sync + 'static,
    {
        let identifier = stream.identifier.clone();
        let action = operation::action(handler);
        let sampler = Sampler::new::<T>(&identifier, Arc::new(Encoded(action.clone())));
        Self {
            register: Box::new(move |executions, prefix| {
                let identifier = authoring::build_identifier(prefix, &identifier, false);
                let stream = plugin::Stream::<T>::new(identifier);
                plugin::Observer::new(&stream, async move |transport: &plugin::Context| {
                    let context = if execution {
                        Context::publication(transport, &executions)?
                    } else {
                        Context::invocation(transport, &executions)?
                    };
                    action.execute(&context).await.map_err(::harmos::Error::from)
                })
            }),
            configure: Vec::new(),
            sampler,
            execution,
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
        Self::windowed(stream, handler, true)
    }

    /// Answer a Plugin-Wide Window Using Only Operation Capabilities.
    pub fn operation_window<T, C, H>(stream: &Stream<T>, handler: H) -> Self
    where
        T: Message,
        C: Message,
        H: for<'a> AsyncFn(&'a operations::Context, &'a plugin::Window) -> Result<C> + Send + Sync + 'static,
    {
        Self::windowed(
            stream,
            async move |inner: &Context, window: &plugin::Window| {
                let context = operations::Context::from_inner(inner.clone());
                handler(&context, window).await
            },
            false,
        )
    }

    fn windowed<T, C, H>(stream: &Stream<T>, handler: H, execution: bool) -> Self
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
                let identifier = authoring::build_identifier(prefix, &identifier, false);
                let stream = plugin::Stream::<T>::new(identifier);
                plugin::Observer::window(&stream, async move |transport: &plugin::Context, window: &plugin::Window| {
                    let context = if execution {
                        Context::publication(transport, &executions)?
                    } else {
                        Context::invocation(transport, &executions)?
                    };
                    handler(&context, window).await.map_err(::harmos::Error::from)
                })
            }),
            configure: Vec::new(),
            sampler,
            execution,
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
            plugin.register(|registrar| registrar.register(observer)).map_err(Error::from)
        })
    }
}

impl Collection {
    /// Collect One Measurement per Tick Using Only Operation Capabilities.
    pub fn operation<H>(handler: H) -> Self
    where
        H: for<'a> AsyncFn(&'a operations::Context, &'a mut Rows) -> Result<()> + Send + Sync + 'static,
    {
        Self {
            collect: Arc::new(Collector(handler)),
            streams: Vec::new(),
            rows: Vec::new(),
            interval: None,
            setting: None,
            start: None,
            stop: None,
        }
    }

    /// Declare a Stream This Collection Feeds; Rows Published to Any Other Stream Are Refused.
    pub fn stream<T: Message>(mut self, stream: Stream<T>) -> Self {
        let identifier = stream.identifier.clone();
        self.rows.push(FedRow {
            stream: identifier.clone(),
            types: T::types(),
            name: T::name(),
        });
        self.streams.push(Box::new(move |feed: Arc<Feed>| {
            stream
                .publish_on(async move |context: &Context, publisher: Publisher<T>| produce(&feed, &identifier, context, publisher).await)
                .registration()
        }));
        self
    }

    /// Tick at a Fixed Interval on the Host Clock.
    pub fn every(mut self, interval: Duration) -> Self {
        self.interval = Some(interval);
        self
    }

    /// Take the Tick Interval From a Plugin Duration Setting, Read on Every Tick.
    pub fn interval_setting(mut self, identifier: impl Into<String>) -> Self {
        self.setting = Some(identifier.into());
        self
    }

    /// Acquire Native Handles When the First Listener Arrives, Before the First Tick.
    pub fn on_start<H>(mut self, hook: H) -> Self
    where
        H: Fn(&operations::Context) -> Result<()> + Send + Sync + 'static,
    {
        self.start = Some(Arc::new(hook));
        self
    }

    /// Release Native Handles When the Last Listener Leaves.
    pub fn on_stop<H>(mut self, hook: H) -> Self
    where
        H: Fn(&operations::Context) -> Result<()> + Send + Sync + 'static,
    {
        self.stop = Some(Arc::new(hook));
        self
    }

    /// Share One Feed Between the Stream Registrations and the In-Process Harness.
    pub(crate) fn into_parts(self) -> Result<(Arc<Feed>, Vec<Registration>)> {
        if self.rows.is_empty() {
            return Err(Error::Invalid("Declare Collection: Feed at Least One Stream With Collection::stream".into()));
        }
        if self.interval.is_some() == self.setting.is_some() {
            return Err(Error::Invalid("Declare Collection: Give It Exactly One Interval With every or interval_setting".into()));
        }
        let feed = Arc::new(Feed {
            collect: self.collect,
            rows: self.rows.into(),
            interval: self.interval,
            setting: self.setting,
            start: self.start,
            stop: self.stop,
            listeners: Mutex::new(0),
            latest: Mutex::new(None),
            generation: Mutex::new(0),
            ticking: tokio::sync::Mutex::new(()),
        });
        let registrations = self.streams.into_iter().map(|register| register(feed.clone())).collect();
        Ok((feed, registrations))
    }
}

impl Rows {
    /// Publish One Row to a Stream This Collection Feeds, at the Tick's Timestamp.
    pub fn publish<T: Message>(&mut self, stream: impl AsRef<str>, row: T) -> Result<()> {
        let stream = stream.as_ref();
        let Some(fed) = self.rows.iter().find(|fed| fed.stream == stream) else {
            return Err(Error::Invalid(format!("Publish Collection Row: Feed Stream {stream} With Collection::stream")));
        };
        if fed.name != T::name() {
            return Err(Error::Invalid(format!("Publish Collection Row: Stream {stream} Carries {}, Not {}", fed.name, T::name())));
        }
        self.published.push((stream.to_owned(), row.encode_to_vec()));
        Ok(())
    }
}

impl Feed {
    /// Count a Listener In, Starting the Collection for the First One.
    pub(crate) fn join(self: &Arc<Self>, context: operations::Context) -> Result<Listener> {
        if self.enter()
            && let Some(start) = &self.start
            && let Err(error) = start(&context)
        {
            self.leave();
            return Err(error);
        }
        Ok(Listener { feed: self.clone(), context })
    }

    /// The Tick Interval: Fixed, or the Saved Duration Setting.
    pub(crate) async fn interval(&self, context: &operations::Context) -> Result<Duration> {
        match (&self.interval, &self.setting) {
            (Some(interval), _) => Ok(*interval),
            (None, Some(setting)) => context.duration_setting(setting).await,
            (None, None) => Err(Error::Invalid("Declare Collection: Give It an Interval With every or interval_setting".into())),
        }
    }

    /// The Tick Covering `at`: the Latest One While It Is Younger Than the Interval, Otherwise a New Measurement.
    pub(crate) async fn tick(&self, context: &operations::Context, at: Timestamp, interval: Duration) -> Result<Arc<Tick>> {
        let _ticking = self.ticking.lock().await;
        let latest = self.latest.lock().unwrap_or_else(PoisonError::into_inner).clone();
        if let Some(latest) = latest
            && nanos(at) < nanos(latest.at).saturating_add(interval.as_nanos() as i128)
        {
            return Ok(latest);
        }
        let mut rows = Rows {
            rows: self.rows.clone(),
            published: Vec::new(),
        };
        self.collect.collect(context, &mut rows).await?;
        let generation = {
            let mut generation = self.generation.lock().unwrap_or_else(PoisonError::into_inner);
            *generation += 1;
            *generation
        };
        let tick = Arc::new(Tick { generation, at, rows: rows.published });
        // A Reading Never Outlives the Last Listener, Even One That Left During the Measurement.
        if self.listening() {
            *self.latest.lock().unwrap_or_else(PoisonError::into_inner) = Some(tick.clone());
        }
        Ok(tick)
    }

    /// Count a Listener In; True for the First.
    pub(crate) fn enter(&self) -> bool {
        let mut listeners = self.listeners.lock().unwrap_or_else(PoisonError::into_inner);
        *listeners += 1;
        *listeners == 1
    }

    /// Count a Listener Out; True for the Last, Which Also Drops the Latest Reading.
    pub(crate) fn leave(&self) -> bool {
        let mut listeners = self.listeners.lock().unwrap_or_else(PoisonError::into_inner);
        *listeners = listeners.saturating_sub(1);
        let last = *listeners == 0;
        if last {
            self.latest.lock().unwrap_or_else(PoisonError::into_inner).take();
        }
        last
    }

    fn listening(&self) -> bool {
        *self.listeners.lock().unwrap_or_else(PoisonError::into_inner) > 0
    }
}

impl Tick {
    /// This Tick's Rows for One Stream, in Publication Order.
    pub(crate) fn rows<'a>(&'a self, stream: &'a str) -> impl Iterator<Item = &'a [u8]> + 'a {
        self.rows.iter().filter(move |(fed, _)| fed == stream).map(|(_, row)| row.as_slice())
    }
}

impl Drop for Listener {
    fn drop(&mut self) {
        // A Failed Release Has No Caller Left to Answer; the Next Start Acquires Afresh.
        if self.feed.leave()
            && let Some(stop) = &self.feed.stop
        {
            let _ = stop(&self.context);
        }
    }
}

impl<H> Collect for Collector<H>
where
    H: for<'a> AsyncFn(&'a operations::Context, &'a mut Rows) -> Result<()> + Send + Sync + 'static,
{
    fn collect<'a>(&'a self, context: &'a operations::Context, rows: &'a mut Rows) -> FutureResult<'a, ()> {
        Box::pin(async move { (self.0)(context, rows).await })
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

/// Publish This Stream's Rows of Every Tick While Its Demand Lasts; Demand Closing Drops the Listener.
async fn produce<T: Message>(feed: &Arc<Feed>, stream: &str, inner: &Context, publisher: Publisher<T>) -> Result<()> {
    let context = operations::Context::from_inner(inner.clone());
    let _listener = feed.join(operations::Context::from_inner(inner.clone()))?;
    let mut published = 0;
    loop {
        let interval = feed.interval(&context).await?;
        let tick = feed.tick(&context, context.now().await?, interval).await?;
        if tick.generation > published {
            published = tick.generation;
            for row in tick.rows(stream) {
                let row = T::decode(row).map_err(|error| Error::Invalid(format!("Publish Collection Row: {error}")))?;
                // Publication Fails Only When Demand Closed, Which Ends This Listener.
                if publisher.publish_at(row, Some(tick.at)).await.is_err() {
                    return Ok(());
                }
            }
        }
        let due = nanos(tick.at).saturating_add(interval.as_nanos() as i128);
        let wait = u64::try_from(due.saturating_sub(nanos(context.now().await?))).unwrap_or_default();
        context.sleep(Duration::from_nanos(wait)).await?;
    }
}

pub(crate) fn nanos(at: Timestamp) -> i128 {
    i128::from(at.seconds) * 1_000_000_000 + i128::from(at.nanos)
}

/// Split Encoded Columns Into One Encoded Row per Sample, Refusing a Column of Another Length.
fn columns_rows<T: Message, C: Message>(columns: &[u8], count: u64) -> Result<Vec<Vec<u8>>> {
    let invalid = |error: String| Error::Invalid(format!("Answer Sample Window: {error}"));
    let descriptor = |types, name: String| -> Result<MessageDescriptor> {
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::time::Duration;

    use super::{Collection, Observer, Rows, Stream};
    use crate::operations;
    use crate::plugin::Window;
    use crate::templates::{Context, Step, Template};
    use crate::testing::Harness;
    use crate::{Plugin, Result, Suite};

    #[derive(crate::Row)]
    struct Reading {
        value: u32,
    }

    #[derive(crate::Message)]
    struct Columns {
        value: Vec<u32>,
    }

    struct State {
        value: u32,
    }

    #[tokio::test]
    async fn plugin_observers_use_operation_context_and_share_the_author_state() -> Result<()> {
        for windowed in [false, true] {
            let stream = Stream::<Reading>::new("shared.reading", "Shared Reading");
            let observer = if windowed {
                Observer::operation_window(&stream, async |context: &operations::Context, window: &Window| {
                    Ok(Columns {
                        value: vec![context.plugin::<State>()?.value; window.count as usize],
                    })
                })
            } else {
                Observer::operation(&stream, async |context: &operations::Context| Ok(Reading { value: context.plugin::<State>()?.value }))
            }
            .every(Duration::from_millis(1));
            let template = Template::of::<()>()
                .identifier("sample")
                .uses_stream("shared.reading")
                .step(Step::new("Sample", async |context: &Context| context.sleep(Duration::from_millis(3)).await).identifier("sample"));
            let plugin = Plugin::of::<()>("1.0.0")
                .identifier("observer")
                .state(State { value: 23 })
                .stream(stream)
                .observer(observer)
                .suite(Suite::of::<()>().identifier("capture").template(template));
            let report = Harness::new(plugin, "capture.sample").run().await?;
            assert!(report.failure().is_none(), "{report:?}");
            let samples = report.samples::<Reading>("shared.reading")?;
            assert_eq!(samples.iter().map(|sample| sample.value.value).collect::<Vec<_>>(), [23, 23, 23]);
        }
        Ok(())
    }

    #[derive(crate::Row)]
    struct Link {
        level: u32,
    }

    #[derive(Clone, Default)]
    struct Counters {
        ticks: Arc<AtomicU32>,
        starts: Arc<AtomicU32>,
        stops: Arc<AtomicU32>,
    }

    // Every Tick Publishes Two Readings and, on Odd Ticks Only, One Link.
    async fn collect(context: &operations::Context, rows: &mut Rows) -> Result<()> {
        let tick = context.plugin::<Counters>()?.ticks.fetch_add(1, Ordering::SeqCst) + 1;
        rows.publish("shared.reading", Reading { value: tick })?;
        rows.publish("shared.reading", Reading { value: tick * 10 })?;
        if tick % 2 == 1 {
            rows.publish("shared.link", Link { level: tick })?;
        }
        Ok(())
    }

    fn collecting(streams: &[&str], counters: Counters) -> Plugin {
        let collection = Collection::operation(collect)
            .stream(Stream::<Reading>::new("shared.reading", "Shared Reading"))
            .stream(Stream::<Link>::new("shared.link", "Shared Link"))
            .every(Duration::from_millis(1))
            .on_start(|context: &operations::Context| {
                context.plugin::<Counters>()?.starts.fetch_add(1, Ordering::SeqCst);
                Ok(())
            })
            .on_stop(|context: &operations::Context| {
                context.plugin::<Counters>()?.stops.fetch_add(1, Ordering::SeqCst);
                Ok(())
            });
        let mut template = Template::of::<()>().identifier("sample");
        for stream in streams {
            template = template.uses_stream(*stream);
        }
        let template = template
            .step(Step::new("Sample", async |context: &Context| context.sleep(Duration::from_millis(3)).await).identifier("sample"));
        Plugin::of::<()>("1.0.0")
            .identifier("collector")
            .state(counters)
            .collection(collection)
            .suite(Suite::of::<()>().identifier("capture").template(template))
    }

    #[tokio::test]
    async fn a_collection_tick_fans_rows_out_with_one_timestamp_and_a_skipped_stream_stays_silent() -> Result<()> {
        let plugin = collecting(&["shared.reading", "shared.link"], Counters::default());
        let report = Harness::new(plugin, "capture.sample").run().await?;
        assert!(report.failure().is_none(), "{report:?}");
        let readings = report.samples::<Reading>("shared.reading")?;
        let links = report.samples::<Link>("shared.link")?;
        assert_eq!(readings.iter().map(|sample| sample.value.value).collect::<Vec<_>>(), [1, 10, 2, 20, 3, 30]);
        assert_eq!(links.iter().map(|sample| sample.value.level).collect::<Vec<_>>(), [1, 3]);
        assert!(readings.chunks(2).all(|tick| tick[0].at == tick[1].at));
        assert_eq!(links.iter().map(|sample| sample.at).collect::<Vec<_>>(), [readings[0].at, readings[4].at]);
        Ok(())
    }

    #[tokio::test]
    async fn a_collection_starts_with_its_first_listener_and_stops_with_its_last() -> Result<()> {
        let counters = Counters::default();
        let report = Harness::new(collecting(&["shared.link"], counters.clone()), "capture.sample").run().await?;
        assert!(report.failure().is_none(), "{report:?}");
        let counts = [&counters.starts, &counters.stops, &counters.ticks].map(|count| count.load(Ordering::SeqCst));
        assert_eq!(counts, [1, 1, 3]);
        let (feed, _) = Collection::operation(collect)
            .stream(Stream::<Reading>::new("shared.reading", "Shared Reading"))
            .every(Duration::from_millis(1))
            .into_parts()?;
        assert_eq!([feed.enter(), feed.enter(), feed.leave(), feed.leave()], [true, false, false, true]);
        Ok(())
    }

    #[tokio::test]
    async fn nothing_is_collected_without_a_listener() -> Result<()> {
        let counters = Counters::default();
        let report = Harness::new(collecting(&[], counters.clone()), "capture.sample").run().await?;
        assert!(report.failure().is_none(), "{report:?}");
        let counts = [&counters.starts, &counters.stops, &counters.ticks].map(|count| count.load(Ordering::SeqCst));
        assert_eq!(counts, [0, 0, 0]);
        Ok(())
    }

    #[test]
    fn plugin_observers_refuse_template_context_handlers() {
        let stream = Stream::<Reading>::new("shared.reading", "Shared Reading");
        let observer = Observer::new(&stream, async |_: &Context| Ok(Reading { value: 0 })).every(Duration::from_millis(1));
        assert!(Plugin::of::<()>("1.0.0").stream(stream).observer(observer).declarations().is_err());
    }
}
