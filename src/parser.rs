use std::collections::HashMap;

// Contains the functions used to check the syntax of the program.
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

// Stores the tokens and keeps track of where the parser is.
pub struct Parser {
    tokens: Vec<String>,
    position: usize,
    scopes: Vec<HashMap<String, String>>,

    // Keeps track of the line number of each token.
    lines: Vec<usize>,
}

impl Parser {
    // Creates a new parser.
    pub fn new(tokens: Vec<String>) -> Self {
        Self {
            tokens,
            position: 0,
            scopes: vec![HashMap::new()],
            lines: Vec::new(),
        }
    }

    // Creates a parser that also knows the line numbers.
    pub fn with_lines(tokens: Vec<String>, lines: Vec<usize>) -> Self {
        let mut parser = Self::new(tokens);
        parser.lines = lines;
        parser
    }

    // Gets the line number where the parser stopped.
    pub fn error_line(&self) -> usize {
        let index = self.position.saturating_sub(1).min(self.lines.len().saturating_sub(1));
        self.lines.get(index).copied().unwrap_or(0)
    }

    // Looks at the token after the current one.
    fn peek_next(&self) -> Option<String> {
        self.tokens.get(self.position + 1).map(|s| s.trim().to_uppercase())
    }

    // Gets the current token.
    fn current(&self) -> Option<&str> {
        self.tokens.get(self.position).map(|s| s.trim())
    }

    // Gets the current token and moves forward.
    fn take(&mut self) -> Option<String> {
        let token = self.tokens.get(self.position)?.trim().to_string();
        self.position += 1;
        Some(token)
    }

    // Checks if the current token is the expected one.
    fn expect(&mut self, expected: &str) -> Result<(), String> {
        match self.take() {
            Some(token) if token.eq_ignore_ascii_case(expected) => Ok(()),
            found => Err(format!(
                "expected {}, found {}",
                expected,
                found.unwrap_or_else(|| "end of file".into())
            )),
        }
    }

    // Starts a new scope for variables.
    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    // Removes the current scope.
    fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    // Saves a variable in the current scope.
    fn define_variable(&mut self, name: String, value: String) {
        self.scopes.last_mut().unwrap().insert(name, value);
    }

    // Checks if a variable exists in any scope.
    fn variable_exists(&self, name: &str) -> bool {
        self.scopes.iter().rev().any(|scope| scope.contains_key(name))
    }

    // Checks if a token is an annotation.
    fn is_annotation(token: &str) -> bool {
        token.starts_with('#')
    }

    // Reads and checks a variable name.
    fn parse_variable_name(&mut self) -> Result<String, String> {
        let name = self.take().ok_or("Missing variable name")?;

        if name.is_empty() || name.contains(char::is_whitespace) {
            return Err("Variable name must be one word".into());
        }

        Ok(name)
    }

    // Reads the value assigned to a variable.
    fn parse_variable_value(&mut self) -> Result<String, String> {
        let mut value = String::new();

        while let Some(token) = self.current() {
            if token.eq_ignore_ascii_case("#MKAY") {
                break;
            }

            // Annotations are not allowed inside a variable value.
            if Self::is_annotation(token) {
                return Err("Unexpected annotation in variable value".into());
            }

            // Adds spaces between the words in the value.
            if !value.is_empty() {
                value.push(' ');
            }

            value.push_str(token);
            self.position += 1;
        }

        // Makes sure the variable has a value.
        if value.is_empty() {
            return Err("Variable value cannot be empty".into());
        }

        Ok(value)
    }
}

