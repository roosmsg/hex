//! Labs › Send to endpoint: a finished dictation goes to a local HTTP
//! endpoint as JSON, for apps that take spoken commands
//! (`labs.send_to_endpoint`, `endpoint.*`; Windows). Gated where the work
//! starts: [`Endpoint::from_settings`] is `None` while the lab is off or no
//! URL is set, and the output worker then pastes as it always did.
//!
//! Logs name the endpoint's host and the HTTP status, never the URL's path,
//! the token, or the body, which carries dictated text.

use std::time::Duration;

use crate::app_settings::{AppSettings, DEFAULT_ENDPOINT_BODY};

/// What `endpoint.body` replaces with the dictation.
pub const PLACEHOLDER: &str = "{text}";

/// How long one POST may take, connection included.
const TIMEOUT: Duration = Duration::from_secs(10);

/// The lab's settings as the output worker sends with: a snapshot, replaced
/// whenever the settings are saved.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Endpoint {
    pub url: String,
    pub token: Option<String>,
    pub body: String,
    /// Paste the dictation too, as usual.
    pub paste: bool,
}

impl Endpoint {
    /// The endpoint `settings` name, while the lab is on here and a URL is
    /// set; otherwise `None`, and nothing is sent.
    pub fn from_settings(settings: &AppSettings) -> Option<Self> {
        let feature = crate::labs::Feature::SendToEndpoint;
        if !feature.on_this_os() || !feature.on(settings) {
            return None;
        }
        let url = settings.endpoint.url.as_deref()?.trim();
        if url.is_empty() {
            return None;
        }
        let body = match settings.endpoint.body.trim() {
            "" => DEFAULT_ENDPOINT_BODY.to_owned(),
            body => body.to_owned(),
        };
        Some(Self {
            url: url.to_owned(),
            token: settings
                .endpoint
                .token
                .as_deref()
                .map(str::trim)
                .filter(|token| !token.is_empty())
                .map(str::to_owned),
            body,
            paste: settings.endpoint.paste,
        })
    }

    /// POSTs `text` in the body template. `Ok` carries the HTTP status of a
    /// 2xx answer; anything else is a message for a person, without the URL.
    pub fn send(&self, text: &str) -> Result<u16, String> {
        let body = render(&self.body, text);
        let configure = |config: crate::net::Config| {
            config
                .http_status_as_error(false)
                .max_redirects(0)
                .timeout_global(Some(TIMEOUT))
        };
        // A loopback URL never leaves this computer, redirects included;
        // any other host goes through the system's proxy and roots.
        let agent = if is_loopback(&self.url) {
            crate::net::loopback_agent(configure)
        } else {
            crate::net::agent(&self.url, configure)
        };
        let mut builder = ureq::http::Request::builder()
            .method("POST")
            .uri(self.url.as_str())
            .header("Content-Type", "application/json");
        if let Some(token) = &self.token {
            builder = builder.header("Authorization", format!("Bearer {token}"));
        }
        let request = builder
            .body(body.into_bytes())
            .map_err(|error| format!("invalid request: {error}"))?;
        let response = agent
            .run(request)
            .map_err(|error| crate::net::unreachable(&self.url, &error))?;
        let status = response.status().as_u16();
        if (200..300).contains(&status) {
            Ok(status)
        } else {
            Err(format!(
                "{} answered HTTP {status}",
                crate::net::host(&self.url)
            ))
        }
    }
}

/// `template` with every `{text}` replaced by `text` escaped for a JSON
/// string, quotes not included: `{"text": "{text}"}` stays valid JSON for
/// any dictation.
pub fn render(template: &str, text: &str) -> String {
    let quoted = serde_json::to_string(text).unwrap_or_else(|_| "\"\"".into());
    let escaped = quoted
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
        .unwrap_or("");
    template.replace(PLACEHOLDER, escaped)
}

