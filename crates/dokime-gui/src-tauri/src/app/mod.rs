/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

pub(crate) mod branding;
pub(crate) mod content;
pub(crate) mod desktop;
pub(crate) mod gitlab;
pub(crate) mod local;
mod logging;
pub(crate) mod media;
pub(crate) mod recording;
pub(crate) mod settings;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

// Third Party Imports
use tauri::Manager;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogResult};

// Local Imports
use crate::bindings;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn run() {
    // WebView2 must create its UI thread in a single-threaded COM apartment.
    #[cfg(windows)]
    unsafe {
        use windows::Win32::System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx};
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
    }
    dotenv::dotenv().ok();
    if let Err(error) = start_tauri_app() {
        eprintln!("failed to run {} desktop shell: {error}", crate::PRODUCT);
        std::process::exit(1);
    }
}

pub async fn desktop_restart(app: tauri::AppHandle) -> Result<(), String> {
    app.state::<crate::native_session::DesktopSessionControl>().shutdown().await.map_err(|error| format!("{error:?}"))?;
    app.request_restart();
    Ok(())
}

/* === Private Functions === */

fn start_tauri_app() -> tauri::Result<()> {
    // Bound before the builder: a plugin drop that stalls before the webview
    // reaches a single command still needs its bridge-open/write/finish
    // lines, so the subscriber cannot wait for `setup()`. `generate_context!`
    // already carries the bundle identifier the file layer resolves its
    // directory from; the same `context` value is later handed to `.build`.
    let context = tauri::generate_context!();
    let _log_guards = logging::init(&context.config().identifier);
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    let launch = crate::native_session::launch::configuration(arguments.clone());
    let shutdown_stdin = launch.error.is_none() && arguments.iter().any(|argument| argument == "--shutdown-stdin");
    let retained = Arc::new(Mutex::new(None::<crate::native_session::DesktopSession>));
    let setup_owner = Arc::clone(&retained);
    let controls = bindings::invoke_handler();
    let rpc = crate::rpc::handler();
    let builder = tauri::Builder::default()
        .manage(crate::rpc::Bridge::default())
        .manage(crate::connection::ConnectionSession::default())
        .manage(launch)
        .on_window_event(|window, event| {
            if matches!(event, tauri::WindowEvent::Destroyed) {
                window.state::<crate::rpc::Bridge>().cancel_owner(window.label());
                // No single generation is authoritative at window teardown;
                // drop the whole media preview cache rather than one slice
                // of it.
                let app = window.app_handle().clone();
                tauri::async_runtime::spawn(async move {
                    media::release_all(&app).await;
                });
            }
        })
        // A page that navigates or reloads abandons every call it made; its
        // streams and uploads end with it rather than holding bridge capacity.
        .on_page_load(|webview, payload| {
            if matches!(payload.event(), tauri::webview::PageLoadEvent::Started) {
                webview.state::<crate::rpc::Bridge>().cancel_owner(webview.label());
            }
        })
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init());
    let builder = media::register(builder);
    #[cfg(feature = "updater")]
    let builder = builder.plugin(tauri_plugin_updater::Builder::new().build());
    let app = builder
        .setup(move |app| {
            settings::setup(app);
            gitlab::setup(app)?;
            let connection = app.state::<crate::connection::ConnectionSession>().inner().clone();
            let owner = crate::native_session::DesktopSession::with_local_configuration(connection, local::configuration(app.handle()));
            *setup_owner.lock().expect("setup owner poisoned") = Some(owner.clone());
            app.manage(crate::native_session::DesktopSessionControl::active(owner));
            branding::setup(app)?;
            create_main_windows(app)?;
            if shutdown_stdin {
                let handle = app.handle().clone();
                // A dedicated thread avoids retaining a blocking Tokio task on
                // desktop exit. EOF uses the existing graceful exit owner.
                std::thread::Builder::new().name("desktop-parent".into()).spawn(move || {
                    let _ = std::io::copy(&mut std::io::stdin().lock(), &mut std::io::sink());
                    handle.exit(0);
                })?;
            }
            Ok(())
        })
        .invoke_handler(move |invoke| if crate::rpc::COMMANDS.contains(&invoke.message.command()) { rpc(invoke) } else { controls(invoke) })
        .build(context);
    let app = match app {
        Ok(app) => app,
        Err(error) => {
            let owner = retained.lock().expect("setup owner poisoned").take();
            if let Some(owner) = owner {
                let _ = tauri::async_runtime::block_on(owner.shutdown());
            }
            return Err(error);
        }
    };
    drop(retained);
    let shutdown_started = Arc::new(AtomicBool::new(false));
    let shutdown_complete = Arc::new(AtomicBool::new(false));
    app.run(move |app, event| {
        if let tauri::RunEvent::WindowEvent { event: tauri::WindowEvent::CloseRequested { api, .. }, .. } = &event
            && !shutdown_complete.load(Ordering::Acquire)
        {
            api.prevent_close();
            app.exit(0);
            return;
        }
        if let tauri::RunEvent::ExitRequested { code, api, .. } = event
            && !shutdown_complete.load(Ordering::Acquire)
        {
            api.prevent_exit();
            if !shutdown_started.swap(true, Ordering::AcqRel) {
                let app = app.clone();
                let complete = Arc::clone(&shutdown_complete);
                let started = Arc::clone(&shutdown_started);
                for window in app.webview_windows().values() {
                    let _ = window.set_title(&format!("{} — Finishing Reports…", crate::PRODUCT));
                }
                tauri::async_runtime::spawn(async move {
                    loop {
                        let result = app.state::<crate::native_session::DesktopSessionControl>().shutdown().await;
                        if let Err(error) = result {
                            tracing::error!(?error, "native desktop shutdown failed");
                            for window in app.webview_windows().values() {
                                let _ = window.set_title(&format!("{} — Report Finalization Failed", crate::PRODUCT));
                            }
                            let (send, receive) = tokio::sync::oneshot::channel();
                            app.dialog()
                                .message(format!(
                                    "{} could not finish closing cleanly. Reports may be incomplete.\n\n{error:?}",
                                    crate::PRODUCT
                                ))
                                .title("Report Finalization Failed")
                                .buttons(MessageDialogButtons::YesNoCancelCustom(
                                    "Retry".into(),
                                    "Exit Without Completed Reports".into(),
                                    "Keep Open".into(),
                                ))
                                .show_with_result(move |answer| {
                                    let _ = send.send(answer);
                                });
                            match receive.await {
                                Ok(MessageDialogResult::Yes) => continue,
                                Ok(MessageDialogResult::Custom(answer)) if answer == "Retry" => continue,
                                Ok(MessageDialogResult::No) => {}
                                Ok(MessageDialogResult::Custom(answer)) if answer == "Exit Without Completed Reports" => {}
                                _ => {
                                    started.store(false, Ordering::Release);
                                    return;
                                }
                            }
                            complete.store(true, Ordering::Release);
                            app.exit(1);
                            return;
                        }
                        complete.store(true, Ordering::Release);
                        app.exit(code.unwrap_or(0));
                        return;
                    }
                });
            }
        }
    });
    Ok(())
}

fn create_main_windows(app: &mut tauri::App) -> tauri::Result<()> {
    for window in app.config().app.windows.iter().filter(|window| !window.create) {
        let navigation_config = app.config().clone();
        // The branding pack reaches the frontend before its first script runs.
        let brand = app.state::<branding::BrandingState>().initialization_script();
        let builder = tauri::WebviewWindowBuilder::from_config(app.handle(), window)?
            .initialization_script(&brand)
            .on_navigation(move |url| crate::native_session::trusted_application_url(url, &navigation_config));
        // The window-level OLE handler would otherwise swallow file drops
        // before WebView2 raises the HTML5 events the plugin drop zone uses.
        #[cfg(windows)]
        let builder = builder.drag_and_drop(false);
        builder.build()?;
    }
    Ok(())
}

/* === Tests === */
