/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod closing;
pub(crate) mod content;
pub(crate) mod desktop;
mod http;
mod instance;
pub(crate) mod local;
mod logging;
pub(crate) mod media;
pub(crate) mod organization;
pub(crate) mod plugins;
pub(crate) mod preferences;
pub(crate) mod recording;
pub(crate) mod settings;
pub(crate) mod updates;

#[cfg(windows)]
mod window_icon;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::{Arc, Mutex};

// Third Party Imports
use tauri::Manager;

// Local Imports
use crate::bindings;

/* === Constants === */

const DESKTOP_RETAINED_LOG_FILES: u64 = 14;

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
    // WebKitGTK's DMA-BUF renderer repaints the whole window on the GTK main
    // thread for every animation frame, and that thread also delivers async
    // command responses, so a spinner can delay them by tens of seconds. An
    // explicit operator value still wins.
    #[cfg(target_os = "linux")]
    if std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none() {
        // SAFETY: the process is still single-threaded; no runtime, logger or
        // GTK thread has started yet.
        unsafe { std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1") };
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
    // Resolve Identity Before Logging, Organization Or Windows.
    let mut context = tauri::generate_context!();
    let paths = dokime::paths::ApplicationPaths::resolve(&context.config().identifier)?;
    context.config_mut().app.app_directories_override = Some(tauri::utils::config::AppDirectoriesOverride::Root(paths.root.clone()));
    let _log_guards = logging::init(&paths.logs(), DESKTOP_RETAINED_LOG_FILES as usize);
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    let launch = crate::native_session::launch::configuration(arguments.clone());
    let instance = if launch.target.is_none() && launch.error.is_none() {
        // Single-Instance Identity Remains Compiled For This Release.
        let instance_paths = dokime::paths::ApplicationPaths::resolve_identifier(&context.config().identifier, None)?;
        match tauri::async_runtime::block_on(instance::Instance::acquire(&instance_paths.root))? {
            Some(instance) => Some(instance),
            None => return Ok(()),
        }
    } else {
        None
    };
    let shutdown_stdin = launch.error.is_none() && arguments.iter().any(|argument| argument == "--shutdown-stdin");
    let retained = Arc::new(Mutex::new(None::<crate::native_session::DesktopSession>));
    let setup_owner = Arc::clone(&retained);
    let controls = bindings::invoke_handler();
    let rpc = crate::rpc::handler();
    let builder = tauri::Builder::default()
        .manage(crate::rpc::Bridge::new())
        .manage(crate::connection::ConnectionSession::default())
        .manage(plugins::DroppedFiles::default())
        .manage(crate::connection::registry::Registry::default())
        .manage(launch)
        .manage(paths)
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::DragDrop(tauri::DragDropEvent::Drop { paths, .. }) = event {
                window.state::<plugins::DroppedFiles>().record(window.label(), paths);
            }
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
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init());
    let builder = media::register(builder);
    #[cfg(feature = "updater")]
    let builder = builder.plugin(tauri_plugin_updater::Builder::new().build());
    let app = builder
        .setup(move |app| {
            settings::setup(app);
            organization::setup(app)?;
            let connection = app.state::<crate::connection::ConnectionSession>().inner().clone();
            let owner = crate::native_session::DesktopSession::with_local_configuration(connection, local::configuration(app.handle()));
            let mut status_updates = owner.subscribe_status();
            let status_app = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                use tauri::Emitter;
                while status_updates.changed().await.is_ok() {
                    let snapshot = status_updates.borrow_and_update().clone();
                    let _ = status_app.emit(crate::native_session::STATUS_CHANGED_EVENT, snapshot);
                }
            });
            *setup_owner.lock().expect("setup owner poisoned") = Some(owner.clone());
            app.manage(crate::native_session::DesktopSessionControl::active(owner));
            updates::setup(app)?;
            create_main_windows(app)?;
            if let Some(instance) = instance {
                app.manage(instance.start(app.handle().clone()));
            }
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
    let closing = closing::Closing::default();
    app.run(move |app, event| match event {
        tauri::RunEvent::WindowEvent {
            event: tauri::WindowEvent::CloseRequested { api, .. },
            ..
        } if !closing.complete() => {
            api.prevent_close();
            closing.request(app, 0);
        }
        tauri::RunEvent::ExitRequested { code, api, .. } if !closing.complete() => {
            api.prevent_exit();
            closing.request(app, code.unwrap_or(0));
        }
        _ => {}
    });
    Ok(())
}

fn create_main_windows(app: &mut tauri::App) -> tauri::Result<()> {
    for window in app.config().app.windows.iter().filter(|window| !window.create) {
        let navigation_config = app.config().clone();
        let brand = app.state::<organization::OrganizationState>().initialization_script();
        let builder = tauri::WebviewWindowBuilder::from_config(app.handle(), window)?
            .data_directory(app.state::<dokime::paths::ApplicationPaths>().webview())
            .initialization_script(&brand)
            .on_navigation(move |url| crate::native_session::trusted_application_url(url, &navigation_config));
        let organization = app.state::<organization::OrganizationState>();
        let builder = match organization.title() {
            Some(title) => builder.title(title),
            None => builder,
        };
        let builder = match organization.window_icon() {
            Some(icon) => builder.icon(icon)?,
            None => builder,
        };
        // The window-level OLE handler would otherwise swallow file drops
        // before WebView2 raises the HTML5 events the plugin drop zone uses.
        #[cfg(windows)]
        let builder = builder.drag_and_drop(false);
        // Elevated WebView2 Hosts Ignore Environment Overrides. Forward The
        // Existing WebDriver Opt-In Through The Native Environment Options.
        #[cfg(windows)]
        let builder = if std::env::var("TAURI_WEBVIEW_AUTOMATION").as_deref() == Ok("true") {
            match std::env::var("WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS") {
                Ok(arguments) if !arguments.is_empty() => builder.additional_browser_args(&format!(
                    "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection --autoplay-policy=no-user-gesture-required {arguments}"
                )),
                _ => builder,
            }
        } else {
            builder
        };
        #[cfg(windows)]
        {
            let window = builder.build()?;
            let hwnd = window.hwnd()?;
            window_icon::taskbar(hwnd)?;
        }
        #[cfg(not(windows))]
        builder.build()?;
    }
    Ok(())
}

/* === Tests === */
