/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// One Desktop And Focus Channel Per Application Root

#[cfg(unix)]
use std::path::PathBuf;
use std::{
    fs::{File, OpenOptions, TryLockError},
    io,
    path::Path,
    time::Duration,
};

use interprocess::local_socket::{
    ListenerOptions, Name,
    tokio::{Listener, Stream, prelude::*},
};
use sha2::{Digest, Sha256};
use tauri::Manager;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const FOCUS_BUDGET: Duration = Duration::from_secs(10);

pub(crate) struct Instance {
    listener: Listener,
    _lease: File,
}

pub(crate) struct InstanceControl {
    task: tauri::async_runtime::JoinHandle<()>,
    _lease: File,
}

impl Instance {
    pub(crate) async fn acquire(root: &Path) -> io::Result<Option<Self>> {
        let deadline = tokio::time::Instant::now() + FOCUS_BUDGET;
        dokime::harmos::filesystem::publication::create_dir_all(root)?;
        let root = root.canonicalize()?;
        let name = socket_name(&root)?;

        // Serialize Only Socket Publication; Never Wait For Backend Startup.
        let startup = OpenOptions::new().create(true).truncate(false).read(true).write(true).open(root.join("desktop-startup.lock"))?;
        let publication = async {
            loop {
                match startup.try_lock() {
                    Ok(()) => return Ok::<_, io::Error>(()),
                    Err(TryLockError::WouldBlock) => tokio::time::sleep(Duration::from_millis(25)).await,
                    Err(TryLockError::Error(error)) => return Err(error),
                }
            }
        };
        tokio::time::timeout_at(deadline, publication)
            .await
            .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "desktop socket publication is busy"))??;
        let lease = OpenOptions::new().create(true).truncate(false).read(true).write(true).open(root.join("desktop.lock"))?;
        match lease.try_lock() {
            Ok(()) => {}
            Err(TryLockError::WouldBlock) => {
                drop(startup);
                let focused = async {
                    let mut stream = Stream::connect(name).await?;
                    #[cfg(windows)]
                    allow_focus(&stream)?;
                    stream.write_all(b"F").await?;
                    let mut response = [0];
                    stream.read_exact(&mut response).await?;
                    if response != *b"F" {
                        return Err(io::Error::new(io::ErrorKind::InvalidData, "desktop focus was not acknowledged"));
                    }
                    Ok(())
                };
                tokio::time::timeout_at(deadline, focused)
                    .await
                    .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "the existing desktop did not acknowledge focus"))??;
                return Ok(None);
            }
            Err(TryLockError::Error(error)) => return Err(error),
        }

        // Only The Exclusive Owner Can Reclaim A Crashed Unix Socket.
        #[cfg(unix)]
        {
            use std::os::unix::fs::FileTypeExt;
            let path = socket_path(&root);
            match std::fs::symlink_metadata(&path) {
                Ok(metadata) if metadata.file_type().is_socket() => std::fs::remove_file(path)?,
                Ok(_) => return Err(io::Error::new(io::ErrorKind::AlreadyExists, "desktop socket path is occupied")),
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => return Err(error),
            }
        }
        let options = ListenerOptions::new().name(name);
        #[cfg(unix)]
        let options = {
            use interprocess::os::unix::local_socket::ListenerOptionsExt;
            options.mode(0o600)
        };
        #[cfg(windows)]
        let options = {
            use interprocess::os::windows::{local_socket::ListenerOptionsExt, security_descriptor::SecurityDescriptor};
            let sid = current_user_sid()?;
            let descriptor = widestring::U16CString::from_str(format!("D:P(A;;GA;;;{sid})")).map_err(io::Error::other)?;
            let permissions = SecurityDescriptor::deserialize(&descriptor)?;
            options.security_descriptor(permissions)
        };
        let listener = options.create_tokio()?;
        drop(startup);
        Ok(Some(Self { listener, _lease: lease }))
    }

    pub(crate) fn start(self, app: tauri::AppHandle) -> InstanceControl {
        let Self { listener, _lease: lease } = self;
        let task = tauri::async_runtime::spawn(async move {
            loop {
                let mut stream = match listener.accept().await {
                    Ok(stream) => stream,
                    Err(error) => {
                        tracing::error!(%error, "desktop focus listener failed");
                        return;
                    }
                };
                let request = async {
                    let mut request = [0];
                    stream.read_exact(&mut request).await?;
                    if request != *b"F" {
                        return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid desktop focus request"));
                    }
                    let (sent, received) = tokio::sync::oneshot::channel();
                    let owner = app.clone();
                    app.run_on_main_thread(move || {
                        let outcome = owner
                            .get_webview_window("main")
                            .ok_or_else(|| "desktop window is not available".to_owned())
                            .and_then(|window| {
                                window
                                    .unminimize()
                                    .and_then(|_| window.show())
                                    .and_then(|_| window.set_focus())
                                    .map_err(|error| error.to_string())
                            });
                        let _ = sent.send(outcome);
                    })
                    .map_err(io::Error::other)?;
                    received.await.map_err(io::Error::other)?.map_err(io::Error::other)?;
                    stream.write_all(b"F").await?;
                    Ok::<(), io::Error>(())
                };
                match tokio::time::timeout(FOCUS_BUDGET, request).await {
                    Err(error) => tracing::warn!(%error, "desktop focus request expired"),
                    Ok(Err(error)) => tracing::warn!(%error, "desktop focus request failed"),
                    Ok(Ok(())) => {}
                }
            }
        });
        InstanceControl { task, _lease: lease }
    }
}

