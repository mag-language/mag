use magc::lexer::Lexer;
use magc::parser::Parser;

pub struct Runtime {
    pub lexer:  Lexer,
    pub parser: Parser,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            lexer:  Lexer::new(),
            parser: Parser::new(),
        }
    }
}