// Implements all the syntax checking functions.
impl SyntaxAnalyzer for Parser {
    fn parse_lolcode(&mut self) -> Result<(), String> {
        // Every program must start with #HAI.
        self.expect("#HAI")?;

        // Reads comments before the head.
        while self.current() == Some("#OBTW") {
            self.parse_comment()?;
        }

        // Reads the head if it is present.
        if self.current() == Some("#MAEK")
            && self.tokens.get(self.position + 1)
                .map(|s| s.trim().eq_ignore_ascii_case("HEAD"))
                .unwrap_or(false)
        {
            self.parse_head()?;
        }

        // Reads the body and checks the ending.
        self.parse_body()?;
        self.expect("#KBYE")?;

        // Makes sure there are no extra tokens after #KBYE.
        if self.position != self.tokens.len() {
            return Err("Unexpected tokens after #KBYE".into());
        }

        Ok(())
    }

    // Checks the head section.
    fn parse_head(&mut self) -> Result<(), String> {
        self.expect("#MAEK")?;
        self.expect("HEAD")?;
        self.parse_title()?;
        self.expect("#MKAY")
    }

    // Checks the title inside the head.
    fn parse_title(&mut self) -> Result<(), String> {
        self.expect("#GIMMEH")?;
        self.expect("TITLE")?;

        let mut found = false;

        while let Some(token) = self.current() {
            if token.eq_ignore_ascii_case("#OIC") {
                break;
            }

            // Annotations cannot appear inside the title.
            if Self::is_annotation(token) {
                return Err("Unexpected annotation inside title".into());
            }

            found = true;
            self.position += 1;
        }

        // The title must contain some text.
        if !found {
            return Err("Title cannot be empty".into());
        }

        self.expect("#OIC")
    }

    // Checks a comment.
    fn parse_comment(&mut self) -> Result<(), String> {
        self.expect("#OBTW")?;

        while let Some(token) = self.current() {
            if token.eq_ignore_ascii_case("#TLDR") {
                return self.expect("#TLDR");
            }

            // Annotations are not allowed inside comments.
            if Self::is_annotation(token) {
                return Err("Annotations are not allowed inside comments".into());
            }

            self.position += 1;
        }

        Err("Comment missing #TLDR".into())
    }

