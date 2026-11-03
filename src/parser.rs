use std::collections::HashMap;

pub trait SyntaxAnalyzer {
    fn parse_lolcode(&mut self) -> Result<(), String>;
    fn parse_head(&mut self) -> Result<(), String>;
    fn parse_title(&mut self) -> Result<(), String>;
    fn parse_comment(&mut self) -> Result<(), String>;
    fn parse_body(&mut self) -> Result<(), String>;
    fn parse_paragraph(&mut self) -> Result<(), String>;
    fn parse_inner_paragraph(&mut self) -> Result<(), String>;
    fn parse_inner_text(&mut self) -> Result<(), String>;
    fn parse_variable_define(&mut self) -> Result<(), String>;
    fn parse_variable_use(&mut self) -> Result<(), String>;
    fn parse_bold(&mut self) -> Result<(), String>;
    fn parse_list(&mut self) -> Result<(), String>;
    fn parse_list_items(&mut self) -> Result<(), String>;
    fn parse_inner_list(&mut self) -> Result<(), String>;
    fn parse_link(&mut self) -> Result<(), String>;
    fn parse_newline(&mut self) -> Result<(), String>;
    fn parse_text(&mut self) -> Result<(), String>;
}

pub struct Parser {
    tokens: Vec<String>,
    position: usize,
    scopes: Vec<HashMap<String, String>>,
    /// Line number of each token (from the lexer), used in error messages.
    lines: Vec<usize>,
}

impl Parser {
    pub fn new(tokens: Vec<String>) -> Self {
        Self {
            tokens,
            position: 0,
            scopes: vec![HashMap::new()],
            lines: Vec::new(),
        }
    }

    /// Create a parser that also knows the source line of every token.
    pub fn with_lines(tokens: Vec<String>, lines: Vec<usize>) -> Self {
        let mut parser = Self::new(tokens);
        parser.lines = lines;
        parser
    }

    /// Source line of the token where parsing stopped (for error messages).
    pub fn error_line(&self) -> usize {
        let index = self.position.saturating_sub(1).min(self.lines.len().saturating_sub(1));
        self.lines.get(index).copied().unwrap_or(0)
    }

    /// Look at the token after the current one, in upper case.
    fn peek_next(&self) -> Option<String> {
        self.tokens.get(self.position + 1).map(|s| s.trim().to_uppercase())
    }


    fn current(&self) -> Option<&str> {
        self.tokens.get(self.position).map(|s| s.trim())
    }

    fn take(&mut self) -> Option<String> {
        let token = self.tokens.get(self.position)?.trim().to_string();
        self.position += 1;
        Some(token)
    }

