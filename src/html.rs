use crate::ast::{Document, Node};

// Converts the document into an HTML page.
pub fn to_html(doc: &Document) -> String {
    let mut out = String::from("<html>\n");

    // Adds all comments to the HTML.
    for comment in &doc.comments {
        out.push_str(&format!("<!-- {} -->\n", comment));
    }

    // Adds the title if one exists.
    if let Some(title) = &doc.title {
        out.push_str(&format!(
            "<head>\n<title>{}</title>\n</head>\n",
            title
        ));
    }

    // Adds the body and its contents.
    out.push_str("<body>\n");
    out.push_str(&nodes_to_html(&doc.body));
    out.push_str("\n</body>\n</html>\n");

    out
}

// Converts a list of AST nodes into HTML.
fn nodes_to_html(nodes: &[Node]) -> String {
    let mut out = String::new();

    for (i, node) in nodes.iter().enumerate() {
        // Adds a space between inline elements.
        if i > 0 {
            let previous = &nodes[i - 1];

            let previous_inline = matches!(
                previous,
                Node::Text(_) | Node::Bold(_) | Node::Link(_)
            );

            let current_inline = matches!(
                node,
                Node::Text(_) | Node::Bold(_) | Node::Link(_)
            );

            if previous_inline && current_inline {
                out.push(' ');
            }
        }

        out.push_str(&node_to_html(node));
    }

    out
}

// Converts one AST node into HTML.
fn node_to_html(node: &Node) -> String {
    match node {
        // Normal text is added as it is.
        Node::Text(text) => text.clone(),

        // Comments are written as HTML comments.
        Node::Comment(text) => format!("<!-- {} -->", text),

        // Paragraphs are wrapped in p tags.
        Node::Paragraph(children) => {
            format!("\n<p>{}</p>\n", nodes_to_html(children))
        }

        // Bold text is wrapped in b tags.
        Node::Bold(text) => format!("<b>{}</b>", text),

        // Creates a clickable link.
        Node::Link(address) => {
            let url = if address.starts_with("http://")
                || address.starts_with("https://")
            {
                address.clone()
            } else {
                format!("https://{}", address)
            };

            format!("<a href=\"{}\">{}</a>", url, address)
        }

        // Adds a line break.
        Node::Newline => "<br>\n".to_string(),

        // Creates an unordered list and adds each item.
        Node::List(items) => {
            let mut out = String::from("\n<ul>\n");

            for item in items {
                out.push_str(&format!(
                    "<li>{}</li>\n",
                    nodes_to_html(item)
                ));
            }

            out.push_str("</ul>\n");
            out
        }
    }
}