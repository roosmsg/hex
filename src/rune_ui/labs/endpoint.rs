//! Labs › Send to endpoint's card body: one block per endpoint (name, which
//! a dictation starts with to go there, URL, bearer token, JSON body, and
//! whether what is sent is still pasted), Remove on each, and Add endpoint. The text fields are
//! drafts saved once typing pauses (`Ui::save_drafts`), like Output's modes
//! and the vocabulary.

use rune::{El, TextEdit, Tx, col, edit_access, row, text};
use rune_kit::{ControlSize, Switch, TextInput};

use super::super::Ui;
use super::super::controls::{ButtonKind, DESCRIPTION, Paint, button};
use super::super::lists::remove_button;
use crate::app_settings::{AppSettings, DEFAULT_ENDPOINT_BODY, EndpointSettings, MAX_ENDPOINTS};
use crate::desktop::i18n::tr;

/// The fields of one endpoint, and an identity that stays with it as
/// endpoints before it are removed, so a callback built for a removed one
/// never reaches another.
struct Entry {
    id: u64,
    name: TextEdit,
    url: TextEdit,
    token: TextEdit,
    body: TextEdit,
    paste: bool,
}

#[derive(Clone, Copy)]
enum Field {
    Name,
    Url,
    Token,
    Body,
}

/// The endpoints' typing, projected into `endpoints` on save.
#[derive(Default)]
pub(in crate::rune_ui) struct Editor {
    entries: Vec<Entry>,
    next_id: u64,
    /// What an edit for an endpoint that is gone lands in; never saved.
    detached: TextEdit,
}

impl Editor {
    pub(in crate::rune_ui) fn new(endpoints: &[EndpointSettings]) -> Self {
        let mut editor = Self::default();
        editor.reload(endpoints);
        editor
    }

    /// Another writer changed the settings: show theirs.
    pub(in crate::rune_ui) fn reload(&mut self, endpoints: &[EndpointSettings]) {
        self.entries.clear();
        for endpoint in endpoints {
            self.push(endpoint);
        }
    }

    fn push(&mut self, endpoint: &EndpointSettings) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.entries.push(Entry {
            id,
            name: TextEdit::new(endpoint.name.clone()),
            url: TextEdit::new(endpoint.url.clone().unwrap_or_default()),
            token: TextEdit::new(endpoint.token.clone().unwrap_or_default()),
            body: TextEdit::new(endpoint.body.clone()),
            paste: endpoint.paste,
        });
        id
    }

    /// The fields as saved endpoints: blanks are none, and a blank body is
    /// the default body.
    pub(in crate::rune_ui) fn project(&self, settings: &mut AppSettings) {
        settings.endpoints = self
            .entries
            .iter()
            .map(|entry| EndpointSettings {
                name: entry.name.text.trim().to_owned(),
                url: non_blank(&entry.url.text),
                token: non_blank(&entry.token.text),
                body: non_blank(&entry.body.text).unwrap_or_else(|| DEFAULT_ENDPOINT_BODY.into()),
                paste: entry.paste,
            })
            .collect();
    }

    fn field(&mut self, id: u64, field: Field) -> &mut TextEdit {
        match self.entries.iter_mut().find(|entry| entry.id == id) {
            Some(entry) => match field {
                Field::Name => &mut entry.name,
                Field::Url => &mut entry.url,
                Field::Token => &mut entry.token,
                Field::Body => &mut entry.body,
            },
            None => {
                self.detached = TextEdit::default();
                &mut self.detached
            }
        }
    }
}

fn non_blank(text: &str) -> Option<String> {
    let trimmed = text.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_owned())
}

/// The card's settings while the lab is on.
pub(super) fn body(ui: &Ui, p: Paint) -> El {
    let editor = &ui.lab_store.endpoint;
    let mut body = col().key("endpoints").gap(12.0);
    for entry in &editor.entries {
        body = body.child(entry_block(p, entry));
    }
    if editor.entries.len() < MAX_ENDPOINTS {
        body = body.child(row().key("add-row").child(button(
            p,
            "add-endpoint",
            tr("Add endpoint"),
            ButtonKind::Secondary,
            add_endpoint,
        )));
    }
    body
}

fn entry_block(p: Paint, entry: &Entry) -> El {
    let id = entry.id;
    let paste = entry.paste;
    col()
        .key(format!("endpoint{id}"))
        .gap(6.0)
        .padding(10.0)
        .radius(p.k.radius.md)
        .border(p.hair, p.k.c.border_weak)
        .child(
            row()
                .key("head")
                .gap(8.0)
                .center_y()
                .child(
                    field(
                        p,
                        "name",
                        tr("Name"),
                        &entry.name,
                        "Restage",
                        id,
                        Field::Name,
                    )
                    .grow(1.0)
                    .shrink(),
                )
                .child(remove_button(
                    p,
                    ControlSize::Regular,
                    move |ui: &mut Ui| remove_endpoint(ui, id),
                )),
        )
        .child(field(
            p,
            "url",
            tr("Endpoint URL"),
            &entry.url,
            "http://127.0.0.1:1234/command",
            id,
            Field::Url,
        ))
        .child(field(
            p,
            "token",
            tr("Bearer token"),
            &entry.token,
            "",
            id,
            Field::Token,
        ))
        .child(field(
            p,
            "body",
            tr("JSON body"),
            &entry.body,
            DEFAULT_ENDPOINT_BODY,
            id,
            Field::Body,
        ))
        .child(
            row()
                .key("paste-row")
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
                    Switch::new(paste)
                        .control_size(ControlSize::Small)
                        .label(tr("Also paste the dictation"))
                        .hide_label()
                        .on_toggle(move |ui: &mut Ui| set_paste(ui, id, !paste))
                        .build(p.k)
                        .key("paste"),
                ),
        )
}

fn field(
    p: Paint,
    key: &'static str,
    label: &'static str,
    edit: &TextEdit,
    placeholder: &'static str,
    id: u64,
    which: Field,
) -> El {
    TextInput::new(edit)
        .label(label)
        .placeholder(placeholder)
        .width(None)
        .on_edit(edit_access(move |ui: &mut Ui| {
            ui.lab_store.endpoint.field(id, which)
        }))
        .build(p.k)
        .key(key)
}

/// Add endpoint: a new one at the end, its name field focused.
fn add_endpoint(ui: &mut Ui, _tx: &mut Tx) {
    let editor = &mut ui.lab_store.endpoint;
    for entry in &mut editor.entries {
        for edit in [
            &mut entry.name,
            &mut entry.url,
            &mut entry.token,
            &mut entry.body,
        ] {
            edit.focused = false;
        }
    }
    let id = editor.push(&EndpointSettings::default());
    editor.field(id, Field::Name).focused = true;
    ui.flush_drafts();
}

/// Removes endpoint `id`; a second press for one already gone does nothing.
fn remove_endpoint(ui: &mut Ui, id: u64) {
    let editor = &mut ui.lab_store.endpoint;
    if let Some(index) = editor.entries.iter().position(|entry| entry.id == id) {
        editor.entries.remove(index);
        ui.flush_drafts();
    }
}

fn set_paste(ui: &mut Ui, id: u64, on: bool) {
    if let Some(entry) = ui
        .lab_store
        .endpoint
        .entries
        .iter_mut()
        .find(|entry| entry.id == id)
    {
        entry.paste = on;
        ui.flush_drafts();
    }
}
