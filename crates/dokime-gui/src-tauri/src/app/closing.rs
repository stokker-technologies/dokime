/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Hide Once, Then Stop the Backend in the Background

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

// Third Party Imports
use tauri::{Emitter, Manager};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogResult};

// Local Imports
use crate::PRODUCT;
use crate::native_session::{DesktopSessionControl, DesktopSessionError};
use crate::rpc::Bridge;

/* === Constants === */

// The bounded plugin teardown plus one stop grace for the backend's final
// checkpoint; past it the operator is offered an explicit choice to exit
// without finishing.
const DESKTOP_CLOSING_DEADLINE: Duration = dokime_runtime::PLUGINS_TIMEOUT.checked_add(dokime_runtime::STOP_GRACE).unwrap();

// The event the WebView follows (`src/lib/desktop/stopping.ts`).
const STOPPING_EVENT: &str = "desktop://stopping";
const EXIT_WITHOUT_FINISHING: &str = "Exit Without Finishing";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

// One per process: every close and exit request of every window lands here.
#[derive(Clone, Default)]
pub(crate) struct Closing(Arc<State>);

#[derive(Default)]
struct State {
    started: AtomicBool,
    complete: AtomicBool,
    asking: AtomicBool,
    abandoned: AtomicBool,
    expired: AtomicBool,
    titles: Mutex<Vec<(String, String)>>,
}

#[derive(Clone, Copy, serde::Serialize)]
struct Announcement {
    stopping: bool,
}

/* === Enums === */

enum Failure {
    Retry,
    Exit,
    KeepOpen,
}

/* === Traits === */

/* === Implementations === */

impl Closing {
    pub(crate) fn complete(&self) -> bool {
        self.0.complete.load(Ordering::Acquire)
    }

    pub(crate) fn request(&self, app: &tauri::AppHandle, code: i32) {
        self.0.request(
            || {
                let started = Instant::now();
                tracing::debug!("desktop close requested");
                let mut titles = self.0.titles.lock().expect("closing titles poisoned");
                for (label, window) in app.webview_windows() {
                    titles.push((label, window.title().unwrap_or_else(|_| PRODUCT.to_owned())));
                    if let Err(error) = window.hide() {
                        tracing::error!(%error, "Hide Closing Window: Close the Window After Shutdown Completes");
                    }
                    window.state::<Bridge>().cancel_owner(window.label());
                }
                tracing::debug!(stage = "hide_windows", elapsed_ms = started.elapsed().as_millis(), "desktop shutdown stage completed");
            },
            || {
                let closing = self.clone();
                let app = app.clone();
                tauri::async_runtime::spawn(async move { closing.stop(app, code).await });
            },
        );
    }

    async fn stop(self, app: tauri::AppHandle, code: i32) {
        // Before The Backend Hears Of It: Its Refusals Belong To The Stop From Here On.
        announce(&app, Announcement { stopping: true });
        loop {
            let started = Instant::now();
            let outcome = self.attempt(&app).await;
            tracing::debug!(stage = "desktop_session", elapsed_ms = started.elapsed().as_millis(), "desktop shutdown stage completed");
            if self.0.abandoned.load(Ordering::Acquire) {
                return self.finish(&app, 1);
            }
            let Err(error) = outcome else {
                return self.finish(&app, code);
            };
            tracing::error!(?error, "native desktop shutdown failed");
            match failure_choice(&app, &error).await {
                Failure::Retry => continue,
                Failure::Exit => return self.finish(&app, 1),
                Failure::KeepOpen => {
                    self.restore_titles(&app);
                    announce(&app, Announcement { stopping: false });
                    self.0.started.store(false, Ordering::Release);
                    return;
                }
            }
        }
    }

    async fn attempt(&self, app: &tauri::AppHandle) -> Result<(), DesktopSessionError> {
        self.0.expired.store(false, Ordering::Release);
        let control = app.state::<DesktopSessionControl>();
        let shutdown = control.shutdown();
        let deadline = tokio::time::sleep(DESKTOP_CLOSING_DEADLINE);
        tokio::pin!(shutdown, deadline);
        loop {
            tokio::select! {
                biased;
                result = &mut shutdown => return result,
                () = &mut deadline, if !self.0.expired.load(Ordering::Acquire) => {
                    self.0.expired.store(true, Ordering::Release);
                    self.offer_force_quit(app);
                }
            }
        }
    }

    fn restore_titles(&self, app: &tauri::AppHandle) {
        for (label, title) in std::mem::take(&mut *self.0.titles.lock().expect("closing titles poisoned")) {
            if let Some(window) = app.get_webview_window(&label) {
                let _ = window.set_title(&title);
                let _ = window.show();
            }
        }
    }

