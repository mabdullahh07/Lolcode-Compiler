use std::collections::HashSet;

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    current_lexeme: String,
    valid_annotations: HashSet<String>,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        let annotations = [
            "#HAI", "#KBYE", "#OBTW", "#TLDR", "#MAEK",
            "#OIC", "#GIMMEH", "#MKAY", "#NEWLINE",
            "#IHAZ", "#ITIZ",
        ];

        Self {
            input: source.chars().collect(),
            position: 0,
            current_lexeme: String::new(),
            valid_annotations: annotations.iter().map(|s| s.to_string()).collect(),
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<String>, String> {
        let mut tokens = Vec::new();

        while self.position < self.input.len() {
            let c = self.input[self.position];

            if c == '#' {
                self.current_lexeme.clear();
                self.current_lexeme.push(c);
                self.position += 1;

                while self.position < self.input.len()
                    && !self.input[self.position].is_whitespace()
                {
                    self.current_lexeme.push(self.input[self.position]);
                    self.position += 1;
                }

                let annotation = self.current_lexeme.to_uppercase();

                if !self.valid_annotations.contains(&annotation) {
                    return Err(format!("Invalid annotation: {}", annotation));
                }

                tokens.push(annotation);
            } else if c == '\n' {
                tokens.push("#NEWLINE".to_string());
                self.position += 1;
            } else if c.is_whitespace() {
                self.position += 1;
            } else {
                self.current_lexeme.clear();

                while self.position < self.input.len()
                    && self.input[self.position] != '#'
                    && self.input[self.position] != '\n'
                {
                    self.current_lexeme.push(self.input[self.position]);
                    self.position += 1;
                }

                let text = self.current_lexeme.trim().to_string();

                if !text.is_empty() {
                    // Separate grammar words from ordinary text.
                    for word in text.split_whitespace() {
                        tokens.push(word.to_string());
                    }
                }
            }
        }

        Ok(tokens)
    }
}
