//! Extended Selector Resolution
//!
//! Supports selector prefixes (inspired by automodus):
//! - `text:` — Find by exact text content (walks up to clickable ancestor)
//! - `text*:` — Find by partial text content
//! - `role:` — Find by ARIA role and accessible name (e.g. `role:button[Submit]`)
//! - `xpath:` — XPath selector
//! - `>>` — Chain operator: `scope >> inner` finds inner within scope, returns scope
//! - (none) — CSS selector (default)

/// Parsed selector type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectorType {
    /// Standard CSS selector
    Css(String),
    /// Find by exact text content
    Text(String),
    /// Find by partial text content
    TextPartial(String),
    /// Find by ARIA role with optional accessible name
    Role { role: String, name: Option<String> },
    /// XPath selector
    XPath(String),
    /// Chain: find right selector within left's matches, return the left (container) element.
    /// Syntax: `[role="button"] >> text:Login`
    Chain(Box<SelectorType>, Box<SelectorType>),
}

impl SelectorType {
    pub fn is_css(&self) -> bool {
        matches!(self, SelectorType::Css(_))
    }

    pub fn as_css(&self) -> Option<&str> {
        match self {
            SelectorType::Css(s) => Some(s),
            _ => None,
        }
    }
}

/// Parse a selector string into its type
pub fn parse_selector(selector: &str) -> SelectorType {
    // Check for chain operator first: `scope >> inner`
    if let Some((left, right)) = selector.split_once(" >> ") {
        return SelectorType::Chain(
            Box::new(parse_single_selector(left.trim())),
            Box::new(parse_single_selector(right.trim())),
        );
    }
    parse_single_selector(selector)
}

fn parse_single_selector(selector: &str) -> SelectorType {
    if let Some(text) = selector.strip_prefix("text:") {
        SelectorType::Text(text.to_string())
    } else if let Some(text) = selector.strip_prefix("text*:") {
        SelectorType::TextPartial(text.to_string())
    } else if let Some(role_spec) = selector.strip_prefix("role:") {
        parse_role_selector(role_spec)
    } else if let Some(xpath) = selector.strip_prefix("xpath:") {
        SelectorType::XPath(xpath.to_string())
    } else {
        SelectorType::Css(selector.to_string())
    }
}

fn parse_role_selector(spec: &str) -> SelectorType {
    if let Some(bracket_pos) = spec.find('[') {
        let role = spec[..bracket_pos].trim().to_string();
        let name = spec[bracket_pos + 1..].trim_end_matches(']').to_string();
        SelectorType::Role {
            role,
            name: if name.is_empty() { None } else { Some(name) },
        }
    } else {
        SelectorType::Role {
            role: spec.to_string(),
            name: None,
        }
    }
}

