//! The compiler driver: runs the lexer, the parser, then builds the
//! AST and produces the HTML text.

use crate::ast::AstParser;
use crate::html;
use crate::lexer::Lexer;
use crate::parser::{Parser, SyntaxAnalyzer};

/// Trait for a simple lolcompiler front-end.
/// Errors should cause immediate exit inside the implementation.
pub trait Compiler {
    /// Begin the compilation process (entry point).
    fn compile(&mut self, source: &str);
    /// Get the next token from the lexical analyzer.
    fn next_token(&mut self) -> String;
    /// Run the syntax analyzer starting from <lolcode>.
    fn parse(&mut self);
    /// Get the current token being processed.
    fn current_token(&self) -> String;
    /// Set the current token (typically used internally).
    fn set_current_token(&mut self, tok: String);
}

/// Holds the token stream and, after a successful compile, the HTML output.
pub struct LolCompiler {
    tokens: Vec<String>,
    lines: Vec<usize>,
    position: usize,
    current: String,
    /// The generated HTML (only set when compilation succeeds).
    pub html: Option<String>,
}

impl LolCompiler {
    /// Create an empty compiler.
    pub fn new() -> Self {
        Self { tokens: Vec::new(), lines: Vec::new(), position: 0, current: String::new(), html: None }
    }

    /// Run the lexer on `source` and store the tokens. Returns the error
    /// message instead of exiting, so it can be unit tested.
    pub fn tokenize(&mut self, source: &str) -> Result<(), String> {
        let mut lexer = Lexer::new(source);
        self.tokens = lexer.tokenize()?;
        self.lines = lexer.token_lines;
        self.position = 0;
        self.current.clear();
        Ok(())
    }

    /// Full pipeline without exiting: lex, parse, build AST, make HTML.
    pub fn try_compile(&mut self, source: &str) -> Result<String, String> {
        self.tokenize(source)?;

        let mut parser = Parser::with_lines(self.tokens.clone(), self.lines.clone());
        if let Err(message) = parser.parse_lolcode() {
            return Err(if message.starts_with("Static semantic error") {
                format!("{} (line {})", message, parser.error_line())
            } else {
                format!("Syntax error (line {}): {}", parser.error_line(), message)
            });
        }

        let document = AstParser::new(self.tokens.clone()).parse_document()?;
        Ok(html::to_html(&document))
    }
}

impl Compiler for LolCompiler {
    fn compile(&mut self, source: &str) {
        match self.try_compile(source) {
            Ok(output) => self.html = Some(output),
            Err(message) => {
                eprintln!("{}", message);
                std::process::exit(1);
            }
        }
    }

    fn next_token(&mut self) -> String {
        let token = self.tokens.get(self.position).cloned().unwrap_or_default();
        if self.position < self.tokens.len() {
            self.position += 1;
        }
        self.set_current_token(token.clone());
        token
    }

    fn parse(&mut self) {
        let mut parser = Parser::with_lines(self.tokens.clone(), self.lines.clone());
        if let Err(message) = parser.parse_lolcode() {
            eprintln!("Syntax error (line {}): {}", parser.error_line(), message);
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
        let err = LolCompiler::new().try_compile("#HAI #GIMMEH VAR x #OIC #KBYE").unwrap_err();
        assert!(err.starts_with("Static semantic error"));
    }

    #[test]
    fn misplaced_mkay_is_syntax_error() {
        let err = LolCompiler::new().try_compile("#HAI #MKAY #KBYE").unwrap_err();
        assert!(err.starts_with("Syntax error"));
    }

    #[test]
    fn bold_with_variable() {
        let src = "#HAI #IHAZ last #ITIZ Simpson #MKAY #GIMMEH BOLD #GIMMEH VAR last #OIC #OIC #KBYE";
        let html = LolCompiler::new().try_compile(src).unwrap();
        assert!(html.contains("<b>Simpson</b>"));
    }
}
