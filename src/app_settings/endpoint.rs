//! Labs › Send to endpoint's own settings: where a finished dictation goes
//! (`crate::dictation::endpoint`).

use super::*;

/// What `endpoint.body` holds until it is changed: one JSON object with
/// the dictation under `text`.
pub const DEFAULT_ENDPOINT_BODY: &str = r#"{"text": "{text}"}"#;

/// Where Labs › Send to endpoint (`labs.send_to_endpoint`, Windows) posts
/// each dictation. The lab sends nothing until `url` is set.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct EndpointSettings {
    /// The http(s) URL that takes the POST.
    pub url: Option<String>,
    /// Sent as `Authorization: Bearer …` while set.
    pub token: Option<String>,
    /// The JSON body; `{text}` stands for the dictation, escaped for a
    /// JSON string (the quotes around it are the template's).
    pub body: String,
    /// Also paste the dictation as usual; off, the endpoint takes the
    /// paste's place.
    pub paste: bool,
}

impl Default for EndpointSettings {
    fn default() -> Self {
        Self {
            url: None,
            token: None,
            body: DEFAULT_ENDPOINT_BODY.into(),
            paste: false,
        }
    }
}
