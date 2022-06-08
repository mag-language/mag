extern crate linefeed;
extern crate rand;

use std::io;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use colored::*;

use rand::{Rng, thread_rng};
use clap::Parser as ClapParser;
use std::fs::File;
use std::io::prelude::*;

use linefeed::{Interface, Prompter, ReadResult};
use linefeed::chars::escape_sequence;
use linefeed::command::COMMANDS;
use linefeed::complete::{Completer, Completion};
use linefeed::inputrc::parse_text;
use linefeed::terminal::Terminal;

use magc::lexer::Lexer;
use magc::parser::{Parser, ParserError};

use magi::interpreter::Interpreter;
use magi::types::Obj;

const HISTORY_FILE: &str = "linefeed.hst";

/// Simple program to greet a person
#[derive(ClapParser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// A path to a file which contains Mag code.
    path: Option<String>,

    /// Number of times to greet
    #[clap(short, long, default_value_t = 1)]
    count: u8,
}

fn main() -> io::Result<()> {
    let interface = Arc::new(Interface::new("demo")?);
    let mut thread_id = 0;

    println!("Interactive Mag v0.3");
    println!("Enter \"help\" for a list of commands.");
    println!("Press Ctrl-D or enter \"quit\" to exit.");

    let args = Args::parse();

    if let Some(path) = args.path {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let mut lexer = Lexer::new(&contents);
        let tokens = lexer.parse();
        println!("{:#?}", &tokens);

        let mut parser = Parser::new(tokens);

        match parser.parse() {
            Ok(res) => println!("{:#?}", res),
            Err(e)   => {
                match e {
                    ParserError::MissingPrefixParselet(token_kind) => {
                        println!("{} {} {}", "error:".bright_red().bold(), "cannot find a prefix parselet for".bold(), format!("{:?}", token_kind).bold());
                    },

                    ParserError::UnexpectedEOF => {
                        println!("{} {}", "error:".bright_red().bold(), "unexpected EOF".bold())
                    },

                    ParserError::UnexpectedToken { expected, found } => {
                        println!("{} {}", "error:".bright_red().bold(), format!("expected token {:?}, found {:?}", expected, found).bold())
                    },

                    ParserError::UnexpectedExpression { expected, found } => {
                        println!("{} {}", "error:".bright_red().bold(), format!("expected expression {:?}, found {:#?}", expected, found).bold())
                    },

                    ParserError::ExpectedPattern => {
                        println!("{} {}", "error:".bright_red().bold(), format!("expected pattern").bold())
                    },

                    ParserError::NoMatch => {
                        println!("{} {}", "error:".bright_red().bold(), format!("the given pattern doesn't match the reference pattern").bold())
                    },
                }
            },
        }
    } else {
        let mut interpreter = Interpreter::new();

        interface.set_completer(Arc::new(DemoCompleter));
        interface.set_prompt(&format!("{} ", "mag>".green().bold()))?;

        if let Err(e) = interface.load_history(HISTORY_FILE) {
            if e.kind() == io::ErrorKind::NotFound {
                println!("History file {} doesn't exist, not loading history.", HISTORY_FILE);
            } else {
                eprintln!("Could not load history file {}: {}", HISTORY_FILE, e);
            }
        }

        while let ReadResult::Input(line) = interface.read_line()? {
            if !line.trim().is_empty() {
                interface.add_history_unique(line.clone());
            }

            let mut lexer = Lexer::new(&line);
            let tokens = lexer.parse();
            //println!("{:#?}", &tokens);

            let mut parser = Parser::new(tokens);

            match parser.parse() {
                Ok(res) => {
                    //println!("{:#?}", res);

                    for expr in res {
                        match interpreter.evaluate(Box::new(Obj::from(expr.clone())), None) {
                            Ok(obj) => println!("{}", format!("{}", obj).yellow()),
                            Err(e) => println!("{} {}", "error:".bright_red().bold(), format!("{:?}", e).bold()),
                        }
                    }
                },
                Err(e)   => {
                    match e {
                        ParserError::MissingPrefixParselet(token_kind) => {
                            println!("{} {} {}", "error:".bright_red().bold(), "cannot find a prefix parselet for".bold(), format!("{:?}", token_kind).bold());
                        },

                        ParserError::UnexpectedEOF => {
                            println!("{} {}", "error:".bright_red().bold(), "unexpected EOF".bold())
                        },

                        ParserError::UnexpectedToken { expected, found } => {
                            println!("{} {}", "error:".bright_red().bold(), format!("expected token {:?}, found {:?}", expected, found).bold())
                        },

                        ParserError::UnexpectedExpression { expected, found } => {
                            println!("{} {}", "error:".bright_red().bold(), format!("expected expression {:?}, found {:#?}", expected, found).bold())
                        },

                        ParserError::ExpectedPattern => {
                            println!("{} {}", "error:".bright_red().bold(), format!("expected pattern").bold())
                        },

                        ParserError::NoMatch => {
                            println!("{} {}", "error:".bright_red().bold(), format!("the given pattern doesn't match the reference pattern").bold())
                        },
                    }
                },
            }
        }

        println!("Goodbye.");
    }

    Ok(())
}

fn split_first_word(s: &str) -> (&str, &str) {
    let s = s.trim();

    match s.find(|ch: char| ch.is_whitespace()) {
        Some(pos) => (&s[..pos], s[pos..].trim_start()),
        None => (s, "")
    }
}

static DEMO_COMMANDS: &[(&str, &str)] = &[
    ("bind",             "Set bindings in inputrc format"),
    ("get",              "Print the value of a variable"),
    ("help",             "You're looking at it"),
    ("list-bindings",    "List bound sequences"),
    ("list-commands",    "List command names"),
    ("list-variables",   "List variables"),
    ("spawn-log-thread", "Spawns a thread that concurrently logs messages"),
    ("history",          "Print history"),
    ("save-history",     "Write history to file"),
    ("quit",             "Quit the demo"),
    ("set",              "Assign a value to a variable"),
];

struct DemoCompleter;

impl<Term: Terminal> Completer<Term> for DemoCompleter {
    fn complete(&self, word: &str, prompter: &Prompter<Term>,
            start: usize, _end: usize) -> Option<Vec<Completion>> {
        let line = prompter.buffer();

        let mut words = line[..start].split_whitespace();

        match words.next() {
            // Complete command name
            None => {
                let mut compls = Vec::new();

                for &(cmd, _) in DEMO_COMMANDS {
                    if cmd.starts_with(word) {
                        compls.push(Completion::simple(cmd.to_owned()));
                    }
                }

                Some(compls)
            }
            // Complete command parameters
            Some("get") | Some("set") => {
                if words.count() == 0 {
                    let mut res = Vec::new();

                    for (name, _) in prompter.variables() {
                        if name.starts_with(word) {
                            res.push(Completion::simple(name.to_owned()));
                        }
                    }

                    Some(res)
                } else {
                    None
                }
            }
            _ => None
        }
    }
}