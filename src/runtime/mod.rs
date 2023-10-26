use magc::lexer::Lexer;
use magc::parser::Parser;
use magc::compiler::Compiler;
use strontium::Strontium;

/// A runtime instance, which contains all the data structures and methods needed to
/// compile and run a program while keeping track of its state and reporting errors.
pub struct Runtime {
    /// Converts a source string into a linear sequence of tokens.
    pub lexer:  Lexer,
    /// Assembles a sequence of tokens into a tree of expressions.
    pub parser: Parser,
    /// Compiles the AST into a sequence of instructions.
    pub compiler: Compiler,
    pub machine: Strontium,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            lexer:  Lexer::new(),
            parser: Parser::new(),
            compiler: Compiler::new(),
            machine: Strontium::new(),
        }
    }
}