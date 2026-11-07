//! Which settings `hex settings set` refuses on this OS, build, or under an
//! organization's policy, and why (`crate::control::settings`).

use crate::control::ControlError;

/// Settings that act only where Hex has a capability (by its
/// `rune_ui::capabilities` name), the same one that shows their row in the
/// window. The first matching prefix decides.
pub(super) const NEEDS: &[(&str, &str)] = &[
    ("meetings.calendar.", "calendar"),
    ("meetings.", "meetings"),
    ("labs.live_chapters", "meetings"),
    ("labs.subtitle_export", "meetings"),
    ("labs.stop_when_call_ends", "meetings"),
    ("labs.more_call_apps", "meetings"),
    ("labs.movable_meeting_capsules", "meetings"),
    ("labs.meeting_notes_vocabulary", "meetings"),
    ("labs.meeting_finished_event", "meetings"),
    ("labs.meeting_descriptive_filenames", "meetings"),
    ("labs.follow_live_transcript", "meetings"),
    ("experimental.hex_microphone", "fun"),
    ("show_dock_icon", "dock_icon"),
    ("labs.hide_menu_bar_icon", "dock_icon"),
    ("labs.quit_closes_window", "dock_icon"),
    ("keyboard_model_switching.", "keyboard_model_switching"),
    ("enable_on_launch", "listener_control"),
    ("share_dictation_shortcut", "share_shortcut"),
];

/// The capability `key` needs, if any.
pub(super) fn needs(key: &str) -> Option<&'static str> {
    NEEDS
        .iter()
        .find(|(prefix, _)| key == *prefix || key.starts_with(prefix))
        .map(|(_, capability)| *capability)
}

/// Why `hex settings set` refuses `name` on this OS, if it does.
pub fn unavailable(name: &str) -> Option<ControlError> {
    unavailable_under(name, crate::policy::Policy::current())
}

pub(super) fn unavailable_under(name: &str, policy: crate::policy::Policy) -> Option<ControlError> {
    if let Some(lock) = policy.setting_lock(name) {
        return Some(managed(name, lock));
    }
    if let Some(feature) = crate::labs::Feature::for_settings_key(name)
        && !feature.shipped()
    {
        return Some(ControlError::new(
            "unsupported",
            format!("`{name}` belongs to a Labs feature that only dev builds have"),
        ));
    }
    // A `labs.` key is only its feature's switch, so off its OS it does nothing.
    if let Some(feature) = crate::labs::Feature::for_settings_key(name)
        && (name.starts_with("labs.") || name == "endpoints" || name == "capture.replay_hotkey")
        && !feature.on_this_os()
    {
        return Some(ControlError::new(
            "unsupported",
            format!("`{name}` belongs to a Labs feature this OS doesn't have"),
        ));
    }
    if name.starts_with("screenshots.") && !cfg!(target_os = "macos") {
        return Some(ControlError::new(
            "unsupported",
            format!("`{name}` belongs to Labs › Screenshots, which only macOS has"),
        ));
    }
    let capability = needs(name).filter(|capability| !crate::rune_ui::capability(capability))?;
    Some(ControlError::new(
        "unsupported",
        format!(
            "`{name}` does nothing on this OS: it needs {capability}, which Hex has only elsewhere"
        ),
    ))
}

pub(super) fn managed(name: &str, lock: crate::policy::Lock) -> ControlError {
    ControlError::new(
        "managed",
        format!("`{name}` can't be changed: {}", lock.refusal()),
    )
}
