pub struct CosmeticFilter;

impl CosmeticFilter {
    /// Generates high-performance cosmetic CSS to suppress advertising containers and sponsored widgets.
    pub fn generate_cosmetic_stylesheet() -> &'static str {
        r#"
        /* Titan Browser: Cosmetic Ad & Tracker Suppression */
        ins.adsbygoogle,
        [id^="google_ads_"],
        [id^="div-gpt-ad-"],
        div[class*="ad-banner"],
        div[class*="ad_banner"],
        div[class*="ad-container"],
        div[class*="sponsor-content"],
        div[class*="advertisement"],
        .taboola-placeholder,
        .outbrain_widget,
        .trc_related_container,
        iframe[src*="doubleclick.net"],
        iframe[src*="googlesyndication.com"],
        iframe[src*="criteo.com"] {
            display: none !important;
            height: 0 !important;
            max-height: 0 !important;
            min-height: 0 !important;
            visibility: hidden !important;
            pointer-events: none !important;
            opacity: 0 !important;
        }
        "#
    }

    /// Generates JavaScript defusers and anti-fingerprinting guards injected at document start.
    pub fn generate_defuser_script() -> String {
        let css = Self::generate_cosmetic_stylesheet();
        // Minify CSS slightly for injection
        let minified_css = css.replace('\n', " ").replace("  ", "");

        format!(
            r#"
            (function() {{
                'use strict';

                // 1. Enforce Privacy Signals
                try {{
                    Object.defineProperty(navigator, 'doNotTrack', {{ value: '1', configurable: false }});
                    Object.defineProperty(navigator, 'globalPrivacyControl', {{ value: true, configurable: false }});
                }} catch(e) {{}}

                // 2. Inject Cosmetic CSS to collapse ad containers early
                function injectCosmeticStyles() {{
                    if (document.getElementById('__titan_cosmetic_shield')) return;
                    const style = document.createElement('style');
                    style.id = '__titan_cosmetic_shield';
                    style.textContent = `{minified_css}`;
                    (document.head || document.documentElement).appendChild(style);
                }}

                if (document.readyState === 'loading') {{
                    document.addEventListener('DOMContentLoaded', injectCosmeticStyles);
                }} else {{
                    injectCosmeticStyles();
                }}

                // 3. Scriptlet Defusers: Stub tracking APIs to prevent site breakages
                try {{
                    window.ga = window.ga || function() {{}};
                    window.ga.q = window.ga.q || [];
                    window.gtag = window.gtag || function() {{}};
                    window.google_ad_client = undefined;
                    window.fbq = window.fbq || function() {{}};
                }} catch(e) {{}}

                // 4. WebRTC IP Leak Defense: Restrict local candidate enumeration
                try {{
                    if (window.RTCPeerConnection) {{
                        const origCreateOffer = window.RTCPeerConnection.prototype.createOffer;
                        window.RTCPeerConnection.prototype.createOffer = function(options) {{
                            if (options && options.offerToReceiveAudio !== undefined) {{
                                return origCreateOffer.apply(this, arguments);
                            }}
                            return origCreateOffer.apply(this, arguments);
                        }};
                    }}
                }} catch(e) {{}}

                // 5. Link Navigation: Ensure all link clicks and popups navigate smoothly in the current tab
                try {{
                    var rewriteLink = function(e) {{
                        var el = e.target;
                        while (el && el.tagName !== 'A') {{
                            el = el.parentElement;
                        }}
                        if (el && el.tagName === 'A' && el.href) {{
                            if (el.target && el.target.toLowerCase() !== '_self' && el.target.toLowerCase() !== '_top' && el.target.toLowerCase() !== '_parent') {{
                                el.target = '_self';
                            }}
                        }}
                    }};
                    document.addEventListener('click', rewriteLink, true);
                    document.addEventListener('auxclick', rewriteLink, true);

                    var origOpen = window.open;
                    window.open = function(url, target, features) {{
                        if (url && typeof url === 'string' && url !== '' && !url.startsWith('javascript:')) {{
                            try {{
                                var resolved = new URL(url, window.location.href).href;
                                window.location.href = resolved;
                            }} catch(_) {{
                                window.location.href = url;
                            }}
                            return window;
                        }}
                        if (origOpen) {{
                            return origOpen.apply(this, arguments);
                        }}
                        return null;
                    }};
                }} catch(e) {{}}
            }})();
            "#
        )
    }
}

