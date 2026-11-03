use crate::ast::AstParser;
use crate::html;
use crate::lexer::Lexer;
use crate::parser::{Parser, SyntaxAnalyzer};

// Contains the main functions used by the compiler.
pub trait Compiler {
    // Starts compiling the source code.
    fn compile(&mut self, source: &str);

    // Gets the next token.
    fn next_token(&mut self) -> String;

    // Runs the parser.
    fn parse(&mut self);

    // Gets the current token.
    fn current_token(&self) -> String;

    // Changes the current token.
    fn set_current_token(&mut self, tok: String);
}

// Stores the tokens and the generated HTML.
pub struct LolCompiler {
    tokens: Vec<String>,
    lines: Vec<usize>,
    position: usize,
    current: String,

    // HTML is stored here after successful compilation.
    pub html: Option<String>,
}

impl LolCompiler {
    // Creates a new compiler with empty values.
    pub fn new() -> Self {
        Self {
            tokens: Vec::new(),
            lines: Vec::new(),
            position: 0,
            current: String::new(),
            html: None,
        }
    }

    // Converts the source code into tokens.
    pub fn tokenize(&mut self, source: &str) -> Result<(), String> {
        let mut lexer = Lexer::new(source);

        // Gets tokens from the lexer.
        self.tokens = lexer.tokenize()?;

        // Saves the line numbers of the tokens.
        self.lines = lexer.token_lines;

        // Resets the token position.
        self.position = 0;
        self.current.clear();

        Ok(())
    }

    // Runs the lexer, parser, AST builder, and HTML generator.
    pub fn try_compile(&mut self, source: &str) -> Result<String, String> {
        self.tokenize(source)?;

        // Checks the source code using the parser.
        let mut parser = Parser::with_lines(
            self.tokens.clone(),
            self.lines.clone(),
        );

        if let Err(message) = parser.parse_lolcode() {
            // Shows semantic errors separately from syntax errors.
            return Err(if message.starts_with("Static semantic error") {
                format!("{} (line {})", message, parser.error_line())
            } else {
                format!(
                    "Syntax error (line {}): {}",
                    parser.error_line(),
                    message
                )
            });
        }

        // Builds the AST and converts it into HTML.
        let document = AstParser::new(self.tokens.clone()).parse_document()?;
        Ok(html::to_html(&document))
    }
}

// Implements the compiler functions for LolCompiler.
impl Compiler for LolCompiler {
    fn compile(&mut self, source: &str) {
        match self.try_compile(source) {
            // Saves the HTML if compilation succeeds.
            Ok(output) => self.html = Some(output),

            // Prints the error and stops the program.
            Err(message) => {
                eprintln!("{}", message);
                std::process::exit(1);
            }
        }
    }

    fn next_token(&mut self) -> String {
        // Gets the token at the current position.
        let token = self.tokens
            .get(self.position)
            .cloned()
            .unwrap_or_default();

        // Moves to the next token.
        if self.position < self.tokens.len() {
            self.position += 1;
        }

        self.set_current_token(token.clone());
        token
    }

    fn parse(&mut self) {
        // Creates a parser and checks the tokens.
        let mut parser = Parser::with_lines(
            self.tokens.clone(),
            self.lines.clone(),
        );

        if let Err(message) = parser.parse_lolcode() {
            eprintln!(
                "Syntax error (line {}): {}",
                parser.error_line(),
                message
            );
            std::process::exit(1);
        }
    }

    fn current_token(&self) -> String {
        self.current.clone()
    }

    fn set_current_token(&mut self, tok: String) {
        self.current = tok;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_and_current_token() {
        let mut c = LolCompiler::new();
        c.tokenize("#HAI hello #KBYE").unwrap();

        assert_eq!(c.next_token(), "#HAI");
        assert_eq!(c.current_token(), "#HAI");
        assert_eq!(c.next_token(), "hello");
        assert_eq!(c.next_token(), "#KBYE");
        assert_eq!(c.next_token(), "");
    }

    #[test]
    fn set_current_token() {
        let mut c = LolCompiler::new();
        c.set_current_token("#OIC".to_string());
        assert_eq!(c.current_token(), "#OIC");
    }

    #[test]
    fn static_scoping() {
        let src = "#HAI #IHAZ name #ITIZ Josh #MKAY Hi #GIMMEH VAR name #OIC \
                   #MAEK PARAGRAF #IHAZ name #ITIZ Jon #MKAY #GIMMEH VAR name #OIC #MKAY \
                   #GIMMEH VAR name #OIC #KBYE";

        let html = LolCompiler::new().try_compile(src).unwrap();

        assert!(html.contains("Hi Josh"));
        assert!(html.contains("<p>Jon</p>"));
        assert!(html.trim_end().ends_with("Josh\n</body>\n</html>"));
    }

    #[test]
    fn undefined_variable_is_semantic_error() {
        let err = LolCompiler::new()
            .try_compile("#HAI #GIMMEH VAR x #OIC #KBYE")
            .unwrap_err();

        assert!(err.starts_with("Static semantic error"));
    }

    #[test]
    fn misplaced_mkay_is_syntax_error() {
        let err = LolCompiler::new()
            .try_compile("#HAI #MKAY #KBYE")
            .unwrap_err();

        assert!(err.starts_with("Syntax error"));
    }

    #[test]
    fn bold_with_variable() {
        let src = "#HAI #IHAZ last #ITIZ Simpson #MKAY #GIMMEH BOLD #GIMMEH VAR last #OIC #OIC #KBYE";
        let html = LolCompiler::new().try_compile(src).unwrap();

        assert!(html.contains("<b>Simpson</b>"));
    }
}