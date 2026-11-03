//! Abstract syntax tree for LOLCODE Markdown.
//!
//! After the `Parser` has checked the syntax, `AstParser` walks the same
//! tokens again, builds the tree, and replaces every `#GIMMEH VAR` with the
//! variable's value using static scoping (a stack of scopes).

use std::collections::HashMap;

/// A whole LOLCODE document: optional comments before the head, an optional
/// title, and the body.
#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    pub comments: Vec<String>,
    pub title: Option<String>,
    pub body: Vec<Node>,
}

/// One element of the body.
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Text(String),
    Comment(String),
    Paragraph(Vec<Node>),
    Bold(String),
    Link(String),
    List(Vec<Vec<Node>>),
    Newline,
}

/// Builds the `Document` from a token list. Holds a stack of scopes,
/// innermost scope last.
pub struct AstParser {
    tokens: Vec<String>,
    pos: usize,
    scopes: Vec<HashMap<String, String>>,
}

impl AstParser {
    /// Create a builder over the tokens produced by the lexer.
    pub fn new(tokens: Vec<String>) -> Self {
        Self { tokens, pos: 0, scopes: vec![HashMap::new()] }
    }

    fn current(&self) -> Option<String> {
        self.tokens.get(self.pos).map(|s| s.trim().to_uppercase())
    }

    fn next(&self) -> Option<String> {
        self.tokens.get(self.pos + 1).map(|s| s.trim().to_uppercase())
    }

    fn take(&mut self) -> Result<String, String> {
        let token = self.tokens.get(self.pos).ok_or("unexpected end of file")?.clone();
        self.pos += 1;
        Ok(token)
    }

    fn expect(&mut self, expected: &str) -> Result<(), String> {
        let token = self.take()?;
        if token.eq_ignore_ascii_case(expected) {
            Ok(())
        } else {
            Err(format!("expected {}, found {}", expected, token))
        }
    }

    /// Static scoping: search from the innermost scope outward.
    fn lookup(&self, name: &str) -> Option<String> {
        self.scopes.iter().rev().find_map(|scope| scope.get(name).cloned())
    }

    /// Collect plain-text words until the given closing annotation,
    /// resolving any variable uses along the way.
    fn text_until(&mut self, end: &str) -> Result<String, String> {
        let mut words = Vec::new();
        while self.current().as_deref() != Some(end) {
            if self.current().as_deref() == Some("#GIMMEH") && self.next().as_deref() == Some("VAR") {
                words.push(self.variable_use()?);
            } else {
                words.push(self.take()?);
            }
        }
        self.expect(end)?;
        Ok(words.join(" "))
    }

    /// `#IHAZ name #ITIZ value #MKAY` — store in the innermost scope.
    fn variable_define(&mut self) -> Result<(), String> {
        self.expect("#IHAZ")?;
        let name = self.take()?;
        self.expect("#ITIZ")?;
        let value = self.text_until("#MKAY")?;
        self.scopes.last_mut().unwrap().insert(name, value);
        Ok(())
    }

    /// `#GIMMEH VAR name #OIC` — return the variable's value.
    fn variable_use(&mut self) -> Result<String, String> {
        self.expect("#GIMMEH")?;
        self.expect("VAR")?;
        let name = self.take()?;
        self.expect("#OIC")?;
        self.lookup(&name).ok_or_else(|| {
            format!("Static semantic error: variable '{}' is used but never defined in this scope", name)
        })
    }

    /// Parse one element that can appear in the body, a paragraph or a list
    /// item. Returns `None` for a variable definition (no output).
    fn element(&mut self) -> Result<Option<Node>, String> {
        let token = self.current().ok_or("unexpected end of file")?;
        let node = match token.as_str() {
            "#IHAZ" => {
                self.variable_define()?;
                return Ok(None);
            }
            "#OBTW" => {
                self.expect("#OBTW")?;
                Node::Comment(self.text_until("#TLDR")?)
            }
            "#NEWLINE" => {
                self.expect("#NEWLINE")?;
                Node::Newline
            }
            "#GIMMEH" => match self.next().as_deref() {
                Some("VAR") => Node::Text(self.variable_use()?),
                Some("BOLD") => {
                    self.pos += 2;
                    Node::Bold(self.text_until("#OIC")?)
                }
                Some("LINX") => {
                    self.pos += 2;
                    let address = self.take()?;
                    self.expect("#OIC")?;
                    Node::Link(address)
                }
                _ => return Err(format!("unexpected #GIMMEH {:?}", self.next())),
            },
            "#MAEK" => match self.next().as_deref() {
                Some("PARAGRAF") => self.paragraph()?,
                Some("LIST") => self.list()?,
                _ => return Err(format!("unexpected #MAEK {:?}", self.next())),
            },
            _ => Node::Text(self.take()?),
        };
        Ok(Some(node))
    }

    /// Parse elements until the closing annotation `end` (not consumed).
    fn elements_until(&mut self, end: &str) -> Result<Vec<Node>, String> {
        let mut nodes = Vec::new();
        while self.current().as_deref() != Some(end) {
            if let Some(node) = self.element()? {
                nodes.push(node);
            }
        }
        Ok(nodes)
    }

    /// `#MAEK PARAGRAF ... #MKAY` — opens a new scope.
    fn paragraph(&mut self) -> Result<Node, String> {
        self.pos += 2;
        self.scopes.push(HashMap::new());
        let nodes = self.elements_until("#MKAY")?;
        self.expect("#MKAY")?;
        self.scopes.pop();
        Ok(Node::Paragraph(nodes))
    }

    /// `#MAEK LIST (#GIMMEH ITEM ... #OIC)+ #MKAY` — each item is a scope.
    fn list(&mut self) -> Result<Node, String> {
        self.pos += 2;
        let mut items = Vec::new();
        while self.current().as_deref() == Some("#GIMMEH") {
            self.pos += 2; // #GIMMEH ITEM
            self.scopes.push(HashMap::new());
            items.push(self.elements_until("#OIC")?);
            self.expect("#OIC")?;
            self.scopes.pop();
        }
        self.expect("#MKAY")?;
        Ok(Node::List(items))
    }

    /// Build the whole document starting at `#HAI`.
    pub fn parse_document(&mut self) -> Result<Document, String> {
        self.expect("#HAI")?;

        let mut comments = Vec::new();
        while self.current().as_deref() == Some("#OBTW") {
            self.expect("#OBTW")?;
            comments.push(self.text_until("#TLDR")?);
        }

        let mut title = None;
        if self.current().as_deref() == Some("#MAEK") && self.next().as_deref() == Some("HEAD") {
            self.pos += 2;
            self.expect("#GIMMEH")?;
            self.expect("TITLE")?;
            title = Some(self.text_until("#OIC")?);
            self.expect("#MKAY")?;
        }

        let body = self.elements_until("#KBYE")?;
        self.expect("#KBYE")?;
        Ok(Document { comments, title, body })
    }
}