/// Whether `url` names this computer: localhost, 127.0.0.0/8, or ::1.
pub fn is_loopback(url: &str) -> bool {
    url::Url::parse(url)
        .ok()
        .and_then(|url| match url.host()? {
            url::Host::Domain(name) => Some(name.eq_ignore_ascii_case("localhost")),
            url::Host::Ipv4(address) => Some(address.is_loopback()),
            url::Host::Ipv6(address) => Some(address.is_loopback()),
        })
        .unwrap_or(false)
}

/// Why `url` can't be `endpoint.url`, if it can't: an http(s) URL with a host.
pub fn check_url(url: &str) -> Result<(), String> {
    let parsed = url::Url::parse(url).map_err(|error| format!("not a URL ({error})"))?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err("expected an http:// or https:// URL".into());
    }
    if parsed.host().is_none() {
        return Err("expected a URL with a host, such as http://127.0.0.1:1234/command".into());
    }
    Ok(())
}

/// Why `body` can't be `endpoint.body`, if it can't: it names `{text}` and
/// is JSON once a dictation is in.
pub fn check_body(body: &str) -> Result<(), String> {
    if !body.contains(PLACEHOLDER) {
        return Err(format!(
            "expected {PLACEHOLDER} somewhere, where the dictation goes"
        ));
    }
    let sample = render(body, "sample \"dictation\"\nline two");
    serde_json::from_str::<serde_json::Value>(&sample)
        .map(|_| ())
        .map_err(|error| format!("not JSON with a dictation in it ({error})"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_placeholder_takes_the_dictation_escaped_for_json() {
        let body = render(DEFAULT_ENDPOINT_BODY, "say \"hi\"\nnow");
        assert_eq!(body, r#"{"text": "say \"hi\"\nnow"}"#);
        let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(parsed["text"], "say \"hi\"\nnow");
        assert_eq!(render("plain {text} and {text}", "x"), "plain x and x");
    }

    #[test]
    fn urls_are_checked_for_scheme_and_host() {
        assert!(check_url("http://127.0.0.1:1234/command").is_ok());
        assert!(check_url("https://example.com/hook").is_ok());
        assert!(check_url("ftp://127.0.0.1/x").is_err());
        assert!(check_url("127.0.0.1:1234").is_err());
        assert!(check_url("http://").is_err(), "no host at all");
        assert!(check_url("mailto:x@example.com").is_err());
    }

    #[test]
    fn bodies_need_the_placeholder_and_valid_json() {
        assert!(check_body(DEFAULT_ENDPOINT_BODY).is_ok());
        assert!(check_body(r#"{"command": "{text}", "source": "hex"}"#).is_ok());
        assert!(check_body(r#"{"text": "fixed"}"#).is_err());
        assert!(check_body(r#"{"text": {text}}"#).is_err());
    }

    #[test]
    fn loopback_is_told_from_other_hosts() {
        assert!(is_loopback("http://127.0.0.1:5/"));
        assert!(is_loopback("http://localhost:5/"));
        assert!(is_loopback("http://[::1]:5/"));
        assert!(!is_loopback("https://example.com/"));
        assert!(!is_loopback("http://192.168.1.2/"));
        assert!(!is_loopback("not a url"));
    }

    #[test]
    fn nothing_is_sent_while_the_lab_is_off_or_unset() {
        let mut settings = AppSettings::default();
        assert_eq!(Endpoint::from_settings(&settings), None);
        settings.labs.send_to_endpoint = true;
        assert_eq!(Endpoint::from_settings(&settings), None, "no URL yet");
        settings.endpoint.url = Some("  ".into());
        assert_eq!(Endpoint::from_settings(&settings), None, "a blank URL");
        settings.endpoint.url = Some("http://127.0.0.1:1234/command".into());
        settings.endpoint.token = Some(" ".into());
        let endpoint = Endpoint::from_settings(&settings);
        if crate::labs::Feature::SendToEndpoint.on_this_os() {
            let endpoint = endpoint.expect("on, with a URL");
            assert_eq!(endpoint.token, None, "a blank token is none");
            assert_eq!(endpoint.body, DEFAULT_ENDPOINT_BODY);
            assert!(!endpoint.paste);
        } else {
            assert_eq!(endpoint, None, "Windows only");
        }
    }
}
