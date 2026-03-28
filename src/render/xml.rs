use std::borrow::Cow;

#[derive(Clone, Debug)]
pub(super) struct Node {
    kind: NodeKind,
    attributes: Vec<(Cow<'static, str>, String)>,
    children: Option<Vec<Node>>,
    text: Option<String>,
}

#[derive(Clone, Debug)]
enum NodeKind {
    Element(&'static str),
    Fragment,
    Text,
}

impl Node {
    pub(super) fn render(&self) -> String {
        match self.kind {
            NodeKind::Text => escape_xml(self.text.as_deref().unwrap_or_default()),
            NodeKind::Fragment => self
                .children
                .as_ref()
                .map(|children| children.iter().map(Node::render).collect())
                .unwrap_or_default(),
            NodeKind::Element(name) => {
                let mut output = String::new();
                output.push('<');
                output.push_str(name);
                for (key, value) in &self.attributes {
                    output.push(' ');
                    output.push_str(key);
                    output.push_str("=\"");
                    output.push_str(&escape_xml(value));
                    output.push('"');
                }
                let has_children = self
                    .children
                    .as_ref()
                    .map(|children| !children.is_empty())
                    .unwrap_or(false);
                let rendered_children = self
                    .children
                    .as_ref()
                    .map(|children| children.iter().map(Node::render).collect::<String>())
                    .unwrap_or_default();
                if !has_children {
                    output.push_str("/>");
                } else {
                    output.push('>');
                    output.push_str(&rendered_children);
                    output.push_str("</");
                    output.push_str(name);
                    output.push('>');
                }
                output
            }
        }
    }
}

pub(super) fn element(
    name: &'static str,
    attributes: Vec<(Cow<'static, str>, String)>,
    children: Vec<Node>,
) -> Node {
    Node {
        kind: NodeKind::Element(name),
        attributes,
        children: Some(children),
        text: None,
    }
}

pub(super) fn fragment(children: Vec<Node>) -> Node {
    Node {
        kind: NodeKind::Fragment,
        attributes: Vec::new(),
        children: Some(children),
        text: None,
    }
}

pub(super) fn text(value: impl Into<String>) -> Node {
    Node {
        kind: NodeKind::Text,
        attributes: Vec::new(),
        children: None,
        text: Some(value.into()),
    }
}

pub(super) fn attr(
    name: impl Into<Cow<'static, str>>,
    value: impl ToString,
) -> (Cow<'static, str>, String) {
    (name.into(), value.to_string())
}

fn escape_xml(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&apos;"),
            _ => escaped.push(ch),
        }
    }
    escaped
}
