extern crate linefeed;
extern crate rand;

use std::io;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use rand::{Rng, thread_rng};

use linefeed::{Interface, Prompter, ReadResult};
use linefeed::chars::escape_sequence;
use linefeed::command::COMMANDS;
use linefeed::complete::{Completer, Completion};
use linefeed::inputrc::parse_text;
use linefeed::terminal::Terminal;

use magc::scanner::Scanner;

const HISTORY_FILE: &str = "linefeed.hst";

fn main() -> io::Result<()> {
    let interface = Arc::new(Interface::new("demo")?);
    let mut thread_id = 0;

    println!("This is the linefeed demo program.");
    println!("Enter \"help\" for a list of commands.");
    println!("Press Ctrl-D or enter \"quit\" to exit.");
    println!("");

    interface.set_completer(Arc::new(DemoCompleter));
    interface.set_prompt("demo> ")?;

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

        let mut scanner = Scanner::new(&line);
        println!("{:#?}", scanner.parse());
    }

    println!("Goodbye.");

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