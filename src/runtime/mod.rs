use magc::lexer::Lexer;
use magc::parser::Parser;
use magc::types::{Expression, Token, ParserError};
use magc::compiler::Compiler;
use strontium::Strontium;
use colored::*;

pub struct RuntimeConfig {
    pub debug: bool,
}

/// A runtime instance, which contains all the data structures and methods needed to
/// compile and run a program while keeping track of its state and reporting errors.
pub struct Runtime {
    pub config: RuntimeConfig,
    /// Converts a source string into a linear sequence of tokens.
    pub lexer:  Lexer,
    /// Assembles a sequence of tokens into a tree of expressions.
    pub parser: Parser,
    /// Compiles the AST into a sequence of instructions.
    pub compiler: Compiler,
    pub machine: Strontium,
}

impl Runtime {
    pub fn new(config: RuntimeConfig) -> Self {
        Self {
            config,
            lexer:  Lexer::new(),
            parser: Parser::new(),
            compiler: Compiler::new(),
            machine: Strontium::new(),
        }
    }

    pub fn lex(&mut self, source: String) -> Vec<Token> {
        self.lexer.add_text(source);
        self.lexer.parse()
    }

    pub fn parse(&mut self, source: String) -> Result<Vec<Expression>, ParserError> {
        self.lexer.add_text(source.clone());
        let tokens = self.lexer.parse();

        self.parser.add_tokens(source, tokens);
        self.parser.parse()
    }

    pub fn compile(&mut self, source: String) -> Result<Vec<strontium::Instruction>, String> {
        self.lexer.add_text(source.clone());
        let tokens = self.lexer.parse();

        self.parser.add_tokens(source, tokens);
        let expressions = self.parser.parse();

        match expressions {
            Ok(expressions) => {
                let mut bytecode = vec![];

                for expression in expressions {
                    let mut compiled = self.compiler.compile_expression(expression, None);

                    match compiled {
                        Ok(mut compiled) => {
                            bytecode.append(&mut compiled);
                        },
                        Err(e) => {
                            return Err(format!("{:?}", e));
                        }
                    }
                }

                Ok(bytecode)
            },
            Err(e) => {
                Err(format!("{:?}", e))
            }
        }
    }

    pub fn execute_source(&mut self, source: String) {
        let expressions_result = self.compile(source);

        match expressions_result {
            Ok(instructions) => {
                // println!("{}\n{:#?}", "instructions:".bright_blue().bold(), instructions);

                for instruction in instructions {
                    self.machine.push_instruction(instruction);
                }

                self.machine.execute().unwrap();
            },
            Err(e) => {
                println!("{} {:?}", "error:".bright_red().bold(), e);
            }
        }
    }
}