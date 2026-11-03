use std::collections::HashMap;

// Stores the whole document, including its title, comments, and body.
#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    pub comments: Vec<String>,
    pub title: Option<String>,
    pub body: Vec<Node>,
}

// Represents the different things that can appear in the document.
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

// Builds the document from tokens and keeps track of variables.
pub struct AstParser {
    tokens: Vec<String>,
    pos: usize,
    scopes: Vec<HashMap<String, String>>,
}

impl AstParser {
    // Creates a new AST parser.
    pub fn new(tokens: Vec<String>) -> Self {
        Self { tokens, pos: 0, scopes: vec![HashMap::new()] }
    }

    // Gets the current token in uppercase.
    fn current(&self) -> Option<String> {
        self.tokens.get(self.pos).map(|s| s.trim().to_uppercase())
    }

    // Gets the next token in uppercase.
    fn next(&self) -> Option<String> {
        self.tokens.get(self.pos + 1).map(|s| s.trim().to_uppercase())
    }

    // Returns the current token and moves forward.
    fn take(&mut self) -> Result<String, String> {
        let token = self.tokens.get(self.pos).ok_or("unexpected end of file")?.clone();
        self.pos += 1;
        Ok(token)
    }

    // Checks if the token matches what we expect.
    fn expect(&mut self, expected: &str) -> Result<(), String> {
        let token = self.take()?;
        if token.eq_ignore_ascii_case(expected) {
            Ok(())
        } else {
            Err(format!("expected {}, found {}", expected, token))
        }
    }

    // Searches for a variable, starting from the current scope.
    fn lookup(&self, name: &str) -> Option<String> {
        self.scopes.iter().rev().find_map(|scope| scope.get(name).cloned())
    }

    // Reads text until the closing annotation and replaces variables.
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

    // Creates a variable and stores its value in the current scope.
    fn variable_define(&mut self) -> Result<(), String> {
        self.expect("#IHAZ")?;
        let name = self.take()?;
        self.expect("#ITIZ")?;
        let value = self.text_until("#MKAY")?;
        self.scopes.last_mut().unwrap().insert(name, value);
        Ok(())
    }

    // Gets the value of a variable.
    fn variable_use(&mut self) -> Result<String, String> {
        self.expect("#GIMMEH")?;
        self.expect("VAR")?;
        let name = self.take()?;
        self.expect("#OIC")?;
        self.lookup(&name).ok_or_else(|| {
            format!("Static semantic error: variable '{}' is used but never defined in this scope", name)
        })
    }

    // Reads one part of the document and returns its AST node.
    // Variable definitions do not produce anything in the HTML.
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

    // Reads elements until the closing annotation.
    fn elements_until(&mut self, end: &str) -> Result<Vec<Node>, String> {
        let mut nodes = Vec::new();
        while self.current().as_deref() != Some(end) {
            if let Some(node) = self.element()? {
                nodes.push(node);
            }
        }
        Ok(nodes)
    }

    // Creates a paragraph and gives it its own variable scope.
    fn paragraph(&mut self) -> Result<Node, String> {
        self.pos += 2;
        self.scopes.push(HashMap::new());
        let nodes = self.elements_until("#MKAY")?;
        self.expect("#MKAY")?;
        self.scopes.pop();
        Ok(Node::Paragraph(nodes))
    }

    // Reads a list. Each list item gets its own variable scope.
    fn list(&mut self) -> Result<Node, String> {
        self.pos += 2;
        let mut items = Vec::new();
        while self.current().as_deref() == Some("#GIMMEH") {
            self.pos += 2; // Skips #GIMMEH ITEM.
            self.scopes.push(HashMap::new());
            items.push(self.elements_until("#OIC")?);
            self.expect("#OIC")?;
            self.scopes.pop();
        }
        self.expect("#MKAY")?;
        Ok(Node::List(items))
    }

    // Builds the document starting from #HAI.
    pub fn parse_document(&mut self) -> Result<Document, String> {
        self.expect("#HAI")?;

        // Reads comments before the head.
        let mut comments = Vec::new();
        while self.current().as_deref() == Some("#OBTW") {
            self.expect("#OBTW")?;
            comments.push(self.text_until("#TLDR")?);
        }

        // Reads the title if the document has a head.
        let mut title = None;
        if self.current().as_deref() == Some("#MAEK") && self.next().as_deref() == Some("HEAD") {
            self.pos += 2;
            self.expect("#GIMMEH")?;
            self.expect("TITLE")?;
            title = Some(self.text_until("#OIC")?);
            self.expect("#MKAY")?;
        }

        // Reads the body and finishes the document.
        let body = self.elements_until("#KBYE")?;
        self.expect("#KBYE")?;

        Ok(Document { comments, title, body })
    }
}