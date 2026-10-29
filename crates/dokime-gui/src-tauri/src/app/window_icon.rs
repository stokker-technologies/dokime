/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::io;

use windows::Win32::{
    Foundation::{HWND, LPARAM, WPARAM},
    UI::WindowsAndMessaging::{ICON_BIG, ICON_SMALL, SendMessageW, WM_GETICON, WM_SETICON},
};

pub(super) fn taskbar(window: HWND) -> io::Result<()> {
    // Tao Retains The Full-Resolution HICON In Its Window State. Both Window
    // Surfaces Borrow That Handle Until Window Destruction; Neither Owns It.
    let small = WPARAM(ICON_SMALL as usize);
    let big = WPARAM(ICON_BIG as usize);
    let icon = unsafe { SendMessageW(window, WM_GETICON, Some(small), None) };
    if icon.0 == 0 {
        return Err(io::Error::other("the organization window icon is unavailable for the Windows taskbar"));
    }
    let handle = LPARAM(icon.0);
    unsafe { SendMessageW(window, WM_SETICON, Some(big), Some(handle)) };

    let applied = unsafe { SendMessageW(window, WM_GETICON, Some(big), None) };
    if applied != icon {
        return Err(io::Error::other("the Windows taskbar icon could not be applied"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::{
        Win32::UI::WindowsAndMessaging::{
            CreateWindowExW, DestroyWindow, IDI_APPLICATION, IDI_WARNING, LoadIconW, WINDOW_EX_STYLE, WS_OVERLAPPEDWINDOW,
        },
        core::w,
    };

    struct Window(HWND);

    impl Drop for Window {
        fn drop(&mut self) {
            unsafe { DestroyWindow(self.0) }.unwrap();
        }
    }

    #[test]
    fn taskbar_borrows_the_window_icon_and_reads_it_back() {
        let window = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("STATIC"),
                w!("Icon Fixture"),
                WS_OVERLAPPEDWINDOW,
                0,
                0,
                100,
                100,
                None,
                None,
                None,
                None,
            )
        }
        .unwrap();
        let window = Window(window);
        let small = unsafe { LoadIconW(None, IDI_APPLICATION) }.unwrap();
        let old_big = unsafe { LoadIconW(None, IDI_WARNING) }.unwrap();
        let small_kind = WPARAM(ICON_SMALL as usize);
        let big_kind = WPARAM(ICON_BIG as usize);
        let small_handle = LPARAM(small.0 as isize);
        let old_big_handle = LPARAM(old_big.0 as isize);
        // Stock Icons Are Shared Handles; This Test Does Not Destroy Them.
        unsafe {
            SendMessageW(window.0, WM_SETICON, Some(small_kind), Some(small_handle));
            SendMessageW(window.0, WM_SETICON, Some(big_kind), Some(old_big_handle));
        }

        taskbar(window.0).unwrap();
        let applied = unsafe { SendMessageW(window.0, WM_GETICON, Some(big_kind), None) };
        assert_eq!(applied.0, small.0 as isize);
        let retained = unsafe { SendMessageW(window.0, WM_GETICON, Some(small_kind), None) };
        assert_eq!(retained.0, small.0 as isize);

        let no_icon = LPARAM(0);
        unsafe { SendMessageW(window.0, WM_SETICON, Some(small_kind), Some(no_icon)) };
        assert!(taskbar(window.0).is_err());
    }
}
