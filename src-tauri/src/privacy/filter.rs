use std::collections::HashSet;
use url::Url;

#[derive(Debug, Clone)]
pub struct BlockDecision {
    pub should_block: bool,
    pub reason: Option<String>,
    pub cleaned_url: Option<String>,
}

pub struct PrivacyEngine {
    blocked_domains: HashSet<&'static str>,
    tracking_params: HashSet<&'static str>,
}

impl Default for PrivacyEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl PrivacyEngine {
    pub fn new() -> Self {
        // High-frequency advertising, tracking, analytics, and telemetry host patterns
        let mut blocked_domains = HashSet::new();

        // Google / DoubleClick telemetry & ads
        blocked_domains.insert("doubleclick.net");
        blocked_domains.insert("googleadservices.com");
        blocked_domains.insert("googlesyndication.com");
        blocked_domains.insert("google-analytics.com");
        blocked_domains.insert("analytics.google.com");
        blocked_domains.insert("adservice.google.com");

        // Meta / Facebook telemetry & tracking pixels
        blocked_domains.insert("connect.facebook.net");
        blocked_domains.insert("graph.facebook.com");
        blocked_domains.insert("pixel.facebook.com");
        blocked_domains.insert("ads.facebook.com");

        // Major ad exchanges & trackers
        blocked_domains.insert("criteo.com");
        blocked_domains.insert("criteo.net");
        blocked_domains.insert("taboola.com");
        blocked_domains.insert("outbrain.com");
        blocked_domains.insert("adnxs.com");
        blocked_domains.insert("adroll.com");
        blocked_domains.insert("scorecardresearch.com");
        blocked_domains.insert("quantserve.com");
        blocked_domains.insert("pubmatic.com");
        blocked_domains.insert("rubiconproject.com");
        blocked_domains.insert("casale-media.com");
        blocked_domains.insert("openx.net");
        blocked_domains.insert("advertising.com");
        blocked_domains.insert("serving-sys.com");

        // Behavioral & session recording trackers
        blocked_domains.insert("hotjar.com");
        blocked_domains.insert("hotjar.io");
        blocked_domains.insert("clarity.ms");
        blocked_domains.insert("amplitude.com");
        blocked_domains.insert("segment.com");
        blocked_domains.insert("segment.io");
        blocked_domains.insert("mixpanel.com");
        blocked_domains.insert("fullstory.com");
        blocked_domains.insert("mouseflow.com");
        blocked_domains.insert("yandex.metrika");
        blocked_domains.insert("mc.yandex.ru");
        blocked_domains.insert("smartlook.com");
        blocked_domains.insert("appsflyer.com");
        blocked_domains.insert("branch.io");

        // URL tracking & surveillance query parameters
        let mut tracking_params = HashSet::new();
        tracking_params.insert("fbclid");
        tracking_params.insert("gclid");
        tracking_params.insert("dclid");
        tracking_params.insert("gbraid");
        tracking_params.insert("wbraid");
        tracking_params.insert("msclkid");
        tracking_params.insert("yclid");
        tracking_params.insert("mc_eid");
        tracking_params.insert("utm_source");
        tracking_params.insert("utm_medium");
        tracking_params.insert("utm_campaign");
        tracking_params.insert("utm_term");
        tracking_params.insert("utm_content");
        tracking_params.insert("utm_id");
        tracking_params.insert("utm_reader");
        tracking_params.insert("_hsenc");
        tracking_params.insert("_hsmi");
        tracking_params.insert("__twitter_impression");

        Self {
            blocked_domains,
            tracking_params,
        }
    }

    /// Evaluates a URL against privacy blocklists and strips surveillance query parameters.
    pub fn evaluate_url(&self, raw_url: &str) -> BlockDecision {
        let parsed = match Url::parse(raw_url) {
            Ok(u) => u,
            Err(_) => {
                return BlockDecision {
                    should_block: false,
                    reason: None,
                    cleaned_url: None,
                }
            }
        };

        // 1. Host-based blocklist evaluation
        if let Some(host) = parsed.host_str() {
            let host_lower = host.to_lowercase();
            for blocked in &self.blocked_domains {
                if host_lower == *blocked || host_lower.ends_with(&format!(".{blocked}")) {
                    return BlockDecision {
                        should_block: true,
                        reason: Some(format!("Blocked tracker / ad domain: {blocked}")),
                        cleaned_url: None,
                    };
                }
            }
        }

        // 2. Query parameter de-tracking
        let mut has_tracking = false;
        if parsed.query().is_some() {
            let filtered_pairs: Vec<(String, String)> = parsed
                .query_pairs()
                .filter(|(key, _)| {
                    let is_tracker = self.tracking_params.contains(key.as_ref());
                    if is_tracker {
                        has_tracking = true;
                    }
                    !is_tracker
                })
                .map(|(k, v)| (k.into_owned(), v.into_owned()))
                .collect();

            if has_tracking {
                let mut cleaned = parsed.clone();
                cleaned.set_query(None);
                if !filtered_pairs.is_empty() {
                    let mut serializer = cleaned.query_pairs_mut();
                    for (k, v) in filtered_pairs {
                        serializer.append_pair(&k, &v);
                    }
                }
                return BlockDecision {
                    should_block: false,
                    reason: Some("Stripped tracking query parameters".to_string()),
                    cleaned_url: Some(cleaned.to_string()),
                };
            }
        }

        BlockDecision {
            should_block: false,
            reason: None,
            cleaned_url: None,
        }
    }

    /// Checks if a hostname matches any blocked tracker/ad pattern.
    pub fn is_blocked_host(&self, host: &str) -> bool {
        let lower = host.to_lowercase();
        self.blocked_domains
            .iter()
            .any(|blocked| lower == *blocked || lower.ends_with(&format!(".{blocked}")))
    }
}

