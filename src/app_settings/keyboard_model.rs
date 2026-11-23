//! Switch model with keyboard language: which model and language each
//! input source selects. Moved here verbatim by this fork (FORK.md), so
//! app_settings.rs stays under the source cap with the fork's own fields.

use color_eyre::eyre::{Result, eyre};
use serde::{Deserialize, Serialize};

use crate::transcription::models::{TranscriptionModelId, TranscriptionSelection};

/// macOS input-source IDs mapped to the model and language they select. An
/// absent entry means "Don't switch"; this never changes the model merely
/// because dictation starts.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct KeyboardModelSwitching {
    pub enabled: bool,
    pub mappings: std::collections::BTreeMap<String, KeyboardModelChoice>,
}

/// What one input source selects: a model and a language it supports.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct KeyboardModelChoice {
    pub model: TranscriptionModelId,
    pub language: String,
}

impl KeyboardModelChoice {
    /// The picker's downloadable models only; Apple Speech is never mapped.
    pub fn validate(&self) -> Result<()> {
        let selection = TranscriptionSelection {
            model: self.model,
            language: self.language.clone(),
        };
        // An import stays mapped while it is missing or off, like the
        // saved selection; switching to it then runs the default model.
        if self.model.is_labs_gated() {
            return crate::transcription::models::validate_saved(&selection);
        }
        let model = crate::transcription::models::validate(&selection)?;
        match model.runtime {
            crate::transcription::models::ModelRuntime::Gguf(_) => Ok(()),
            crate::transcription::models::ModelRuntime::AppleSpeech => {
                Err(eyre!("{} can't be switched to by input source", model.name))
            }
        }
    }
}