/// Generate JavaScript to find an element by selector type.
/// Returns JS expression that evaluates to the element or null.
pub fn selector_to_js(selector: &SelectorType) -> String {
    match selector {
        SelectorType::Css(css) => {
            format!(
                "document.querySelector({})",
                serde_json::to_string(css).unwrap()
            )
        }
        SelectorType::Text(text) => {
            // TreeWalker on text nodes — simplest, most robust approach.
            // Text nodes are atomic (no children), so textContent is always just their own text.
            // Walk up from the text node's parent to find the nearest clickable ancestor.
            format!(
                r#"(function() {{
    var text = {};
    function isClickable(el) {{
        if (!el) return false;
        var tag = el.tagName && el.tagName.toLowerCase();
        if (tag === 'button' || tag === 'a') return true;
        if (el.getAttribute('role') === 'button') return true;
        if (el.getAttribute('tabindex') !== null) return true;
        if (el.onclick) return true;
        return false;
    }}
    var w = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT);
    while (w.nextNode()) {{
        if (w.currentNode.textContent.trim() === text) {{
            var el = w.currentNode.parentElement;
            while (el) {{
                if (isClickable(el)) return el;
                el = el.parentElement;
            }}
            return w.currentNode.parentElement;
        }}
    }}
    return null;
}})()"#,
                serde_json::to_string(text).unwrap()
            )
        }
        SelectorType::TextPartial(text) => {
            format!(
                r#"(function() {{
    var text = {};
    var lc = text.toLowerCase();
    function isClickable(el) {{
        if (!el) return false;
        var tag = el.tagName && el.tagName.toLowerCase();
        if (tag === 'button' || tag === 'a') return true;
        if (el.getAttribute('role') === 'button') return true;
        if (el.getAttribute('tabindex') !== null) return true;
        if (el.onclick) return true;
        return false;
    }}
    var w = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT);
    while (w.nextNode()) {{
        if (w.currentNode.textContent.toLowerCase().includes(lc)) {{
            var el = w.currentNode.parentElement;
            while (el) {{
                if (isClickable(el)) return el;
                el = el.parentElement;
            }}
            return w.currentNode.parentElement;
        }}
    }}
    return null;
}})()"#,
                serde_json::to_string(text).unwrap()
            )
        }
        SelectorType::Role { role, name } => match name {
            Some(name) => format!(
                r#"(function() {{
    var role = {};
    var name = {};
    var byRole = document.querySelectorAll('[role="' + role + '"]');
    for (var i = 0; i < byRole.length; i++) {{
        var n = byRole[i].getAttribute('aria-label') || (byRole[i].innerText || '').trim();
        if (n === name || n.includes(name)) return byRole[i];
    }}
    var implicit = {{'button':'button,input[type="button"],input[type="submit"]','link':'a[href]','textbox':'input[type="text"],input:not([type]),textarea','checkbox':'input[type="checkbox"]'}};
    var sel = implicit[role];
    if (sel) {{
        var els = document.querySelectorAll(sel);
        for (var i = 0; i < els.length; i++) {{
            var n = els[i].getAttribute('aria-label') || els[i].value || (els[i].innerText || '').trim();
            if (n === name || n.includes(name)) return els[i];
        }}
    }}
    return null;
}})()"#,
                serde_json::to_string(role).unwrap(),
                serde_json::to_string(name).unwrap()
            ),
            None => format!(
                r#"(function() {{
    var role = {};
    var el = document.querySelector('[role="' + role + '"]');
    if (el) return el;
    var implicit = {{'button':'button','link':'a[href]','textbox':'input[type="text"],textarea','checkbox':'input[type="checkbox"]'}};
    if (implicit[role]) return document.querySelector(implicit[role]);
    return null;
}})()"#,
                serde_json::to_string(role).unwrap()
            ),
        },
        SelectorType::XPath(xpath) => {
            format!(
                "document.evaluate({}, document, null, XPathResult.FIRST_ORDERED_NODE_TYPE, null).singleNodeValue",
                serde_json::to_string(xpath).unwrap()
            )
        }
        SelectorType::Chain(left, right) => {
            let containers = containers_js(left);
            let check = scoped_match_js(right);
            format!(
                r#"(function() {{
    var cs = {containers};
    for (var i = 0; i < cs.length; i++) {{
        var c = cs[i];
        {check}
    }}
    return null;
}})()"#,
                containers = containers,
                check = check,
            )
        }
    }
}

/// JS expression that returns a collection of elements matching the selector.
fn containers_js(selector: &SelectorType) -> String {
    match selector {
        SelectorType::Css(css) => format!(
            "document.querySelectorAll({})",
            serde_json::to_string(css).unwrap()
        ),
        SelectorType::Role { role, name: None } => format!(
            "document.querySelectorAll({})",
            serde_json::to_string(&format!("[role=\"{}\"]", role)).unwrap()
        ),
        // For everything else, find single match and wrap in array
        _ => format!(
            "(function() {{ var r = {}; return r ? [r] : []; }})()",
            selector_to_js(selector)
        ),
    }
}

