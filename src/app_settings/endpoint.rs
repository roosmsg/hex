//! Labs › Send to endpoint's endpoints: where a dictation that starts with an
//! endpoint's name goes (`crate::dictation::endpoint`).

use super::*;

/// What an endpoint's body holds until it is changed: one JSON object with
/// the dictation under `text`.
pub const DEFAULT_ENDPOINT_BODY: &str = r#"{"text": "{text}"}"#;

/// How many endpoints Labs › Send to endpoint keeps.
pub const MAX_ENDPOINTS: usize = 10;

/// One endpoint of Labs › Send to endpoint (`labs.send_to_endpoint`,
/// Windows). It receives nothing until `url` is set.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct EndpointSettings {
    /// Said first, it sends the rest of the dictation here: "Restage, open
    /// research". Without one, the endpoint receives nothing.
    pub name: String,
    /// The http(s) URL that takes the POST.
    pub url: Option<String>,
    /// Sent as `Authorization: Bearer …` while set.
    pub token: Option<String>,
    /// The JSON body; `{text}` stands for the dictation, escaped for a
    /// JSON string (the quotes around it are the template's).
    pub body: String,
    /// Also paste what was sent, as usual; off, the endpoint takes the
    /// paste's place.
    pub paste: bool,
}

impl Default for EndpointSettings {
    fn default() -> Self {
        Self {
            name: String::new(),
            url: None,
            token: None,
            body: DEFAULT_ENDPOINT_BODY.into(),
            paste: false,
        }
    }
}

impl AppSettings {
    /// A settings file from the single-endpoint version keeps its endpoint:
    /// it becomes the first, without a name, so it receives once it has one.
    pub(super) fn adopt_legacy_endpoint(&mut self) {
        let Some(legacy) = self.legacy_endpoint.take() else {
            return;
        };
        if self.endpoints.is_empty()
            && legacy
                .url
                .as_deref()
                .is_some_and(|url| !url.trim().is_empty())
        {
            self.endpoints.push(legacy);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_single_endpoint_of_the_first_version_becomes_the_first() {
        let json = serde_json::json!({
            "endpoint": {"url": "http://127.0.0.1:1234/command", "token": "t", "body": DEFAULT_ENDPOINT_BODY, "paste": false}
        });
        let mut settings: AppSettings = serde_json::from_value(json).unwrap();
        settings.adopt_legacy_endpoint();
        assert_eq!(settings.endpoints.len(), 1);
        assert_eq!(settings.endpoints[0].name, "", "named by the user");
        assert_eq!(settings.endpoints[0].token.as_deref(), Some("t"));
        let written = serde_json::to_value(&settings).unwrap();
        assert!(
            written.get("endpoint").is_none(),
            "the old key is never written"
        );
        assert_eq!(
            written["endpoints"][0]["url"],
            "http://127.0.0.1:1234/command"
        );
    }

    #[test]
    fn an_empty_first_version_endpoint_adds_nothing() {
        let mut settings: AppSettings =
            serde_json::from_value(serde_json::json!({"endpoint": {"url": null}})).unwrap();
        settings.adopt_legacy_endpoint();
        assert!(settings.endpoints.is_empty());
        assert!(
            serde_json::to_value(&settings)
                .unwrap()
                .get("endpoints")
                .is_none()
        );
    }
}
