//! Lexical analyzer for LOLCODE Markdown.
//!
//! Reads the source one character at a time (following the Phase 1 state
//! diagram) and splits it into tokens: annotations such as `#HAI`, and
//! plain-text words.

/// Trait for a simple lexical analyzer.
/// Implements a character-by-character analysis from a state machine design.
pub trait LexicalAnalyzer {
    /// Return the next character from the input.
    fn get_char(&mut self) -> char;
    /// Add a character to the current potential token.
    fn add_char(&mut self, c: char);
    /// Lookup a potential token to determine if it is valid.
    fn lookup(&self, s: &str) -> bool;
}

/// Every legal annotation in the language.
pub const ANNOTATIONS: [&str; 11] = [
    "#HAI", "#KBYE", "#OBTW", "#TLDR", "#MAEK", "#MKAY", "#GIMMEH", "#OIC",
    "#NEWLINE", "#IHAZ", "#ITIZ",
];

/// Keywords that follow `#MAEK` or `#GIMMEH`.
pub const KEYWORDS: [&str; 8] = [
    "HEAD", "TITLE", "PARAGRAF", "BOLD", "LIST", "ITEM", "LINX", "VAR",
];

/// The lexer holds the input characters, where it is in them,
/// the lexeme being built, and the current line (for error messages).
pub struct Lexer {
    input: Vec<char>,
    position: usize,
    current_lexeme: String,
    line: usize,
    /// Line number of each token produced, used by the parser in errors.
    pub token_lines: Vec<usize>,
}

impl Lexer {
    /// Create a lexer for the given source text.
    pub fn new(source: &str) -> Self {
        Self {
            input: source.chars().collect(),
            position: 0,
            current_lexeme: String::new(),
            line: 1,
            token_lines: Vec::new(),
        }
    }

    /// True if `c` is a character allowed in plain text.
    fn is_allowed_text_char(c: char) -> bool {
        c.is_ascii_alphanumeric()
            || matches!(c, ',' | '.' | '"' | ':' | '?' | '!' | '%' | '=' | '/' | ' ' | '\t' | '\n' | '\r')
    }

    /// Run the state machine over the whole input and return the token list.
    /// Returns an error message on the first lexical error.
    pub fn tokenize(&mut self) -> Result<Vec<String>, String> {
        let mut tokens = Vec::new();

        while self.position < self.input.len() {
            self.current_lexeme.clear();
            let start_line = self.line;

            if self.input[self.position] == '#' {
                // S1/S2: gather an annotation up to the next whitespace.
                let ch = self.get_char();
                self.add_char(ch);
                while self.position < self.input.len()
                    && !self.input[self.position].is_whitespace()
                {
                    let c = self.get_char();
                    self.add_char(c);
                }

                let annotation = self.current_lexeme.to_uppercase();
                if !self.lookup(&annotation) {
                    return Err(format!(
                        "Lexical error (line {}): '{}' is not a valid annotation",
                        start_line, self.current_lexeme
                    ));
                }
                tokens.push(annotation);
                self.token_lines.push(start_line);
            } else {
                // S3/S4: gather plain text up to the next '#'.
                while self.position < self.input.len() && self.input[self.position] != '#' {
                    let c = self.get_char();
                    if !Self::is_allowed_text_char(c) {
                        return Err(format!(
                            "Lexical error (line {}): character '{}' is not allowed in text",
                            self.line, c
                        ));
                    }
                    self.add_char(c);
                }

                // Split the text into words, keeping each word's line number.
                let mut line = start_line;
                let mut word = String::new();
                for c in self.current_lexeme.chars() {
                    if c.is_whitespace() {
                        if !word.is_empty() {
                            tokens.push(std::mem::take(&mut word));
                            self.token_lines.push(line);
                        }
                        if c == '\n' {
                            line += 1;
                        }
                    } else {
                        word.push(c);
                    }
                }
                if !word.is_empty() {
                    tokens.push(word);
                    self.token_lines.push(line);
                }
            }
        }

        Ok(tokens)
    }
}

impl LexicalAnalyzer for Lexer {
    fn get_char(&mut self) -> char {
        let c = self.input[self.position];
        self.position += 1;
        if c == '\n' {
            self.line += 1;
        }
        c
    }

    fn add_char(&mut self, c: char) {
        self.current_lexeme.push(c);
    }

    fn lookup(&self, s: &str) -> bool {
        let upper = s.to_uppercase();
        ANNOTATIONS.contains(&upper.as_str()) || KEYWORDS.contains(&upper.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_is_case_insensitive() {
        let lexer = Lexer::new("");
        assert_eq!(lexer.lookup("#HAI"), true);
        assert_eq!(lexer.lookup("#hai"), true);
        assert_eq!(lexer.lookup("#HEY"), false);
    }

    #[test]
    fn get_and_add_char() {
        let mut lexer = Lexer::new("ab");
        assert_eq!(lexer.get_char(), 'a');
        lexer.add_char('a');
        assert_eq!(lexer.current_lexeme, "a");
        assert_eq!(lexer.get_char(), 'b');
    }

    #[test]
    fn tokenize_simple() {
        let mut lexer = Lexer::new("#hai Hello world #KBYE");
        assert_eq!(
            lexer.tokenize().unwrap(),
            vec!["#HAI", "Hello", "world", "#KBYE"]
        );
    }

    #[test]
    fn invalid_annotation_is_lexical_error() {
        let mut lexer = Lexer::new("#HAI #HEY #KBYE");
        assert!(lexer.tokenize().unwrap_err().starts_with("Lexical error"));
    }
}
