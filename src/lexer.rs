
pub trait LexicalAnalyzer {
    fn get_char(&mut self) -> char;
    fn add_char(&mut self, c: char);
    fn lookup(&self, s: &str) -> bool;
}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    current_lexeme: String,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Self {
            input: source.chars().collect(),
            position: 0,
            current_lexeme: String::new(),
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<String>, String> {
        let mut tokens = Vec::new();

        while self.position < self.input.len() {
            self.current_lexeme.clear();

            if self.input[self.position] == '#' {
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
                    return Err(format!("Invalid annotation: {}", annotation));
                }

                tokens.push(annotation);
            } else {
                while self.position < self.input.len()
                    && self.input[self.position] != '#'
                {
                    let c = self.get_char();
                    self.add_char(c);
                }

                if !self.current_lexeme.is_empty() {
                    tokens.push(self.current_lexeme.clone());
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
        c
    }

    fn add_char(&mut self, c: char) {
        self.current_lexeme.push(c);
    }

    fn lookup(&self, s: &str) -> bool {
        matches!(
            s,
            "#HAI" | "#KBYE" | "#OBTW" | "#TLDR" |
            "#MAEK" | "#MKAY" | "#GIMMEH" | "#OIC" |
            "#NEWLINE" | "#IHAZ" | "#ITIZ"
        )
    }
}
