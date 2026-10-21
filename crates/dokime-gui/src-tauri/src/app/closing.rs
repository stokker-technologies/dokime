/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// The window close: an orderly backend stop, its visible state and the force-quit choice

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

// Third Party Imports
use tauri::{Emitter, Manager};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogResult};

// Local Imports
use crate::PRODUCT;
use crate::native_session::{DesktopSessionControl, DesktopSessionError};

/* === Constants === */

/// The event the WebView follows (`src/lib/desktop/stopping.ts`).
const STOPPING_EVENT: &str = "desktop://stopping";
/// A stop shorter than this shows nothing: an idle close simply closes.
const VISIBLE_AFTER: Duration = Duration::from_millis(400);
/// The run's settle budget plus the backend's final checkpoint; past it the
/// operator is asked, as a second close or Ctrl-C would.
const STOP_DEADLINE: Duration = Duration::from_secs(dokime_domain::STOP_SETTLEMENT_SECONDS + 30);
const EXIT_WITHOUT_FINISHING: &str = "Exit Without Finishing";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// One per process: every close and exit request of every window lands here.
#[derive(Clone, Default)]
pub(crate) struct Closing(Arc<State>);

#[derive(Default)]
struct State {
    started: AtomicBool,
    complete: AtomicBool,
    asking: AtomicBool,
    abandoned: AtomicBool,
    titles: Mutex<Vec<(String, String)>>,
}

#[derive(Clone, Copy, serde::Serialize)]
struct Announcement {
    stopping: bool,
    visible: bool,
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

    /// The first request starts the orderly stop; one made during it offers to
    /// exit without finishing.
    pub(crate) fn request(&self, app: &tauri::AppHandle, code: i32) {
        if self.0.started.swap(true, Ordering::AcqRel) {
            self.offer_force_quit(app);
            return;
        }
        let closing = self.clone();
        let app = app.clone();
        tauri::async_runtime::spawn(async move { closing.stop(app, code).await });
    }

    async fn stop(self, app: tauri::AppHandle, code: i32) {
        // Before The Backend Hears Of It: Its Refusals Belong To The Stop From Here On.
        announce(&app, Announcement { stopping: true, visible: false });
        loop {
            let outcome = self.attempt(&app).await;
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
                    announce(&app, Announcement { stopping: false, visible: false });
                    self.0.started.store(false, Ordering::Release);
                    return;
                }
            }
        }
    }

    async fn attempt(&self, app: &tauri::AppHandle) -> Result<(), DesktopSessionError> {
        let control = app.state::<DesktopSessionControl>();
        let shutdown = control.shutdown();
        let visible = tokio::time::sleep(VISIBLE_AFTER);
        let deadline = tokio::time::sleep(STOP_DEADLINE);
        tokio::pin!(shutdown, visible, deadline);
        let (mut shown, mut expired) = (false, false);
        loop {
            tokio::select! {
                result = &mut shutdown => return result,
                () = &mut visible, if !shown => {
                    shown = true;
                    self.show_stopping(app);
                }
                () = &mut deadline, if !expired => {
                    expired = true;
                    self.offer_force_quit(app);
                }
            }
        }
    }

    fn show_stopping(&self, app: &tauri::AppHandle) {
        let mut titles = self.0.titles.lock().expect("closing titles poisoned");
        for (label, window) in app.webview_windows() {
            if !titles.iter().any(|(saved, _)| *saved == label) {
                titles.push((label.clone(), window.title().unwrap_or_else(|_| PRODUCT.to_owned())));
            }
            let _ = window.set_title(&format!("{PRODUCT} — Stopping…"));
        }
        announce(app, Announcement { stopping: true, visible: true });
    }

    fn restore_titles(&self, app: &tauri::AppHandle) {
        for (label, title) in std::mem::take(&mut *self.0.titles.lock().expect("closing titles poisoned")) {
            if let Some(window) = app.get_webview_window(&label) {
                let _ = window.set_title(&title);
            }
        }
    }

    fn offer_force_quit(&self, app: &tauri::AppHandle) {
        if self.0.asking.swap(true, Ordering::AcqRel) {
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
                if exits(&answer) {
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
