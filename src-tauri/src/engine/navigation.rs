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

    /// Derives a clean, professional, human-readable title from a URL.
    pub fn title_from_url(url_str: &str) -> String {
        let trimmed = url_str.trim();
        if trimmed.is_empty()
            || trimmed == "browser://newtab"
            || trimmed.contains("newtab.html")
            || trimmed == "about:blank"
        {
            return "New tab".to_string();
        }

        if let Ok(u) = Url::parse(trimmed) {
            if let Some(host) = u.host_str() {
                let clean_host = host.strip_prefix("www.").unwrap_or(host);

                // Google search results
                if clean_host == "google.com" || clean_host.ends_with(".google.com") {
                    if let Some((_, q)) = u.query_pairs().find(|(k, _)| k == "q") {
                        if !q.trim().is_empty() {
                            return format!("{} - Google Search", q.trim());
                        }
                    }
                    return "Google".to_string();
                }

                // YouTube
                if clean_host == "youtube.com" || clean_host == "m.youtube.com" || clean_host == "youtu.be" {
                    if let Some((_, v)) = u.query_pairs().find(|(k, _)| k == "v") {
                        if !v.trim().is_empty() {
                            return "YouTube".to_string();
                        }
                    }
                    return "YouTube".to_string();
                }

                // GitHub
                if clean_host == "github.com" {
                    let path = u.path().trim_matches('/');
                    if !path.is_empty() {
                        return format!("{path} · GitHub");
                    }
                    return "GitHub".to_string();
                }

                // Wikipedia
                if clean_host == "wikipedia.org" || clean_host.ends_with(".wikipedia.org") {
                    let path = u.path().trim_matches('/');
                    if let Some(article) = path.strip_prefix("wiki/") {
                        let clean_title = article.replace("%20", " ").replace('_', " ");
                        return format!("{clean_title} - Wikipedia");
                    }
                    return "Wikipedia".to_string();
                }

                // Reddit
                if clean_host == "reddit.com" || clean_host.ends_with(".reddit.com") {
                    let path = u.path().trim_matches('/');
                    if !path.is_empty() {
                        return format!("{path} · Reddit");
                    }
                    return "Reddit".to_string();
                }

                // Generic capitalized domain name: e.g. "amazon.com" -> "Amazon"
                let domain_parts: Vec<&str> = clean_host.split('.').collect();
                if domain_parts.len() >= 2 {
                    let main_name = domain_parts[domain_parts.len() - 2];
                    let mut chars = main_name.chars();
                    if let Some(first) = chars.next() {
                        return first.to_uppercase().collect::<String>() + chars.as_str();
                    }
                }
                return clean_host.to_string();
            }
        }

        trimmed.to_string()
    }
}