    // Checks everything inside the body.
    fn parse_body(&mut self) -> Result<(), String> {
        while let Some(token) = self.current() {
            // Stops reading the body when #KBYE is found.
            if token.eq_ignore_ascii_case("#KBYE") {
                return Ok(());
            }

            match token.to_uppercase().as_str() {
                "#OBTW" => self.parse_comment()?,
                "#MAEK" => {
                    // Checks what comes after #MAEK.
                    let next = self.tokens.get(self.position + 1)
                        .map(|s| s.trim().to_uppercase());

                    match next.as_deref() {
                        Some("PARAGRAF") => self.parse_paragraph()?,
                        Some("LIST") => self.parse_list()?,
                        _ => return Err("Invalid #MAEK construct".into()),
                    }
                }
                "#GIMMEH" => {
                    // Checks what comes after #GIMMEH.
                    let next = self.tokens.get(self.position + 1)
                        .map(|s| s.trim().to_uppercase());

                    match next.as_deref() {
                        Some("BOLD") => self.parse_bold()?,
                        Some("LINX") => self.parse_link()?,
                        Some("VAR") => self.parse_variable_use()?,
                        Some("ITEM") => return Err("#GIMMEH ITEM is only allowed inside #MAEK LIST".into()),
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

    // Checks a paragraph and creates its own variable scope.
    fn parse_paragraph(&mut self) -> Result<(), String> {
        self.expect("#MAEK")?;
        self.expect("PARAGRAF")?;
        self.push_scope();

        self.parse_inner_paragraph()?;

        self.expect("#MKAY")?;
        self.pop_scope();
        Ok(())
    }

    // Checks everything inside a paragraph.
    fn parse_inner_paragraph(&mut self) -> Result<(), String> {
        while let Some(token) = self.current() {
            // Stops when the paragraph ends.
            if token.eq_ignore_ascii_case("#MKAY") {
                return Ok(());
            }

            match token.to_uppercase().as_str() {
                "#OBTW" => self.parse_comment()?,
                "#IHAZ" => self.parse_variable_define()?,
                "#GIMMEH" => {
                    // Checks which annotation comes after #GIMMEH.
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
                    // Checks if a list or another paragraph is starting.
                    let next = self.tokens.get(self.position + 1)
                        .map(|s| s.trim().to_uppercase());

                    match next.as_deref() {
                        Some("LIST") => self.parse_list()?,
                        Some("PARAGRAF") => self.parse_paragraph()?,
                        _ => return Err("Invalid #MAEK construct inside paragraph".into()),
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

    // Reads normal text inside a paragraph.
    fn parse_inner_text(&mut self) -> Result<(), String> {
        self.parse_text()
    }

    // Reads a variable definition and saves it.
    fn parse_variable_define(&mut self) -> Result<(), String> {
        self.expect("#IHAZ")?;
        let name = self.parse_variable_name()?;
        self.expect("#ITIZ")?;
        let value = self.parse_variable_value()?;
        self.expect("#MKAY")?;

        self.define_variable(name, value);
        Ok(())
    }

    // Checks if a variable has been defined before it is used.
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

    // Checks the text inside a bold annotation.
    fn parse_bold(&mut self) -> Result<(), String> {
        self.expect("#GIMMEH")?;
        self.expect("BOLD")?;

        let mut found = false;

        while let Some(token) = self.current() {
            // Stops when the bold annotation ends.
            if token.eq_ignore_ascii_case("#OIC") {
                break;
            }

            // Allows variables inside bold text.
            if token.eq_ignore_ascii_case("#GIMMEH")
                && self.peek_next().as_deref() == Some("VAR")
            {
                self.parse_variable_use()?;
                found = true;
                continue;
            }

            // Other annotations are not allowed inside bold.
            if Self::is_annotation(token) {
                return Err(format!(
                    "unexpected annotation {} inside bold text",
                    token
                ));
            }

            found = true;
            self.position += 1;
        }

        // Bold text cannot be empty.
        if !found {
            return Err("bold text cannot be empty".into());
        }

        self.expect("#OIC")
    }

    // Checks a list and creates its variable scope.
    fn parse_list(&mut self) -> Result<(), String> {
        self.expect("#MAEK")?;
        self.expect("LIST")?;
        self.push_scope();

        self.parse_list_items()?;

        self.expect("#MKAY")?;
        self.pop_scope();
        Ok(())
    }

    // Checks all the items inside a list.
    fn parse_list_items(&mut self) -> Result<(), String> {
        // A list must have at least one item.
        if self.current() != Some("#GIMMEH") {
            return Err("List must contain at least one #GIMMEH ITEM".into());
        }

        // Reads each list item.
        while self.current() == Some("#GIMMEH") {
            self.parse_inner_list()?;
        }

        Ok(())
    }

    // Checks one list item and creates its own scope.
    fn parse_inner_list(&mut self) -> Result<(), String> {
        self.expect("#GIMMEH")?;
        self.expect("ITEM")?;
        self.push_scope();

        while let Some(token) = self.current() {
            // Ends the list item.
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
                    // Checks what comes after #GIMMEH.
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

    // Checks a link and its address.
    fn parse_link(&mut self) -> Result<(), String> {
        self.expect("#GIMMEH")?;
        self.expect("LINX")?;

        let address = self.take().ok_or("Missing link address")?;

        // The link address must be a single word.
        if address.is_empty() || address.contains(char::is_whitespace) {
            return Err("Link address must be one word".into());
        }

        self.expect("#OIC")
    }

    // Checks the newline annotation.
    fn parse_newline(&mut self) -> Result<(), String> {
        self.expect("#NEWLINE")
    }

    // Checks normal text.
    fn parse_text(&mut self) -> Result<(), String> {
        if let Some(token) = self.current() {
            // Text cannot be an annotation.
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

    // Converts a list of tokens into a parser.
    fn parse_tokens(tokens: Vec<&str>) -> Result<(), String> {
        let tokens = tokens.into_iter().map(String::from).collect();
        Parser::new(tokens).parse_lolcode()
    }

    // Runs the lexer and parser on source code.
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