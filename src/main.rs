mod lexer;
mod parser;

use lexer::Lexer;
use parser::Parser;

fn main() {
    let sample = "#HAI #MAEK HEAD #GIMMEH TITLE My Document #OIC #MKAY Hello world! #KBYE";

    let mut lexer = Lexer::new(sample);

    match lexer.tokenize() {
        Ok(tokens) => {
            println!("Tokens: {:?}", tokens);

            let mut parser = Parser::new(tokens);

            match parser.parse_lolcode() {
                Ok(()) => println!("Syntax is valid!"),
                Err(error) => println!("Syntax error: {}", error),
            }
        }
        Err(error) => println!("Lexical error: {}", error),
    }
}
