use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    Result,
};

use std::io::stdout;
use std::io::Write;

use colored::*;
use crate::runtime::{Runtime, RuntimeConfig};

use strontium::machine::bytecode::BytecodeError;
use strontium::types::StrontiumError;

pub struct Repl {
    runtime: Runtime,
    cursor_x: usize,
    _history: Vec<String>,
    _history_y: usize,
}

impl Repl {
    pub fn new() -> Self {
        Self {
            runtime: Runtime::new(RuntimeConfig { debug: true }),
            cursor_x: 0,
            _history: vec![],
            _history_y: 0,
        }
    }

    pub fn launch(&mut self) -> Result<()> {
        println!("");
        println!("{}", std::fs::read_to_string("./logo.txt")?);
        let _should_continue = true;

        while _should_continue {
            // need to explicitly flush this to ensure it prints before read_line
            print!("{} ", ">>>".green().bold());
            stdout().flush()?;

            let line = format!("{}\n", self.read_line()?);

            /*self.runtime.lexer.add_text(line);
            let tokens = self.runtime.lexer.parse();
            let _tree = self.runtime.parser.add_tokens(self.runtime.lexer.source.clone(), tokens);
            let result = self.runtime.parser.parse();*/

            let result = self.runtime.compiler.compile(line.clone());

            match result {
                Ok(instructions) => {
                    println!("{}\n{:#?}", "instructions:".bright_blue().bold(), instructions);

                    for instruction in instructions.clone() {
                        self.runtime.machine.push_instruction(instruction);
                    }

                    if instructions.len() > 0 {
                        match self.runtime.machine.execute_until_eof() {
                            Ok(res) => println!("{:?}", res),
                            Err(e) => {
                                println!("{} {}", "error:".bright_red().bold(), format!("{:?}", e).bold());
                                match e {
                                    StrontiumError::BytecodeError(BytecodeError::UnexpectedEof(_)) => {
                                        println!("{} {:?}", "bytecode:".bright_blue().bold(), self.runtime.machine.registers.get("bc").unwrap());
                                    },

                                    _ => {},
                                }
                            }
                        }
                    }
                },
                Err(e) => {
                    println!("{} {}", "error:".bright_red().bold(), format!("{}", e).bold());
                    println!("{}",       "  |".blue().bold());
                    println!("{}    {}", "1 |".blue().bold(), line);
                    println!("{}",       "  |".blue().bold());
                }
            }
        }

        Ok(())
    }

    fn read_line(&mut self) -> Result<String> {
        let mut line = String::new();

        while let Event::Key(KeyEvent { code, .. }) = event::read()? {
            match code {
                KeyCode::Enter => {
                    break;
                },

                KeyCode::Left => {
                    self.cursor_x -= 1;
                    break;
                },

                KeyCode::Char(c) => {
                    line.push(c);
                    self.cursor_x += 1;
                },
                _ => {}
            }
        }

        Ok(line)
    }
}