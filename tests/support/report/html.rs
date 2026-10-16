//! HTML escaping and template filling for the reference comparison report.

/// Escapes text for element content and double-quoted attribute values.
///
/// Everything injected into the report goes through this function: case
/// fixtures, renderer output, error strings, and repository metadata.
pub fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Fills `{{NAME}}` placeholders in a report template asset.
///
/// Substituted values are inserted verbatim and never re-scanned, so CSS and
/// JavaScript assets may contain arbitrary brace syntax. Unknown placeholders
/// panic, which fails the reference test instead of shipping a broken report.
pub fn render_template(template: &str, values: &[(&str, &str)]) -> String {
    let mut rendered = String::with_capacity(template.len() + 4096);
    let mut rest = template;
    while let Some(start) = rest.find("{{") {
        rendered.push_str(&rest[..start]);
        let candidate = &rest[start + 2..];
        match candidate.find("}}") {
            Some(end) => {
                let key = &candidate[..end];
                match values.iter().find(|(name, _)| *name == key) {
                    Some((_, value)) => rendered.push_str(value),
                    None => panic!("unknown report template placeholder `{{{{{key}}}}}`"),
                }
                rest = &candidate[end + 2..];
            }
            None => {
                rendered.push_str(&rest[start..]);
                rest = "";
            }
        }
    }
    rendered.push_str(rest);
    rendered
}

#[cfg(test)]
mod tests {
    use super::{escape_html, render_template};

    #[test]
    fn escaping_covers_every_html_metacharacter() {
        assert_eq!(escape_html("a&b<c>\"d\""), "a&amp;b&lt;c&gt;&quot;d&quot;");
    }

    #[test]
    fn template_values_never_render_placeholders_again() {
        let rendered =
            render_template("x={{A}} y={{B}}", &[("A", "{{B}}"), ("B", "!"), ("C", "?")]);
        assert_eq!(rendered, "x={{B}} y=!");
    }

    #[test]
    #[should_panic(expected = "unknown report template placeholder")]
    fn unknown_placeholders_panic() {
        render_template("{{MISSING}}", &[("PRESENT", "value")]);
    }
}
