//! Labs › Send to endpoint's card body: the URL, the bearer token, the JSON
//! body, and whether the dictation is still pasted. The text fields are
//! drafts saved once typing pauses (`Ui::save_drafts`), like Output's
//! modes and the vocabulary.

use rune::{El, TextEdit, col, row, text};
use rune_kit::{ControlSize, Switch, TextInput};

use super::super::Ui;
use super::super::controls::{DESCRIPTION, Paint};
use crate::app_settings::{AppSettings, DEFAULT_ENDPOINT_BODY, EndpointSettings};
use crate::desktop::i18n::tr;

/// The three text fields' typing, projected into `endpoint.*` on save.
#[derive(Default)]
pub(in crate::rune_ui) struct Editor {
    url: TextEdit,
    token: TextEdit,
    body: TextEdit,
}

impl Editor {
    pub(in crate::rune_ui) fn new(settings: &EndpointSettings) -> Self {
        Self {
            url: TextEdit::new(settings.url.clone().unwrap_or_default()),
            token: TextEdit::new(settings.token.clone().unwrap_or_default()),
            body: TextEdit::new(settings.body.clone()),
        }
    }

    /// Another writer changed the settings: show theirs.
    pub(in crate::rune_ui) fn reload(&mut self, settings: &EndpointSettings) {
        *self = Self::new(settings);
    }

    /// The fields as saved settings: blanks are none, and a blank body is
    /// the default body.
    pub(in crate::rune_ui) fn project(&self, settings: &mut AppSettings) {
        settings.endpoint.url = non_blank(&self.url.text);
        settings.endpoint.token = non_blank(&self.token.text);
        settings.endpoint.body =
            non_blank(&self.body.text).unwrap_or_else(|| DEFAULT_ENDPOINT_BODY.into());
    }
}

fn non_blank(text: &str) -> Option<String> {
    let trimmed = text.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_owned())
}

/// The card's settings while the lab is on.
pub(super) fn body(ui: &Ui, p: Paint) -> El {
    let editor = &ui.lab_store.endpoint;
    col()
        .key("endpoint")
        .gap(8.0)
        .child(field(
            p,
            "endpoint-url",
            tr("Endpoint URL"),
            &editor.url,
            "http://127.0.0.1:1234/command",
            |ui: &mut Ui| &mut ui.lab_store.endpoint.url,
        ))
        .child(field(
            p,
            "endpoint-token",
            tr("Bearer token"),
            &editor.token,
            "",
            |ui: &mut Ui| &mut ui.lab_store.endpoint.token,
        ))
        .child(field(
            p,
            "endpoint-body",
            tr("JSON body"),
            &editor.body,
            DEFAULT_ENDPOINT_BODY,
            |ui: &mut Ui| &mut ui.lab_store.endpoint.body,
        ))
        .child(
            text(tr("{text} stands for the dictation. The endpoint answers while Hex waits, up to 10 seconds."))
                .key("hint")
                .font(DESCRIPTION)
                .color(p.muted())
                .wrap(),
        )
        .child(paste_row(ui, p))
}

fn field(
    p: Paint,
    key: &'static str,
    label: &'static str,
    edit: &TextEdit,
    placeholder: &'static str,
    access: impl Fn(&mut Ui) -> &mut TextEdit + 'static,
) -> El {
    TextInput::new(edit)
        .label(label)
        .placeholder(placeholder)
        .width(None)
        .on_edit(access)
        .build(p.k)
        .key(key)
}

/// Also paste the dictation: off, the endpoint takes the paste's place.
fn paste_row(ui: &Ui, p: Paint) -> El {
    let on = ui.settings.endpoint.paste;
    row()
        .key("endpoint-paste-row")
        .gap(8.0)
        .center_y()
        .child(
            text(tr("Also paste the dictation"))
                .key("label")
                .font(DESCRIPTION)
                .color(p.muted())
                .grow(1.0)
                .shrink(),
        )
        .child(
            Switch::new(on)
                .control_size(ControlSize::Small)
                .label(tr("Also paste the dictation"))
                .hide_label()
                .on_toggle(move |ui: &mut Ui| {
                    ui.update(|settings| settings.endpoint.paste = !on);
                })
                .build(p.k)
                .key("endpoint-paste"),
        )
}
