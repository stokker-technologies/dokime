/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution Resource Lifetime Proof

/* === Modules === */

#[path = "../support/resources.rs"]
mod resources;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

// Third Party Imports
use tokio::sync::Notify;

// Local Imports
use dokime::prelude::*;
use resources::ResourceState;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Device(Arc<AtomicU64>);

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Drop for Device {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn main() -> Result<()> {
    let entered = Arc::new(Notify::new());
    let released = Arc::new(Notify::new());
    let dropped = Arc::new(AtomicU64::new(0));
    let initialize_dropped = dropped.clone();
    let observer_entered = entered.clone();
    let observer_released = released.clone();
    let stream = Stream::<ResourceState>::new("resources.state", "Resource State");
    let observer = Observer::new(&stream, async move |ctx: &Context| {
        let before = ctx.get::<Device>().is_ok();
        observer_entered.notify_one();
        observer_released.notified().await;
        Ok(ResourceState {
            before,
            present: ctx.get::<Device>().is_ok(),
            dropped: dropped.load(Ordering::SeqCst),
        })
    })
    .every_ms(1000);
    let template = Template::new("resources.case", "Resource Lifetime")
        .initialize(async move |ctx: &Context| {
            ctx.set(Device(initialize_dropped.clone()));
            Ok(())
        })
        .stream(stream)
        .observer(observer);
    let ready = Operation::new("resources.observer_ready", dokime::Visibility::Protected, async move |_: &Context, _: ()| {
        entered.notified().await;
        Ok(())
    });
    let release = Operation::new("resources.release_observer", dokime::Visibility::Protected, async move |_: &Context, _: ()| {
        released.notify_one();
        Ok(())
    });
    Plugin::new("resources", "1.0.0")
        .suite(Suite::new("resources", "Resources").template(template))
        .operation(ready)
        .operation(release)
        .serve()
}

/* === Tests === */
