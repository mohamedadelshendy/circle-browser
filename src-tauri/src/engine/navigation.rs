use url::Url;

pub struct NavigationResolver;

impl NavigationResolver {
    /// Determines whether an input string is a direct URL or a search query.
    /// Returns a validated, canonical target URL.
    pub fn resolve(input: &str, default_search_engine: &str) -> String {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return "browser://newtab".to_string();
        }

        // Internal schema handling
        if trimmed == "browser://newtab" || trimmed == "about:blank" {
            return trimmed.to_string();
        }

        // Check if an explicit scheme is provided
        if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
            if let Ok(parsed) = Url::parse(trimmed) {
                return parsed.to_string();
            }
        }

        // Detect domain-like patterns without protocol (e.g. "github.com", "localhost:3000", "sub.domain.org/path")
        let has_spaces = trimmed.contains(' ');
        let has_dot = trimmed.contains('.');
        let is_localhost = trimmed.starts_with("localhost");

        if !has_spaces && (has_dot || is_localhost) {
            let candidate = format!("https://{}", trimmed);
            if let Ok(parsed) = Url::parse(&candidate) {
                if let Some(host) = parsed.host_str() {
                    if host.contains('.') || host == "localhost" {
                        return candidate;
                    }
                }
            }
        }

        // Fallback: encode as search query defaulting to Google Search
        let encoded_query: String = url::form_urlencoded::byte_serialize(trimmed.as_bytes()).collect();

        match default_search_engine.to_lowercase().as_str() {
            "duckduckgo" => format!("https://duckduckgo.com/?q={}", encoded_query),
            _ => format!("https://www.google.com/search?q={}", encoded_query),
        }
    }
}