/// JS code to check if `right` matches inside container variable `c`, returning `c` if found.
fn scoped_match_js(selector: &SelectorType) -> String {
    match selector {
        SelectorType::Css(css) => format!(
            "if (c.querySelector({})) return c;",
            serde_json::to_string(css).unwrap()
        ),
        SelectorType::Text(text) => format!(
            "var w = document.createTreeWalker(c, NodeFilter.SHOW_TEXT); while (w.nextNode()) {{ if (w.currentNode.textContent.trim() === {}) return c; }}",
            serde_json::to_string(text).unwrap()
        ),
        SelectorType::TextPartial(text) => format!(
            "var _lc = {}.toLowerCase(); var w = document.createTreeWalker(c, NodeFilter.SHOW_TEXT); while (w.nextNode()) {{ if (w.currentNode.textContent.toLowerCase().includes(_lc)) return c; }}",
            serde_json::to_string(text).unwrap()
        ),        // For anything else, find via full selector and check containment
        _ => format!(
            "var _inner = {}; if (_inner && c.contains(_inner)) return c;",
            selector_to_js(selector)
        ),
    }
}

/// JS to check if element exists (returns boolean).
/// For text selectors, does a fast body.innerText check instead of full DOM walk.
pub fn selector_exists_js(selector: &SelectorType) -> String {
    match selector {
        SelectorType::Text(text) => {
            format!(
                "document.body && document.body.innerText.includes({})",
                serde_json::to_string(text).unwrap()
            )
        }
        SelectorType::TextPartial(text) => {
            format!(
                "document.body && document.body.innerText.toLowerCase().includes({})",
                serde_json::to_string(&text.to_lowercase()).unwrap()
            )
        }
        _ => format!("({}) !== null", selector_to_js(selector)),
    }
}

/// JS to click an element (scrolls into view first)
pub fn selector_click_js(selector: &SelectorType) -> String {
    format!(
        r#"(function() {{
    var el = {};
    if (!el) return false;
    el.scrollIntoView({{ behavior: 'instant', block: 'center' }});
    el.click();
    return true;
}})()"#,
        selector_to_js(selector)
    )
}

/// JS to get element's text content
pub fn selector_get_text_js(selector: &SelectorType) -> String {
    format!(
        r#"(function() {{
    var el = {};
    if (!el) return null;
    return el.innerText || el.textContent || el.value || '';
}})()"#,
        selector_to_js(selector)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_css_selector() {
        assert_eq!(
            parse_selector("button.submit"),
            SelectorType::Css("button.submit".to_string())
        );
        assert_eq!(
            parse_selector("#my-id"),
            SelectorType::Css("#my-id".to_string())
        );
    }

    #[test]
    fn test_parse_text_selector() {
        assert_eq!(
            parse_selector("text:Log in"),
            SelectorType::Text("Log in".to_string())
        );
        assert_eq!(
            parse_selector("text*:phone"),
            SelectorType::TextPartial("phone".to_string())
        );
    }

    #[test]
    fn test_parse_role_selector() {
        assert_eq!(
            parse_selector("role:button"),
            SelectorType::Role {
                role: "button".to_string(),
                name: None
            }
        );
        assert_eq!(
            parse_selector("role:button[Next]"),
            SelectorType::Role {
                role: "button".to_string(),
                name: Some("Next".to_string())
            }
        );
    }

    #[test]
    fn test_parse_xpath_selector() {
        assert_eq!(
            parse_selector("xpath://button"),
            SelectorType::XPath("//button".to_string())
        );
    }

    #[test]
    fn test_css_generates_queryselector() {
        let sel = parse_selector("#pane-side");
        let js = selector_to_js(&sel);
        assert!(js.contains("querySelector"));
    }

    #[test]
    fn test_text_generates_treewalker() {
        let sel = parse_selector("text:Next");
        let js = selector_to_js(&sel);
        assert!(js.contains("createTreeWalker"));
        assert!(js.contains("isClickable"));
    }

    #[test]
    fn test_parse_chain_selector() {
        let sel = parse_selector(r#"[role="button"] >> text:Login"#);
        assert_eq!(
            sel,
            SelectorType::Chain(
                Box::new(SelectorType::Css(r#"[role="button"]"#.to_string())),
                Box::new(SelectorType::Text("Login".to_string())),
            )
        );
    }

    #[test]
    fn test_chain_generates_scoped_walk() {
        let sel = parse_selector(r#"[role="button"] >> text:Submit"#);
        let js = selector_to_js(&sel);
        assert!(js.contains("querySelectorAll"));
        assert!(js.contains("createTreeWalker"));
    }
}
