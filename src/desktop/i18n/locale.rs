//! The system's language and locale, for the interface language and the
//! speech model: the Windows user locale, macOS's preferred languages, or
//! the POSIX locale variables (`crate::desktop::i18n`).

pub(crate) fn system_language() -> String {
    #[cfg(target_os = "windows")]
    {
        windows_user_language()
    }
    #[cfg(not(target_os = "windows"))]
    {
        language_from_env(
            std::env::var("LC_ALL").ok().as_deref(),
            std::env::var("LC_MESSAGES").ok().as_deref(),
            std::env::var("LANG").ok().as_deref(),
        )
    }
}

/// The system locale with its script and region, for the interface
/// language: the Windows user locale or the POSIX locale variables.
pub(super) fn system_locale() -> String {
    #[cfg(target_os = "windows")]
    {
        windows_user_locale()
    }
    #[cfg(not(target_os = "windows"))]
    {
        locale_from_env(
            std::env::var("LC_ALL").ok().as_deref(),
            std::env::var("LC_MESSAGES").ok().as_deref(),
            std::env::var("LANG").ok().as_deref(),
        )
    }
}

/// The person's first preferred locale in full (`zh-Hant-TW`), for the
/// interface language; on macOS the system's language list.
pub(super) fn preferred_locale() -> String {
    #[cfg(target_os = "macos")]
    if let Some(locale) = macos_preferred_locale() {
        return locale;
    }
    system_locale()
}

/// The person's first preferred language, for choosing a speech model. On
/// macOS this is the system's language list, which apps opened from Finder
/// see without `LANG`.
pub(crate) fn preferred_language() -> String {
    #[cfg(target_os = "macos")]
    if let Some(language) = macos_preferred_language() {
        return language;
    }
    system_language()
}

#[cfg(target_os = "macos")]
fn macos_preferred_language() -> Option<String> {
    primary_subtag(&macos_preferred_locale()?)
}

/// The first entry of `NSLocale.preferredLanguages`, such as `zh-Hant-TW`.
#[cfg(target_os = "macos")]
fn macos_preferred_locale() -> Option<String> {
    use objc2::msg_send;
    use objc2::rc::Retained;
    use objc2::runtime::AnyClass;
    use objc2_foundation::{NSArray, NSString};

    let class = AnyClass::get(c"NSLocale")?;
    let languages: Option<Retained<NSArray<NSString>>> =
        unsafe { msg_send![class, preferredLanguages] };
    Some(languages?.firstObject()?.to_string())
}

/// `pt-BR` and `zh-Hans-CN` yield `pt` and `zh`.
pub(super) fn primary_subtag(tag: &str) -> Option<String> {
    let subtag = tag.split(['-', '_']).next()?.trim().to_ascii_lowercase();
    (!subtag.is_empty()).then_some(subtag)
}

/// The primary subtag of the Windows user locale: `pl-PL` yields `pl`.
#[cfg(target_os = "windows")]
fn windows_user_language() -> String {
    windows_user_locale()
        .split(['-', '_'])
        .next()
        .unwrap_or("en")
        .to_ascii_lowercase()
}

/// The Windows user locale name in full, such as `zh-TW`; `en` if unread.
#[cfg(target_os = "windows")]
fn windows_user_locale() -> String {
    use windows_sys::Win32::Globalization::GetUserDefaultLocaleName;
    const LOCALE_NAME_CAPACITY: usize = 85;
    let mut buffer = [0_u16; LOCALE_NAME_CAPACITY];
    let length = unsafe { GetUserDefaultLocaleName(buffer.as_mut_ptr(), buffer.len() as i32) };
    if length <= 1 {
        return "en".into();
    }
    String::from_utf16_lossy(&buffer[..length as usize - 1])
}

/// The primary language subtag from POSIX locale variables, in their
/// standard precedence: `pl_PL.UTF-8` yields `pl`.
#[cfg_attr(target_os = "windows", allow(dead_code))]
pub(super) fn language_from_env(
    lc_all: Option<&str>,
    lc_messages: Option<&str>,
    lang: Option<&str>,
) -> String {
    [lc_all, lc_messages, lang]
        .into_iter()
        .flatten()
        .map(str::trim)
        .find(|value| !value.is_empty() && *value != "C" && *value != "POSIX")
        .map(|value| {
            value
                .chars()
                .take_while(|character| character.is_ascii_alphabetic())
                .collect::<String>()
                .to_lowercase()
        })
        .unwrap_or_default()
}

/// The first set POSIX locale variable in full, in the same precedence as
/// [`language_from_env`]: `zh_TW.UTF-8` stays `zh_TW.UTF-8`.
#[cfg_attr(target_os = "windows", allow(dead_code))]
pub(super) fn locale_from_env(
    lc_all: Option<&str>,
    lc_messages: Option<&str>,
    lang: Option<&str>,
) -> String {
    [lc_all, lc_messages, lang]
        .into_iter()
        .flatten()
        .map(str::trim)
        .find(|value| !value.is_empty() && *value != "C" && *value != "POSIX")
        .unwrap_or_default()
        .to_owned()
}
