// This file reads the source code and splits it into tokens.

// Contains the functions used by the lexer.
pub trait LexicalAnalyzer {
    // Gets the next character from the source code.
    fn get_char(&mut self) -> char;

    // Adds a character to the current token.
    fn add_char(&mut self, c: char);

    // Checks if a token is valid.
    fn lookup(&self, s: &str) -> bool;
}

// All annotations allowed in the language.
pub const ANNOTATIONS: [&str; 11] = [
    "#HAI", "#KBYE", "#OBTW", "#TLDR", "#MAEK", "#MKAY", "#GIMMEH", "#OIC",
    "#NEWLINE", "#IHAZ", "#ITIZ",
];

// Keywords that can be used after annotations.
pub const KEYWORDS: [&str; 8] = [
    "HEAD", "TITLE", "PARAGRAF", "BOLD", "LIST", "ITEM", "LINX", "VAR",
];

// Stores the source code and keeps track of the current position.
pub struct Lexer {
    input: Vec<char>,
    position: usize,
    current_lexeme: String,
    line: usize,

    // Stores the line number for every token.
    pub token_lines: Vec<usize>,
}

impl Lexer {
    // Creates a new lexer using the source code.
    pub fn new(source: &str) -> Self {
        Self {
            input: source.chars().collect(),
            position: 0,
            current_lexeme: String::new(),
            line: 1,
            token_lines: Vec::new(),
        }
    }

    // Checks which characters are allowed in normal text.
    fn is_allowed_text_char(c: char) -> bool {
        c.is_ascii_alphanumeric()
            || matches!(
                c,
                ',' | '.' | '"' | ':' | '?' | '!' | '%' | '=' | '/' | ' ' | '\t' | '\n' | '\r'
            )
    }

    // Reads the whole source code and creates tokens.
    pub fn tokenize(&mut self) -> Result<Vec<String>, String> {
        let mut tokens = Vec::new();

        while self.position < self.input.len() {
            self.current_lexeme.clear();
            let start_line = self.line;

            // If the character is #, it is an annotation or keyword.
            if self.input[self.position] == '#' {
                // Reads the annotation until whitespace.
                let ch = self.get_char();
                self.add_char(ch);

                while self.position < self.input.len()
                    && !self.input[self.position].is_whitespace()
                {
                    let c = self.get_char();
                    self.add_char(c);
                }

                // Checks if the annotation is valid.
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
                // Reads normal text until the next annotation.
                while self.position < self.input.len()
                    && self.input[self.position] != '#'
                {
                    let c = self.get_char();

                    // Stops if an invalid character is found.
                    if !Self::is_allowed_text_char(c) {
                        return Err(format!(
                            "Lexical error (line {}): character '{}' is not allowed in text",
                            self.line, c
                        ));
                    }

                    self.add_char(c);
                }

                // Splits normal text into separate words.
                let mut line = start_line;
                let mut word = String::new();

                for c in self.current_lexeme.chars() {
                    if c.is_whitespace() {
                        // Saves the word when whitespace is reached.
                        if !word.is_empty() {
                            tokens.push(std::mem::take(&mut word));
                            self.token_lines.push(line);
                        }

                        // Updates the line number after a new line.
                        if c == '\n' {
                            line += 1;
                        }
                    } else {
                        word.push(c);
                    }
                }

                // Saves the last word if there is one.
                if !word.is_empty() {
                    tokens.push(word);
                    self.token_lines.push(line);
                }
            }
        }

        Ok(tokens)
    }
}

// Implements the lexer functions.
impl LexicalAnalyzer for Lexer {
    fn get_char(&mut self) -> char {
        let c = self.input[self.position];
        self.position += 1;

        // Moves to the next line when a newline is read.
        if c == '\n' {
            self.line += 1;
        }

        c
    }

    fn add_char(&mut self, c: char) {
        self.current_lexeme.push(c);
    }

    // Checks if the word is an annotation or keyword.
    fn lookup(&self, s: &str) -> bool {
        let upper = s.to_uppercase();
        ANNOTATIONS.contains(&upper.as_str())
            || KEYWORDS.contains(&upper.as_str())
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