//! The Windows app mode (Settings › Personalisation › Colours › Choose your
//! default app mode), read from the registry and watched, so the window
//! follows a switch while Hex runs.
//!
//! rune follows the system through winit's `ThemeChanged`, which winit raises
//! when uxtheme's `ShouldAppsUseDarkMode` changes its answer. But rune also
//! sets the process's preferred app mode for its native menus
//! (`SetPreferredAppMode`, forced dark or light by what the window draws),
//! and from then on `ShouldAppsUseDarkMode` answers with that forced mode, so
//! a switch in Windows never reached the window. The registry value is the
//! user's actual choice.

use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

use windows_sys::Win32::System::Registry::{
    HKEY, REG_NOTIFY_CHANGE_LAST_SET, RegNotifyChangeKeyValue,
};
use winreg::RegKey;
use winreg::enums::{HKEY_CURRENT_USER, KEY_NOTIFY, KEY_READ};

const PERSONALIZE: &str = r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize";

const UNKNOWN: u8 = 0;
const DARK: u8 = 1;
const LIGHT: u8 = 2;

static MODE: AtomicU8 = AtomicU8::new(UNKNOWN);
static SWITCHED: AtomicBool = AtomicBool::new(false);

/// Whether apps should be dark, as Windows says now; `None` before
/// [`watch`] or where the value can't be read (then rune's answer stands).
pub(crate) fn dark() -> Option<bool> {
    match MODE.load(Ordering::Relaxed) {
        DARK => Some(true),
        LIGHT => Some(false),
        _ => None,
    }
}

/// True once after each switch [`watch`] saw, for the window's poll to
/// redraw with the other palette.
pub(crate) fn take_switched() -> bool {
    SWITCHED.swap(false, Ordering::Relaxed)
}

fn read() -> u8 {
    RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey(PERSONALIZE)
        .and_then(|key| key.get_value::<u32, _>("AppsUseLightTheme"))
        .map_or(UNKNOWN, |light| if light == 0 { DARK } else { LIGHT })
}

/// Reads the app mode now, then watches it on a thread of its own and calls
/// `switched` after each change, so the window can wake and redraw.
pub(crate) fn watch(switched: impl Fn() + Send + 'static) {
    MODE.store(read(), Ordering::Relaxed);
    let spawned = std::thread::Builder::new()
        .name("hex-app-mode".into())
        .spawn(move || {
            let Ok(key) = RegKey::predef(HKEY_CURRENT_USER)
                .open_subkey_with_flags(PERSONALIZE, KEY_READ | KEY_NOTIFY)
            else {
                return;
            };
            loop {
                // SAFETY: `key` is an open registry key with KEY_NOTIFY that
                // outlives the call; synchronous mode, so no event handle.
                let status = unsafe {
                    RegNotifyChangeKeyValue(
                        key.raw_handle() as usize as HKEY,
                        0,
                        REG_NOTIFY_CHANGE_LAST_SET,
                        std::ptr::null_mut(),
                        0,
                    )
                };
                if status != 0 {
                    tracing::warn!(status, "stopped watching the Windows app mode");
                    return;
                }
                let now = read();
                if MODE.swap(now, Ordering::Relaxed) != now {
                    SWITCHED.store(true, Ordering::Relaxed);
                    switched();
                }
            }
        });
    if let Err(error) = spawned {
        tracing::warn!(%error, "could not watch the Windows app mode");
    }
}