    fn expect(&mut self, expected: &str) -> Result<(), String> {
        match self.take() {
            Some(token) if token.eq_ignore_ascii_case(expected) => Ok(()),
            found => Err(format!(
                "expected {}, found {}",
                expected, found.unwrap_or_else(|| "end of file".into())
            )),
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    fn define_variable(&mut self, name: String, value: String) {
        self.scopes.last_mut().unwrap().insert(name, value);
    }

    fn variable_exists(&self, name: &str) -> bool {
        self.scopes.iter().rev().any(|scope| scope.contains_key(name))
    }

    fn is_annotation(token: &str) -> bool {
        token.starts_with('#')
    }

    fn parse_variable_name(&mut self) -> Result<String, String> {
        let name = self.take().ok_or("Missing variable name")?;

        if name.is_empty() || name.contains(char::is_whitespace) {
            return Err("Variable name must be one word".into());
        }

        Ok(name)
    }

    fn parse_variable_value(&mut self) -> Result<String, String> {
        let mut value = String::new();

        while let Some(token) = self.current() {
            if token.eq_ignore_ascii_case("#MKAY") {
                break;
            }

            if Self::is_annotation(token) {
                return Err("Unexpected annotation in variable value".into());
            }

            if !value.is_empty() {
                value.push(' ');
            }

            value.push_str(token);
            self.position += 1;
        }

        if value.is_empty() {
            return Err("Variable value cannot be empty".into());
        }

        Ok(value)
    }
}

impl SyntaxAnalyzer for Parser {
    fn parse_lolcode(&mut self) -> Result<(), String> {
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

        self.parse_body()?;
        self.expect("#KBYE")?;

        if self.position != self.tokens.len() {
            return Err("Unexpected tokens after #KBYE".into());
        }

        Ok(())
    }

    fn parse_head(&mut self) -> Result<(), String> {
        self.expect("#MAEK")?;
        self.expect("HEAD")?;
        self.parse_title()?;
        self.expect("#MKAY")
    }

    fn parse_title(&mut self) -> Result<(), String> {
        self.expect("#GIMMEH")?;
        self.expect("TITLE")?;

        let mut found = false;

        while let Some(token) = self.current() {
            if token.eq_ignore_ascii_case("#OIC") {
                break;
            }

            if Self::is_annotation(token) {
                return Err("Unexpected annotation inside title".into());
            }

            found = true;
            self.position += 1;
        }

        if !found {
            return Err("Title cannot be empty".into());
        }

        self.expect("#OIC")
    }

    fn parse_comment(&mut self) -> Result<(), String> {
        self.expect("#OBTW")?;

        while let Some(token) = self.current() {
            if token.eq_ignore_ascii_case("#TLDR") {
                return self.expect("#TLDR");
            }

            if Self::is_annotation(token) {
                return Err("Annotations are not allowed inside comments".into());
            }

            self.position += 1;
        }

        Err("Comment missing #TLDR".into())
    }

    fn parse_body(&mut self) -> Result<(), String> {
        while let Some(token) = self.current() {
            if token.eq_ignore_ascii_case("#KBYE") {
                return Ok(());
            }

            match token.to_uppercase().as_str() {
                "#OBTW" => self.parse_comment()?,
                "#MAEK" => {
                    let next = self.tokens.get(self.position + 1)
                        .map(|s| s.trim().to_uppercase());

                    match next.as_deref() {
                        Some("PARAGRAF") => self.parse_paragraph()?,
                        Some("LIST") => self.parse_list()?,
                        _ => return Err("Invalid #MAEK construct".into()),
                    }
                }
                "#GIMMEH" => {
                    let next = self.tokens.get(self.position + 1)
                        .map(|s| s.trim().to_uppercase());

                    match next.as_deref() {
                        Some("BOLD") => self.parse_bold()?,
                        Some("LINX") => self.parse_link()?,
                        Some("VAR") => self.parse_variable_use()?,
                        _ => return Err("Invalid #GIMMEH construct".into()),
                    }
                }
                "#IHAZ" => self.parse_variable_define()?,
                "#NEWLINE" => self.parse_newline()?,
                _ if Self::is_annotation(token) =>
                    return Err(format!("Unexpected annotation {}", token)),
                _ => self.parse_text()?,
            }
        }

        Err("Missing #KBYE".into())
    }

    fn parse_paragraph(&mut self) -> Result<(), String> {
        self.expect("#MAEK")?;
        self.expect("PARAGRAF")?;
        self.push_scope();

        self.parse_inner_paragraph()?;

        self.expect("#MKAY")?;
        self.pop_scope();
        Ok(())
    }

    fn parse_inner_paragraph(&mut self) -> Result<(), String> {
        while let Some(token) = self.current() {
            if token.eq_ignore_ascii_case("#MKAY") {
                return Ok(());
            }

            match token.to_uppercase().as_str() {
                "#OBTW" => self.parse_comment()?,
                "#IHAZ" => self.parse_variable_define()?,
                "#GIMMEH" => {
                    let next = self.tokens.get(self.position + 1)
                        .map(|s| s.trim().to_uppercase());

                    match next.as_deref() {
                        Some("VAR") => self.parse_variable_use()?,
                        Some("BOLD") => self.parse_bold()?,
                        Some("LINX") => self.parse_link()?,
                        _ => return Err("Invalid annotation inside paragraph".into()),
                    }
                }
                "#MAEK" => {
                    let next = self.tokens.get(self.position + 1)
                        .map(|s| s.trim().to_uppercase());

                    if next.as_deref() == Some("LIST") {
                        self.parse_list()?;
                    } else {
                        return Err("Nested paragraph is not allowed".into());
                    }
                }
                "#NEWLINE" => self.parse_newline()?,
                _ if Self::is_annotation(token) =>
                    return Err(format!("Unexpected annotation {}", token)),
                _ => self.parse_inner_text()?,
            }
        }

        Err("Paragraph missing #MKAY".into())
    }

    fn parse_inner_text(&mut self) -> Result<(), String> {
        self.parse_text()
    }

    fn parse_variable_define(&mut self) -> Result<(), String> {
        self.expect("#IHAZ")?;
        let name = self.parse_variable_name()?;
        self.expect("#ITIZ")?;
        let value = self.parse_variable_value()?;
        self.expect("#MKAY")?;

        self.define_variable(name, value);
        Ok(())
    }

    fn parse_variable_use(&mut self) -> Result<(), String> {
        self.expect("#GIMMEH")?;
        self.expect("VAR")?;
        let name = self.parse_variable_name()?;
        self.expect("#OIC")?;

        if !self.variable_exists(&name) {
            return Err(format!(
                "Static semantic error: variable '{}' is used but never defined in this scope",
                name
            ));
        }

        Ok(())
    }

    fn parse_bold(&mut self) -> Result<(), String> {
        self.expect("#GIMMEH")?;
        self.expect("BOLD")?;

        let mut found = false;

        while let Some(token) = self.current() {
            if token.eq_ignore_ascii_case("#OIC") {
                break;
            }

            if token.eq_ignore_ascii_case("#GIMMEH") && self.peek_next().as_deref() == Some("VAR") {
                self.parse_variable_use()?;
                found = true;
                continue;
            }

            if Self::is_annotation(token) {
                return Err(format!("unexpected annotation {} inside bold text", token));
            }

            found = true;
            self.position += 1;
        }

        if !found {
            return Err("bold text cannot be empty".into());
        }

        self.expect("#OIC")
    }

    fn parse_list(&mut self) -> Result<(), String> {
        self.expect("#MAEK")?;
        self.expect("LIST")?;
        self.push_scope();

        self.parse_list_items()?;

        self.expect("#MKAY")?;
        self.pop_scope();
        Ok(())
    }

    fn parse_list_items(&mut self) -> Result<(), String> {
        if self.current() != Some("#GIMMEH") {
            return Err("List must contain at least one #GIMMEH ITEM".into());
        }

        while self.current() == Some("#GIMMEH") {
            self.parse_inner_list()?;
        }

        Ok(())
    }

    fn parse_inner_list(&mut self) -> Result<(), String> {
        self.expect("#GIMMEH")?;
        self.expect("ITEM")?;
        self.push_scope();

        while let Some(token) = self.current() {
            if token.eq_ignore_ascii_case("#OIC") {
                self.expect("#OIC")?;
                self.pop_scope();
                return Ok(());
            }

            match token.to_uppercase().as_str() {
                "#IHAZ" => self.parse_variable_define()?,
                "#OBTW" => self.parse_comment()?,
                "#NEWLINE" => self.parse_newline()?,
                "#GIMMEH" => {
                    let next = self.tokens.get(self.position + 1)
                        .map(|s| s.trim().to_uppercase());

                    match next.as_deref() {
                        Some("BOLD") => self.parse_bold()?,
                        Some("VAR") => self.parse_variable_use()?,
                        Some("LINX") => self.parse_link()?,
                        _ => return Err("Invalid annotation inside list item".into()),
                    }
                }
                _ if Self::is_annotation(token) =>
                    return Err(format!("Unexpected annotation {}", token)),
                _ => self.parse_text()?,
            }
        }

        Err("List item missing #OIC".into())
    }

    fn parse_link(&mut self) -> Result<(), String> {
        self.expect("#GIMMEH")?;
        self.expect("LINX")?;

        let address = self.take().ok_or("Missing link address")?;

        if address.is_empty() || address.contains(char::is_whitespace) {
            return Err("Link address must be one word".into());
        }

        self.expect("#OIC")
    }

    fn parse_newline(&mut self) -> Result<(), String> {
        self.expect("#NEWLINE")
    }

    fn parse_text(&mut self) -> Result<(), String> {
        if let Some(token) = self.current() {
            if Self::is_annotation(token) {
                return Err(format!("Expected text, found annotation {}", token));
            }

            self.position += 1;
            Ok(())
        } else {
            Err("Expected text, found end of input".into())
        }
    }
}


#[cfg(test)]
mod tests {
    use super::{Parser, SyntaxAnalyzer};

    fn parse_tokens(tokens: Vec<&str>) -> Result<(), String> {
        let tokens = tokens.into_iter().map(String::from).collect();
        Parser::new(tokens).parse_lolcode()
    }

    fn parse_source(source: &str) -> Result<(), String> {
        let mut lexer = crate::lexer::Lexer::new(source);
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        parser.parse_lolcode()
    }

    #[test]
    fn valid_document() {
        assert!(parse_tokens(vec![
            "#HAI", "#MAEK", "HEAD", "#GIMMEH", "TITLE",
            "My", "Document", "#OIC", "#MKAY",
            "Hello", "world!", "#KBYE"
        ]).is_ok());
    }

    #[test]
    fn valid_head_and_body() {
        assert!(parse_source(
            "#HAI #MAEK HEAD #GIMMEH TITLE My Document #OIC #MKAY Hello world! #KBYE"
        ).is_ok());
    }

    #[test]
    fn missing_hai() {
        assert!(parse_source("Hello #KBYE").is_err());
    }

    #[test]
    fn missing_kbye() {
        assert!(parse_tokens(vec!["#HAI", "Hello", "world"]).is_err());
    }

    #[test]
    fn missing_title_oic() {
        assert!(parse_tokens(vec![
            "#HAI", "#MAEK", "HEAD", "#GIMMEH",
            "TITLE", "Hello", "#MKAY", "#KBYE"
        ]).is_err());
    }

    #[test]
    fn invalid_annotation() {
        assert!(parse_source("#HAI #INVALID #KBYE").is_err());
    }

    #[test]
    fn undefined_variable() {
        assert!(parse_tokens(vec![
            "#HAI", "#GIMMEH", "VAR", "missing", "#OIC", "#KBYE"
        ]).is_err());
    }

    #[test]
    fn variable_definition_and_use() {
        assert!(parse_tokens(vec![
            "#HAI",
            "#IHAZ", "name", "#ITIZ", "Abdullah", "#MKAY",
            "#GIMMEH", "VAR", "name", "#OIC",
            "#KBYE"
        ]).is_ok());
    }
}