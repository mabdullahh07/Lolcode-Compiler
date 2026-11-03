use std::collections::HashSet;

pub struct Parser {
    tokens: Vec<String>,
    position: usize,
    variables: HashSet<String>,
}

impl Parser {
    pub fn new(tokens: Vec<String>) -> Self {
        Self {
            tokens,
            position: 0,
            variables: HashSet::new(),
        }
    }

    fn current(&self) -> Option<&str> {
        self.tokens.get(self.position).map(String::as_str)
    }

    fn take(&mut self) -> Option<String> {
        let token = self.tokens.get(self.position)?.clone();
        self.position += 1;
        Some(token)
    }

    fn expect(&mut self, expected: &str) -> Result<(), String> {
        match self.take() {
            Some(token) if token.eq_ignore_ascii_case(expected) => Ok(()),
            found => Err(format!("Expected {}, found {:?}", expected, found)),
        }
    }

    fn is_annotation(token: &str) -> bool {
        token.starts_with('#')
    }

    fn parse_comment(&mut self) -> Result<(), String> {
        self.expect("#OBTW")?;
        while let Some(token) = self.current() {
            if token == "#TLDR" {
                return self.expect("#TLDR");
            }
            self.position += 1;
        }
        Err("Comment missing #TLDR".into())
    }

    fn parse_title(&mut self) -> Result<(), String> {
        self.expect("#GIMMEH")?;
        self.expect("TITLE")?;

        while let Some(token) = self.current() {
            if token == "#OIC" {
                return self.expect("#OIC");
            }
            self.position += 1;
        }

        Err("Title missing #OIC".into())
    }

    fn parse_head(&mut self) -> Result<(), String> {
        self.expect("#MAEK")?;

        let head = self.take().ok_or("Missing HEAD")?;

        if !head.trim().eq_ignore_ascii_case("HEAD") {
            return Err("Expected HEAD after #MAEK".into());
        }

        self.expect("#GIMMEH")?;

        let title = self.take().ok_or("Missing TITLE")?;

        if !title.trim().to_uppercase().starts_with("TITLE") {
            return Err("Expected TITLE after #GIMMEH".into());
        }

        self.expect("#OIC")?;

        // Skip whitespace tokens before #MKAY
        while let Some(token) = self.current() {
            if token.trim().is_empty() {
                self.position += 1;
            } else {
                break;
            }
        }

        self.expect("#MKAY")?;

        Ok(())
    }

    fn parse_variable_define(&mut self) -> Result<(), String> {
        self.expect("#IHAZ")?;
        let name = self.take().ok_or("Missing variable name")?;
        self.expect("#ITIZ")?;

        if self.current().is_none() {
            return Err("Missing variable value".into());
        }
        self.position += 1;

        self.expect("#MKAY")?;
        self.variables.insert(name);
        Ok(())
    }

    fn parse_variable_use(&mut self) -> Result<(), String> {
        self.expect("#GIMMEH")?;
        self.expect("VAR")?;
        let name = self.take().ok_or("Missing variable name")?;
        self.expect("#OIC")?;

        if !self.variables.contains(&name) {
            return Err(format!("Undefined variable: {}", name));
        }
        Ok(())
    }

    fn parse_bold(&mut self) -> Result<(), String> {
        self.expect("#GIMMEH")?;
        self.expect("BOLD")?;

        while let Some(token) = self.current() {
            if token == "#OIC" {
                return self.expect("#OIC");
            }
            self.position += 1;
        }

        Err("Bold missing #OIC".into())
    }

    fn parse_link(&mut self) -> Result<(), String> {
        self.expect("#GIMMEH")?;
        self.expect("LINX")?;
        self.take().ok_or("Missing link address")?;
        self.expect("#OIC")
    }

    fn parse_list(&mut self) -> Result<(), String> {
        self.expect("#MAEK")?;
        self.expect("LIST")?;

        while let Some(token) = self.current() {
            if token == "#MKAY" {
                return self.expect("#MKAY");
            }

            if token == "#GIMMEH" {
                self.expect("#GIMMEH")?;
                self.expect("ITEM")?;

                while let Some(token) = self.current() {
                    if token == "#OIC" {
                        self.expect("#OIC")?;
                        break;
                    }
                    self.position += 1;
                }
            } else {
                self.position += 1;
            }
        }

        Err("List missing #MKAY".into())
    }

    fn parse_paragraph(&mut self) -> Result<(), String> {
        self.expect("#MAEK")?;
        self.expect("PARAGRAF")?;

        while let Some(token) = self.current() {
            if token == "#MKAY" {
                return self.expect("#MKAY");
            }

            if token == "#OBTW" {
                self.parse_comment()?;
            } else if token == "#GIMMEH" {
                match self.tokens.get(self.position + 1).map(String::as_str) {
                    Some("VAR") => self.parse_variable_use()?,
                    Some("BOLD") => self.parse_bold()?,
                    Some("LINX") => self.parse_link()?,
                    _ => return Err("Invalid GIMMEH inside paragraph".into()),
                }
            } else {
                self.position += 1;
            }
        }

        Err("Paragraph missing #MKAY".into())
    }

    fn parse_body(&mut self) -> Result<(), String> {
        while let Some(token) = self.current() {
            if token == "#KBYE" {
                return Ok(());
            }

            match token {
                "#OBTW" => self.parse_comment()?,
                "#MAEK" => {
                    let next = self.tokens
                        .get(self.position + 1)
                        .map(|s| s.trim().to_uppercase());

                    match next.as_deref() {
                        Some("PARAGRAF") => self.parse_paragraph()?,
                        Some("LIST") => self.parse_list()?,
                        Some("HEAD") => self.parse_head()?,
                        _ => return Err(format!(
                            "Unknown #MAEK construct. Next token: {:?}",
                            next
                        )),
                    }
                },
                "#GIMMEH" => match self.tokens.get(self.position + 1).map(String::as_str) {
                    Some("VAR") => self.parse_variable_use()?,
                    Some("BOLD") => self.parse_bold()?,
                    Some("LINX") => self.parse_link()?,
                    _ => return Err("Unknown #GIMMEH construct".into()),
                },
                "#IHAZ" => self.parse_variable_define()?,
                "#NEWLINE" => { self.position += 1; }
                other if Self::is_annotation(other) =>
                    return Err(format!("Unexpected annotation {}", other)),
                _ => { self.position += 1; }
            }
        }

        Err("Missing #KBYE".into())
    }

    pub fn parse_lolcode(&mut self) -> Result<(), String> {
        self.expect("#HAI")?;

        while self.current() == Some("#OBTW") {
            self.parse_comment()?;
        }

        if self.current() == Some("#MAEK")
            && self.tokens.get(self.position + 1)
                .map(|s| s.trim().eq_ignore_ascii_case("HEAD"))
                .unwrap_or(false)
        {
            self.parse_head()?;
        }

        if self.current() == Some("#IHAZ") {
            self.parse_variable_define()?;
        }

        self.parse_body()?;
        self.expect("#KBYE")?;

        if self.position != self.tokens.len() {
            return Err("Unexpected tokens after #KBYE".into());
        }

        Ok(())
    }
}