    fn offer_force_quit(&self, app: &tauri::AppHandle) {
        if !self.0.force_available() || self.0.asking.swap(true, Ordering::AcqRel) {
            return;
        }
        let closing = self.clone();
        let owner = app.clone();
        app.dialog()
            .message(format!(
                "{PRODUCT} is still stopping the running execution and saving its evidence.\n\nExit without finishing? \
                 The next start seals whatever is left unfinished."
            ))
            .title("Exit Without Finishing?")
            .buttons(MessageDialogButtons::OkCancelCustom(EXIT_WITHOUT_FINISHING.into(), "Keep Waiting".into()))
            .show_with_result(move |answer| {
                closing.0.asking.store(false, Ordering::Release);
                if closing.0.force_available() && exits(&answer) {
                    // The stop task sees the backend end and exits without a failure dialog.
                    closing.0.abandoned.store(true, Ordering::Release);
                    owner.state::<DesktopSessionControl>().abandon();
                }
            });
    }

    fn finish(&self, app: &tauri::AppHandle, code: i32) {
        self.0.complete.store(true, Ordering::Release);
        app.exit(code);
    }
}

impl State {
    fn request(&self, hide: impl FnOnce(), start: impl FnOnce()) {
        if self.complete.load(Ordering::Acquire) || self.started.swap(true, Ordering::AcqRel) {
            return;
        }
        self.expired.store(false, Ordering::Release);
        hide();
        start();
    }

    fn force_available(&self) -> bool {
        self.started.load(Ordering::Acquire) && self.expired.load(Ordering::Acquire) && !self.complete.load(Ordering::Acquire)
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn announce(app: &tauri::AppHandle, announcement: Announcement) {
    if let Err(error) = app.emit(STOPPING_EVENT, announcement) {
        tracing::debug!(%error, "the stop announcement did not reach the WebView");
    }
}

fn exits(answer: &MessageDialogResult) -> bool {
    match answer {
        MessageDialogResult::Ok => true,
        MessageDialogResult::Custom(label) => label == EXIT_WITHOUT_FINISHING,
        _ => false,
    }
}

async fn failure_choice(app: &tauri::AppHandle, error: &DesktopSessionError) -> Failure {
    for window in app.webview_windows().values() {
        let _ = window.set_title(&format!("{PRODUCT} — Stop Failed"));
    }
    let (send, receive) = tokio::sync::oneshot::channel();
    app.dialog()
        .message(format!("{PRODUCT} could not stop cleanly. Evidence may be incomplete.\n\n{error:?}"))
        .title("Stop Failed")
        .buttons(MessageDialogButtons::YesNoCancelCustom("Retry".into(), EXIT_WITHOUT_FINISHING.into(), "Keep Open".into()))
        .show_with_result(move |answer| {
            let _ = send.send(answer);
        });
    match receive.await {
        Ok(MessageDialogResult::Yes) => Failure::Retry,
        Ok(MessageDialogResult::Custom(answer)) if answer == "Retry" => Failure::Retry,
        Ok(MessageDialogResult::No) => Failure::Exit,
        Ok(MessageDialogResult::Custom(answer)) if answer == EXIT_WITHOUT_FINISHING => Failure::Exit,
        _ => Failure::KeepOpen,
    }
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::{Ordering, State};
    use std::cell::RefCell;

    #[test]
    fn an_explicit_keep_open_allows_a_fresh_close_without_a_stale_force_choice() {
        let state = State::default();
        state.request(|| {}, || {});
        state.expired.store(true, Ordering::Release);
        assert!(state.force_available());
        state.started.store(false, Ordering::Release);
        assert!(!state.force_available());
        let actions = RefCell::new(Vec::new());
        state.request(|| actions.borrow_mut().push("hidden"), || actions.borrow_mut().push("shutdown"));
        assert_eq!(*actions.borrow(), ["hidden", "shutdown"]);
        assert!(!state.force_available());
    }

    #[test]
    fn first_close_hides_before_shutdown_and_repeated_requests_do_nothing() {
        let state = State::default();
        let actions = RefCell::new(Vec::new());
        state.request(|| actions.borrow_mut().push("hidden"), || actions.borrow_mut().push("shutdown"));
        assert_eq!(*actions.borrow(), ["hidden", "shutdown"]);
        state.request(|| panic!("already hidden"), || panic!("already stopping"));
        state.expired.store(true, Ordering::Release);
        assert!(state.force_available());
        state.request(|| panic!("already hidden"), || panic!("already stopping"));
        state.complete.store(true, Ordering::Release);
        assert!(!state.force_available());
        state.request(|| panic!("completed"), || panic!("completed"));
    }
}
