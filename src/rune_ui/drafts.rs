//! Text fields whose typing is saved once it pauses: Output's modes, the
//! vocabulary, and Labs › Send to endpoint's fields. Each editor projects
//! its text into the settings; a draft is what differs from the saved ones.

use std::time::{Duration, Instant};

use super::Ui;
use crate::app_settings::AppSettings;

/// How long typing pauses before a text field's edit is saved.
pub(super) const DRAFT_DELAY: Duration = Duration::from_millis(250);

/// Which of the window's text editors hold unsaved typing.
#[derive(Clone, Copy, Debug)]
pub(super) struct Drafts {
    pub(super) modes: bool,
    pub(super) vocabulary: bool,
    /// Labs › Send to endpoint's fields.
    pub(super) endpoint: bool,
}

impl Ui {
    /// The settings with every text field's current text applied.
    pub(super) fn drafted(&self) -> AppSettings {
        let mut candidate = self.settings.clone();
        self.modes.project(&mut candidate);
        candidate.vocabulary = self.vocabulary.project();
        self.lab_store.endpoint.project(&mut candidate);
        candidate
    }

    /// Which editors hold typing the saved settings don't have yet.
    pub(super) fn drafts(&self) -> Drafts {
        let settings = &self.settings;
        let mut modes = settings.clone();
        self.modes.project(&mut modes);
        let mut endpoint = settings.clone();
        self.lab_store.endpoint.project(&mut endpoint);
        Drafts {
            modes: modes.dictation_processing != settings.dictation_processing,
            vocabulary: self.vocabulary.project() != settings.vocabulary,
            endpoint: endpoint.endpoints != settings.endpoints,
        }
    }

    /// Saves text-field edits once typing pauses for [`DRAFT_DELAY`].
    pub(super) fn save_drafts(&mut self, now: Instant) -> bool {
        let candidate = self.drafted();
        if candidate == self.settings {
            self.draft_since = None;
            return false;
        }
        let since = *self.draft_since.get_or_insert(now);
        if now.duration_since(since) < DRAFT_DELAY {
            return false;
        }
        self.commit(candidate)
    }

    /// Saves pending text-field edits now (leaving a pane or a mode).
    pub(super) fn flush_drafts(&mut self) {
        let candidate = self.drafted();
        if candidate != self.settings {
            self.commit(candidate);
        }
    }
}
