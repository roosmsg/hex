//! Labs › Send to endpoint: a dictation that starts with an endpoint's
//! trigger word goes to that local HTTP endpoint as JSON, without the
//! trigger word, for apps that take spoken commands ("Restage, open
//! research"). An endpoint without a trigger word takes every dictation no
//! trigger word claims. `labs.send_to_endpoint` and `endpoints`; Windows.
//!
//! Gated where the work starts: [`from_settings`] is empty while the lab is
//! off, and the output worker then pastes as it always did.
//!
//! Logs name the endpoint and its host and the HTTP status, never the URL's
//! path, the token, or the body, which carries dictated text.

use std::collections::HashSet;
use std::time::Duration;

use unicode_normalization::UnicodeNormalization;

use crate::app_settings::{AppSettings, DEFAULT_ENDPOINT_BODY, EndpointSettings, MAX_ENDPOINTS};

/// What an endpoint's body replaces with the dictation.
pub const PLACEHOLDER: &str = "{text}";

/// What `hex settings get endpoints` shows in place of a token.
pub const MASKED_TOKEN: &str = "(set)";

/// How long one POST may take, connection included.
const TIMEOUT: Duration = Duration::from_secs(10);

/// A trigger word may span this many spoken words ("hey restage").
const MAX_TRIGGER_WORDS: usize = 4;

/// One endpoint as the output worker sends with: a snapshot, replaced
/// whenever the settings are saved.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Endpoint {
    pub name: String,
    /// The trigger word as letters and digits only, lower case; empty for
    /// the endpoint that takes every unclaimed dictation.
    pub trigger: String,
    pub url: String,
    pub token: Option<String>,
    pub body: String,
    /// Paste what was sent too, as usual.
    pub paste: bool,
}

/// The endpoints `settings` hold that can receive, while the lab is on
/// here: those with a URL. Empty otherwise, and nothing is sent.
pub fn from_settings(settings: &AppSettings) -> Vec<Endpoint> {
    let feature = crate::labs::Feature::SendToEndpoint;
    if !feature.on_this_os() || !feature.on(settings) {
        return Vec::new();
    }
    settings
        .endpoints
        .iter()
        .take(MAX_ENDPOINTS)
        .filter_map(Endpoint::from_entry)
        .collect()
}