impl Drop for InstanceControl {
    fn drop(&mut self) {
        self.task.abort();
    }
}

#[cfg(windows)]
fn allow_focus(stream: &Stream) -> io::Result<()> {
    use interprocess::local_socket::traits::StreamCommon;
    use windows::Win32::UI::WindowsAndMessaging::AllowSetForegroundWindow;
    let peer = stream.peer_creds()?;
    let process = peer.pid().ok_or_else(|| io::Error::new(io::ErrorKind::PermissionDenied, "desktop peer has no process identity"))?;
    // The New Foreground Process Grants Its Existing Desktop Permission To Focus.
    let granted = unsafe { AllowSetForegroundWindow(process) };
    let Err(error) = granted else { return Ok(()) };
    // Background Launches May Lack A Foreground Grant; The Existing Window
    // Still Owns Activation And Must Receive The Focus Request.
    if error.code() != windows::Win32::Foundation::E_ACCESSDENIED {
        return Err(io::Error::other(error));
    }
    tracing::debug!(%error, "desktop launch has no foreground permission to transfer");
    Ok(())
}

#[cfg(windows)]
fn current_user_sid() -> io::Result<String> {
    use windows::Win32::{
        Foundation::{CloseHandle, HANDLE, HLOCAL, LocalFree},
        Security::{Authorization::ConvertSidToStringSidW, GetTokenInformation, TOKEN_QUERY, TOKEN_USER, TokenUser},
        System::Threading::{GetCurrentProcess, OpenProcessToken},
    };
    use windows::core::PWSTR;
    let mut token = HANDLE::default();
    // Token Information Owns The SID Until The Conversion Has Copied It.
    unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) }.map_err(io::Error::other)?;
    let result = (|| {
        let mut length = 0;
        let queried = unsafe { GetTokenInformation(token, TokenUser, None, 0, &mut length) };
        if length == 0 || length > 64 * 1024 {
            return Err(queried
                .err()
                .map(io::Error::other)
                .unwrap_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid user token length")));
        }
        let mut buffer = vec![0_u64; (length as usize).div_ceil(std::mem::size_of::<u64>())];
        unsafe { GetTokenInformation(token, TokenUser, Some(buffer.as_mut_ptr().cast()), length, &mut length) }
            .map_err(io::Error::other)?;
        // The Aligned Buffer Was Initialized By GetTokenInformation Above.
        let user = unsafe { &*buffer.as_ptr().cast::<TOKEN_USER>() };
        let mut sid = PWSTR::null();
        unsafe { ConvertSidToStringSidW(user.User.Sid, &mut sid) }.map_err(io::Error::other)?;
        let text = unsafe { sid.to_string() }.map_err(io::Error::other);
        // ConvertSidToStringSidW Allocates On The Local Heap.
        unsafe { LocalFree(Some(HLOCAL(sid.0.cast()))) };
        text
    })();
    let closed = unsafe { CloseHandle(token) }.map_err(io::Error::other);
    let sid = result?;
    closed?;
    Ok(sid)
}

fn socket_name(root: &Path) -> io::Result<Name<'static>> {
    #[cfg(windows)]
    {
        use interprocess::local_socket::{GenericNamespaced, ToNsName};
        let digest = Sha256::digest(root.as_os_str().as_encoded_bytes());
        let digest = digest.iter().map(|byte| format!("{byte:02x}")).collect::<String>();
        format!("dokime-desktop-{digest}").to_ns_name::<GenericNamespaced>()
    }
    #[cfg(unix)]
    {
        use interprocess::local_socket::{GenericFilePath, ToFsName};
        socket_path(root).to_fs_name::<GenericFilePath>()
    }
}

#[cfg(unix)]
fn socket_path(root: &Path) -> PathBuf {
    let digest = Sha256::digest(root.as_os_str().as_encoded_bytes());
    let digest = digest.iter().map(|byte| format!("{byte:02x}")).collect::<String>();
    std::env::temp_dir().join(format!("dokime-desktop-{digest}.sock"))
}
