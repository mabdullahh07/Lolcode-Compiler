//! Translates the AST into HTML text.

use crate::ast::{Document, Node};

/// Turn a whole document into an HTML page.
pub fn to_html(doc: &Document) -> String {
    let mut out = String::from("<html>\n");
    for comment in &doc.comments {
        out.push_str(&format!("<!-- {} -->\n", comment));
    }
    if let Some(title) = &doc.title {
        out.push_str(&format!("<head>\n<title>{}</title>\n</head>\n", title));
    }
    out.push_str("<body>\n");
    out.push_str(&nodes_to_html(&doc.body));
    out.push_str("\n</body>\n</html>\n");
    out
}

/// Turn a list of nodes into HTML, putting spaces between plain words.
fn nodes_to_html(nodes: &[Node]) -> String {
    let parts: Vec<String> = nodes.iter().map(node_to_html).collect();
    parts.join(" ")
}

/// Turn one node into its HTML tag(s).
fn node_to_html(node: &Node) -> String {
    match node {
        Node::Text(text) => text.clone(),
        Node::Comment(text) => format!("<!-- {} -->", text),
        Node::Paragraph(children) => format!("\n<p>{}</p>\n", nodes_to_html(children)),
        Node::Bold(text) => format!("<b>{}</b>", text),
        Node::Link(address) => format!("<a href=\"{0}\">{0}</a>", address),
        Node::Newline => "<br>\n".to_string(),
        Node::List(items) => {
            let mut out = String::from("\n<ul>\n");
            for item in items {
                out.push_str(&format!("<li>{}</li>\n", nodes_to_html(item)));
            }
            out.push_str("</ul>\n");
            out
        }
    }
}