impl Endpoint {
    fn from_entry(entry: &EndpointSettings) -> Option<Self> {
        let url = entry.url.as_deref()?.trim();
        if url.is_empty() {
            return None;
        }
        let body = match entry.body.trim() {
            "" => DEFAULT_ENDPOINT_BODY.to_owned(),
            body => body.to_owned(),
        };
        Some(Self {
            name: match entry.name.trim() {
                "" => crate::net::host(url),
                name => name.to_owned(),
            },
            trigger: fold(&entry.trigger),
            url: url.to_owned(),
            token: entry
                .token
                .as_deref()
                .map(str::trim)
                .filter(|token| !token.is_empty())
                .map(str::to_owned),
            body,
            paste: entry.paste,
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

/// The endpoint `text` is for and what to send it: the text after the
/// longest trigger word it starts with, or, when no trigger word matches,
/// the whole text for the endpoint without one. `None`: paste as usual.
///
/// A trigger word matches however speech splits or punctuates it: "Re-stage,
/// open research" and "re stage open research" both start with "Restage".
pub fn route<'a>(endpoints: &'a [Endpoint], text: &str) -> Option<(&'a Endpoint, String)> {
    let words = words(text);
    let mut best: Option<(&Endpoint, usize)> = None;
    for endpoint in endpoints
        .iter()
        .filter(|endpoint| !endpoint.trigger.is_empty())
    {
        let mut joined = String::new();
        for (_, end, word) in words.iter().take(MAX_TRIGGER_WORDS) {
            joined.push_str(word);
            if joined.len() > endpoint.trigger.len() {
                break;
            }
            if joined == endpoint.trigger {
                if best.is_none_or(|(current, _)| endpoint.trigger.len() > current.trigger.len()) {
                    best = Some((endpoint, *end));
                }
                break;
            }
        }
    }
    if let Some((endpoint, end)) = best {
        let rest = text[end..].trim_start_matches(|c: char| {
            c.is_whitespace() || matches!(c, ',' | '.' | ':' | ';' | '!' | '?' | '-' | '–' | '—')
        });
        return Some((endpoint, rest.trim_end().to_owned()));
    }
    endpoints
        .iter()
        .find(|endpoint| endpoint.trigger.is_empty())
        .map(|endpoint| (endpoint, text.trim().to_owned()))
}

/// The words of `text` with where each ends, folded for comparison.
fn words(text: &str) -> Vec<(usize, usize, String)> {
    let mut out = Vec::new();
    let mut start = None;
    for (index, c) in text.char_indices() {
        match (c.is_whitespace(), start) {
            (false, None) => start = Some(index),
            (true, Some(from)) => {
                out.push((from, index, fold(&text[from..index])));
                start = None;
            }
            _ => {}
        }
    }
    if let Some(from) = start {
        out.push((from, text.len(), fold(&text[from..])));
    }
    out.retain(|(_, _, word)| !word.is_empty());
    out
}

/// Letters and digits only, lower case, accents dropped: "Ré-Stage," → "restage".
pub fn fold(text: &str) -> String {
    text.nfd()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
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

/// Why `url` can't be an endpoint's URL, if it can't: an http(s) URL with a host.
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

/// Why `body` can't be an endpoint's body, if it can't: it names `{text}`
/// and is JSON once a dictation is in.
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

/// Why `endpoints` can't be saved, if they can't: at most
/// [`MAX_ENDPOINTS`], valid URLs and bodies, no trigger word twice, and at
/// most one endpoint without a trigger word.
pub fn check_all(endpoints: &[EndpointSettings]) -> Result<(), String> {
    if endpoints.len() > MAX_ENDPOINTS {
        return Err(format!("at most {MAX_ENDPOINTS} endpoints"));
    }
    let mut triggers = HashSet::new();
    let mut catch_all = false;
    for (index, entry) in endpoints.iter().enumerate() {
        let which = match entry.name.trim() {
            "" => format!("endpoint {}", index + 1),
            name => format!("endpoint {name:?}"),
        };
        if let Some(url) = entry.url.as_deref().filter(|url| !url.trim().is_empty()) {
            check_url(url.trim()).map_err(|why| format!("{which}: {why}"))?;
        }
        if !entry.body.trim().is_empty() {
            check_body(entry.body.trim()).map_err(|why| format!("{which}: {why}"))?;
        }
        let trigger = fold(&entry.trigger);
        if trigger.is_empty() {
            if std::mem::replace(&mut catch_all, true) {
                return Err(format!(
                    "{which}: only one endpoint may go without a trigger word"
                ));
            }
        } else if !triggers.insert(trigger) {
            return Err(format!(
                "{which}: another endpoint has the trigger word {:?}",
                entry.trigger.trim()
            ));
        }
    }
    Ok(())
}

/// `endpoints` as `hex settings` shows them: each token as [`MASKED_TOKEN`].
pub fn masked(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Array(entries) => serde_json::Value::Array(
            entries
                .into_iter()
                .map(|mut entry| {
                    if let Some(token) = entry.get_mut("token")
                        && token.as_str().is_some_and(|token| !token.is_empty())
                    {
                        *token = serde_json::Value::String(MASKED_TOKEN.into());
                    }
                    entry
                })
                .collect(),
        ),
        other => other,
    }
}

/// A list set back with masked tokens keeps the tokens it had: an entry whose
/// token is [`MASKED_TOKEN`] takes the token of the saved endpoint with the
/// same name or, renamed, the same URL. One that matches neither is refused
/// rather than saved without its token.
pub fn keep_masked_tokens(
    new: &mut [EndpointSettings],
    saved: &[EndpointSettings],
) -> Result<(), String> {
    for entry in new {
        if entry.token.as_deref() != Some(MASKED_TOKEN) {
            continue;
        }
        let same_url = |old: &&EndpointSettings| {
            old.url.as_deref().map(str::trim) == entry.url.as_deref().map(str::trim)
        };
        let old = saved
            .iter()
            .find(|old| old.name.trim() == entry.name.trim())
            .or_else(|| saved.iter().find(same_url))
            .ok_or_else(|| {
                format!(
                    "no saved endpoint has the name or URL of `{}`, so `{MASKED_TOKEN}` keeps no token; give the token itself",
                    entry.name.trim()
                )
            })?;
        entry.token = old.token.clone();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn endpoint(name: &str, trigger: &str) -> Endpoint {
        Endpoint {
            name: name.into(),
            trigger: fold(trigger),
            url: format!("http://127.0.0.1:1/{name}"),
            token: None,
            body: DEFAULT_ENDPOINT_BODY.into(),
            paste: false,
        }
    }

    #[test]
    fn a_trigger_word_sends_the_rest_to_its_endpoint() {
        let endpoints = [endpoint("restage", "Restage"), endpoint("lights", "lights")];
        let (to, rest) = route(&endpoints, "Restage, open research.").unwrap();
        assert_eq!(to.name, "restage");
        assert_eq!(rest, "open research.");
        let (to, rest) = route(&endpoints, "Lights: dim to 30%").unwrap();
        assert_eq!(to.name, "lights");
        assert_eq!(rest, "dim to 30%");
    }

    #[test]
    fn a_trigger_word_matches_however_speech_splits_it() {
        let endpoints = [endpoint("restage", "Restage")];
        for spoken in [
            "re stage open research",
            "Re-stage open research",
            "RESTAGE — open research",
            "Réstage open research",
        ] {
            let (_, rest) =
                route(&endpoints, spoken).unwrap_or_else(|| panic!("{spoken:?} went nowhere"));
            assert_eq!(rest, "open research", "{spoken:?}");
        }
    }

    #[test]
    fn a_word_that_only_starts_like_the_trigger_is_not_it() {
        let endpoints = [endpoint("restage", "Restage")];
        assert!(route(&endpoints, "Restaged the play yesterday").is_none());
        assert!(route(&endpoints, "Rest is good").is_none());
        assert!(
            route(&endpoints, "I said Restage open").is_none(),
            "only at the start"
        );
    }

    #[test]
    fn the_longest_trigger_word_wins() {
        let endpoints = [endpoint("short", "hey"), endpoint("long", "hey restage")];
        assert_eq!(
            route(&endpoints, "Hey Restage, open x").unwrap().0.name,
            "long"
        );
        assert_eq!(route(&endpoints, "Hey, open x").unwrap().0.name, "short");
    }

    #[test]
    fn an_endpoint_without_a_trigger_word_takes_the_rest() {
        let endpoints = [endpoint("restage", "Restage"), endpoint("all", "")];
        assert_eq!(
            route(&endpoints, "Restage, open research").unwrap().0.name,
            "restage"
        );
        let (to, rest) = route(&endpoints, "  just some text ").unwrap();
        assert_eq!((to.name.as_str(), rest.as_str()), ("all", "just some text"));
        assert!(
            route(&endpoints[..1], "just some text").is_none(),
            "pasted as usual"
        );
    }

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
    fn the_list_refuses_doubled_triggers_and_two_catch_alls() {
        let entry = |name: &str, trigger: &str| EndpointSettings {
            name: name.into(),
            trigger: trigger.into(),
            url: Some("http://127.0.0.1:1/x".into()),
            ..Default::default()
        };
        assert!(check_all(&[entry("a", "Restage"), entry("b", "lights"), entry("c", "")]).is_ok());
        assert!(check_all(&[entry("a", "Restage"), entry("b", "re stage")]).is_err());
        assert!(check_all(&[entry("a", ""), entry("b", " ")]).is_err());
        let mut bad = entry("a", "x");
        bad.url = Some("not a url".into());
        assert!(check_all(&[bad]).is_err());
        assert!(check_all(&vec![EndpointSettings::default(); MAX_ENDPOINTS + 1]).is_err());
    }

    #[test]
    fn tokens_are_masked_and_kept_when_set_back() {
        let saved = vec![EndpointSettings {
            name: "Restage".into(),
            token: Some("secret".into()),
            ..Default::default()
        }];
        let shown = masked(serde_json::to_value(&saved).unwrap());
        assert_eq!(shown[0]["token"], MASKED_TOKEN);
        let mut back: Vec<EndpointSettings> = serde_json::from_value(shown).unwrap();
        keep_masked_tokens(&mut back, &saved).unwrap();
        assert_eq!(back[0].token.as_deref(), Some("secret"));
    }

    #[test]
    fn a_renamed_endpoint_keeps_its_token_by_url_and_a_stranger_is_refused() {
        let saved = vec![EndpointSettings {
            name: String::new(),
            url: Some("http://127.0.0.1:5/command".into()),
            token: Some("secret".into()),
            ..Default::default()
        }];
        let mut renamed = vec![EndpointSettings {
            name: "Restage".into(),
            url: Some("http://127.0.0.1:5/command".into()),
            token: Some(MASKED_TOKEN.into()),
            ..Default::default()
        }];
        keep_masked_tokens(&mut renamed, &saved).unwrap();
        assert_eq!(renamed[0].token.as_deref(), Some("secret"));
        let mut stranger = vec![EndpointSettings {
            name: "Lights".into(),
            url: Some("http://127.0.0.1:6/".into()),
            token: Some(MASKED_TOKEN.into()),
            ..Default::default()
        }];
        assert!(keep_masked_tokens(&mut stranger, &saved).is_err());
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
        settings.endpoints.push(EndpointSettings {
            name: "Restage".into(),
            trigger: "Restage".into(),
            url: Some("http://127.0.0.1:1234/command".into()),
            token: Some(" ".into()),
            ..Default::default()
        });
        settings.endpoints.push(EndpointSettings::default());
        assert!(from_settings(&settings).is_empty(), "the lab is off");
        settings.labs.send_to_endpoint = true;
        let endpoints = from_settings(&settings);
        if crate::labs::Feature::SendToEndpoint.on_this_os() {
            assert_eq!(
                endpoints.len(),
                1,
                "an endpoint without a URL receives nothing"
            );
            assert_eq!(endpoints[0].token, None, "a blank token is none");
            assert_eq!(endpoints[0].trigger, "restage");
        } else {
            assert!(endpoints.is_empty(), "Windows only");
        }
    }
}
