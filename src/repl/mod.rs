use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    Result,
};

use std::io::stdout;
use std::io::Write;

use colored::*;
use super::runtime::Runtime;

pub struct Repl {
    runtime: Runtime,
    cursor_x: usize,
    _history: Vec<String>,
    _history_y: usize,
}

impl Repl {
    pub fn new() -> Self {
        Self {
            runtime: Runtime::new(),
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

            let line = self.read_line()?;

            /*self.runtime.lexer.add_text(line);
            let tokens = self.runtime.lexer.parse();
            let _tree = self.runtime.parser.add_tokens(self.runtime.lexer.source.clone(), tokens);
            let result = self.runtime.parser.parse();*/

            let result = self.runtime.compiler.compile(line);

            match result {
                Ok(instructions) => {
                    // println!("{}\n{:#?}", "instructions:".bright_blue().bold(), instructions);

                    for instruction in instructions {
                        self.runtime.machine.execute(instruction).unwrap();
                    }
                },
                Err(e) => {
                    println!("{} {:?}", "error:".bright_red().bold(), e);
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