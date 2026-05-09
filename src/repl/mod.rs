use std::collections::VecDeque;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use crossterm::{
    cursor,
    event::{
        self, DisableBracketedPaste, EnableBracketedPaste, Event, KeyCode, KeyEvent, KeyModifiers,
    },
    style::Print,
    terminal, ExecutableCommand, QueueableCommand, Result,
};
use std::time::Duration;

use crate::runtime::{Runtime, RuntimeConfig};
use colored::*;
use signal_hook::{consts::SIGINT, flag};

use strontium::machine::bytecode::BytecodeError;
use strontium::machine::CancellationToken;
use strontium::types::StrontiumError;

const PROMPT: &str = ">>> ";
const PROMPT_WIDTH: usize = 4;
const HISTORY_LIMIT: usize = 1000;
const PASTE_DEBOUNCE: Duration = Duration::from_millis(8);

pub struct Repl {
    runtime: Runtime,
    cancellation: CancellationToken,
}

impl Repl {
    pub fn new(debug: bool) -> Self {
        Self {
            runtime: Runtime::new(RuntimeConfig { debug }),
            cancellation: CancellationToken::new(),
        }
    }

    pub fn launch(&mut self) -> Result<()> {
        println!("");
        println!("{}", std::fs::read_to_string("./logo.txt")?);
        flag::register(SIGINT, self.cancellation.flag())?;

        let mut editor = LineEditor::new(Self::history_path(), ReplTheme::from_env());
        editor.load_history();

        loop {
            match editor.read_line()? {
                ReadLine::Input(input) => {
                    let trimmed = input.trim();

                    match trimmed {
                        "" => continue,
                        ":quit" | ":exit" => break,
                        _ => {}
                    }

                    editor.add_history(input.clone());
                    self.execute_line(format!("{}\n", input));
                }
                ReadLine::Interrupted => continue,
                ReadLine::Eof => break,
            }
        }

        editor.save_history();

        Ok(())
    }

    fn execute_line(&mut self, line: String) {
        let result = self.runtime.compiler.compile(line.clone());

        match result {
            Ok(instructions) => {
                if self.runtime.config.debug {
                    println!(
                        "{}\n{:#?}",
                        "instructions:".bright_blue().bold(),
                        instructions
                    );
                }

                self.runtime.machine.reset();
                self.runtime.machine.multimethod_table.clear();
                for reg in &self.runtime.compiler.method_registrations {
                    self.runtime.machine.register_method(
                        reg.method_name.clone(),
                        reg.pattern.clone(),
                        reg.address,
                    );
                }

                for instruction in instructions.clone() {
                    self.runtime.machine.push_instruction(instruction);
                }

                if instructions.len() > 0 {
                    self.cancellation.reset();
                    match self
                        .runtime
                        .machine
                        .execute_until_eof_cancellable(&self.cancellation)
                    {
                        Ok(_) => {}
                        Err(StrontiumError::Interrupted) => {
                            println!("\n{}", "interrupted".bright_yellow().bold());
                            self.cancellation.reset();
                        }
                        Err(e) => {
                            println!(
                                "{} {}",
                                "error:".bright_red().bold(),
                                format!("{:?}", e).bold()
                            );
                            match e {
                                StrontiumError::BytecodeError(BytecodeError::UnexpectedEof(_)) => {
                                    println!(
                                        "{} {:?}",
                                        "bytecode:".bright_blue().bold(),
                                        self.runtime.machine.registers.get("bc").unwrap()
                                    );
                                }

                                _ => {}
                            }
                        }
                    }
                }
            }
            Err(e) => {
                println!(
                    "{} {}",
                    "error:".bright_red().bold(),
                    format!("{}", e).bold()
                );
                println!("{}", "  |".blue().bold());
                println!("{}    {}", "1 |".blue().bold(), line);
                println!("{}", "  |".blue().bold());
            }
        }
    }

    fn history_path() -> Option<PathBuf> {
        std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".mag_history"))
    }
}

enum ReadLine {
    Input(String),
    Interrupted,
    Eof,
}

struct LineEditor {
    history: Vec<String>,
    history_path: Option<PathBuf>,
    theme: ReplTheme,
    rendered_width: usize,
    pending_events: VecDeque<Event>,
    ignore_next_submit: bool,
}

impl LineEditor {
    fn new(history_path: Option<PathBuf>, theme: ReplTheme) -> Self {
        Self {
            history: vec![],
            history_path,
            theme,
            rendered_width: 0,
            pending_events: VecDeque::new(),
            ignore_next_submit: false,
        }
    }

    fn read_line(&mut self) -> Result<ReadLine> {
        let _raw = RawMode::enable()?;
        let mut stdout = io::stdout();
        let mut buffer = InputBuffer::new();
        let mut history_pos: Option<usize> = None;
        let mut draft: Vec<char> = vec![];

        self.render(&mut stdout, &buffer)?;

        loop {
            match self.read_event()? {
                Event::Paste(pasted) => {
                    buffer.insert_text(&pasted);
                    history_pos = None;
                }
                Event::Key(KeyEvent {
                    code, modifiers, ..
                }) => match (code, modifiers) {
                    (KeyCode::Enter, _) => {
                        if self.ignore_next_submit {
                            self.ignore_next_submit = false;
                            continue;
                        }
                        stdout.queue(Print("\r\n"))?;
                        stdout.flush()?;
                        return Ok(ReadLine::Input(buffer.to_string()));
                    }
                    (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                        stdout.queue(Print("^C\r\n"))?;
                        stdout.flush()?;
                        return Ok(ReadLine::Interrupted);
                    }
                    (KeyCode::Char('d'), KeyModifiers::CONTROL) if buffer.is_empty() => {
                        stdout.queue(Print("\r\n"))?;
                        stdout.flush()?;
                        return Ok(ReadLine::Eof);
                    }
                    (KeyCode::Char('a'), KeyModifiers::CONTROL) | (KeyCode::Home, _) => {
                        buffer.move_home();
                    }
                    (KeyCode::Char('e'), KeyModifiers::CONTROL) | (KeyCode::End, _) => {
                        buffer.move_end();
                    }
                    (KeyCode::Left, _) => {
                        buffer.move_left();
                    }
                    (KeyCode::Right, _) => {
                        buffer.move_right();
                    }
                    (KeyCode::Up, _) => {
                        self.move_history(-1, &mut history_pos, &mut draft, &mut buffer);
                        buffer.move_end();
                    }
                    (KeyCode::Down, _) => {
                        self.move_history(1, &mut history_pos, &mut draft, &mut buffer);
                        buffer.move_end();
                    }
                    (KeyCode::Backspace, _) => {
                        buffer.backspace();
                        history_pos = None;
                    }
                    (KeyCode::Delete, _) => {
                        buffer.delete();
                        history_pos = None;
                    }
                    (KeyCode::Esc, _) => {
                        buffer.clear();
                        history_pos = None;
                        draft.clear();
                        self.pending_events.clear();
                        self.ignore_next_submit = true;
                    }
                    (KeyCode::Char(c), _) => {
                        buffer.insert_char(c);
                        history_pos = None;
                        self.ignore_next_submit = false;
                        self.consume_queued_text(&mut buffer)?;
                    }
                    _ => {}
                },
                _ => {}
            }

            self.render(&mut stdout, &buffer)?;
        }
    }

    fn read_event(&mut self) -> Result<Event> {
        match self.pending_events.pop_front() {
            Some(event) => Ok(event),
            None => event::read(),
        }
    }

    fn consume_queued_text(&mut self, buffer: &mut InputBuffer) -> Result<()> {
        while event::poll(PASTE_DEBOUNCE)? {
            match event::read()? {
                Event::Paste(pasted) => buffer.insert_text(&pasted),
                Event::Key(KeyEvent {
                    code: KeyCode::Char(c),
                    modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
                    ..
                }) => {
                    buffer.insert_char(c);
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Enter,
                    ..
                }) => buffer.insert_char('\n'),
                event => {
                    self.pending_events.push_back(event);
                    break;
                }
            }
        }

        Ok(())
    }

    fn load_history(&mut self) {
        if let Some(path) = &self.history_path {
            match fs::read_to_string(path) {
                Ok(history) => {
                    self.history = history
                        .lines()
                        .filter(|line| !line.trim().is_empty())
                        .map(String::from)
                        .collect();
                    self.truncate_history();
                }
                Err(e) if e.kind() == io::ErrorKind::NotFound => {}
                Err(e) => eprintln!(
                    "{} failed to load history: {}",
                    "warning:".bright_yellow().bold(),
                    e
                ),
            }
        }
    }

    fn save_history(&self) {
        if let Some(path) = &self.history_path {
            if let Err(e) = fs::write(path, self.history.join("\n")) {
                eprintln!(
                    "{} failed to save history: {}",
                    "warning:".bright_yellow().bold(),
                    e
                );
            }
        }
    }

    fn add_history(&mut self, line: String) {
        if line.trim().is_empty() {
            return;
        }

        if self.history.last() != Some(&line) {
            self.history.push(line);
            self.truncate_history();
        }
    }

    fn truncate_history(&mut self) {
        if self.history.len() > HISTORY_LIMIT {
            let overflow = self.history.len() - HISTORY_LIMIT;
            self.history.drain(0..overflow);
        }
    }

    fn move_history(
        &self,
        direction: isize,
        history_pos: &mut Option<usize>,
        draft: &mut Vec<char>,
        buffer: &mut InputBuffer,
    ) {
        if self.history.is_empty() {
            return;
        }

        match (*history_pos, direction) {
            (None, -1) => {
                *draft = buffer.chars.clone();
                *history_pos = Some(self.history.len() - 1);
            }
            (Some(pos), -1) if pos > 0 => *history_pos = Some(pos - 1),
            (Some(pos), 1) if pos + 1 < self.history.len() => *history_pos = Some(pos + 1),
            (Some(_), 1) => {
                *history_pos = None;
                buffer.replace(draft.clone());
                return;
            }
            _ => return,
        }

        if let Some(pos) = *history_pos {
            buffer.replace(self.history[pos].chars().collect());
        }
    }

    fn render<W: Write>(&mut self, stdout: &mut W, buffer: &InputBuffer) -> Result<()> {
        let input = buffer.to_string();
        let width = PROMPT_WIDTH + buffer.len();
        let clear_width = self.rendered_width.max(width);

        stdout
            .queue(cursor::MoveToColumn(0))?
            .queue(Print(" ".repeat(clear_width)))?
            .queue(cursor::MoveToColumn(0))?
            .queue(Print(self.theme.prompt(PROMPT)))?
            .queue(Print(highlight_mag(&input, &self.theme)))?
            .queue(cursor::MoveToColumn(
                (PROMPT_WIDTH + buffer.cursor()).min(u16::MAX as usize) as u16,
            ))?;

        self.rendered_width = width;
        stdout.flush()?;
        Ok(())
    }
}

#[derive(Clone, Default)]
struct InputBuffer {
    chars: Vec<char>,
    cursor: usize,
}

impl InputBuffer {
    fn new() -> Self {
        Self::default()
    }

    fn len(&self) -> usize {
        self.chars.len()
    }

    fn cursor(&self) -> usize {
        self.cursor
    }

    fn is_empty(&self) -> bool {
        self.chars.is_empty()
    }

    fn to_string(&self) -> String {
        self.chars.iter().collect()
    }

    fn replace(&mut self, chars: Vec<char>) {
        self.chars = chars;
        self.cursor = self.chars.len();
    }

    fn clear(&mut self) {
        self.chars.clear();
        self.cursor = 0;
    }

    fn insert_char(&mut self, ch: char) {
        if let Some(ch) = normalize_pasted_char(ch) {
            self.chars.insert(self.cursor, ch);
            self.cursor += 1;
        }
    }

    fn insert_text(&mut self, text: &str) {
        for ch in text.chars() {
            self.insert_char(ch);
        }
    }

    fn backspace(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.chars.remove(self.cursor);
        }
    }

    fn delete(&mut self) {
        if self.cursor < self.chars.len() {
            self.chars.remove(self.cursor);
        }
    }

    fn move_home(&mut self) {
        self.cursor = 0;
    }

    fn move_end(&mut self) {
        self.cursor = self.chars.len();
    }

    fn move_left(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    fn move_right(&mut self) {
        if self.cursor < self.chars.len() {
            self.cursor += 1;
        }
    }
}

struct RawMode;

impl RawMode {
    fn enable() -> Result<Self> {
        terminal::enable_raw_mode()?;
        io::stdout().execute(EnableBracketedPaste)?;
        Ok(Self)
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        let _ = io::stdout().execute(DisableBracketedPaste);
        let _ = terminal::disable_raw_mode();
    }
}

fn normalize_pasted_char(ch: char) -> Option<char> {
    match ch {
        '\r' | '\n' | '\t' => Some(' '),
        ch if ch.is_control() => None,
        ch => Some(ch),
    }
}

#[derive(Clone, Copy)]
struct ReplTheme {
    prompt: &'static str,
    keyword: &'static str,
    type_name: &'static str,
    string: &'static str,
    number: &'static str,
    punctuation: &'static str,
    operator: &'static str,
}

impl ReplTheme {
    fn from_env() -> Self {
        match std::env::var("MAG_REPL_THEME").as_deref() {
            Ok("mono") | Ok("plain") => Self::mono(),
            _ => Self::mag(),
        }
    }

    fn mag() -> Self {
        Self {
            prompt: "\x1b[1;38;5;120m",
            keyword: "\x1b[1;38;5;75m",
            type_name: "\x1b[1;38;5;207m",
            string: "\x1b[1;38;5;214m",
            number: "\x1b[1;38;5;214m",
            punctuation: "\x1b[1;38;5;111m",
            operator: "\x1b[1;38;5;210m",
        }
    }

    fn mono() -> Self {
        Self {
            prompt: "",
            keyword: "",
            type_name: "",
            string: "",
            number: "",
            punctuation: "",
            operator: "",
        }
    }

    fn prompt(&self, text: &str) -> String {
        self.paint(self.prompt, text)
    }

    fn keyword(&self, text: &str) -> String {
        self.paint(self.keyword, text)
    }

    fn type_name(&self, text: &str) -> String {
        self.paint(self.type_name, text)
    }

    fn string(&self, text: &str) -> String {
        self.paint(self.string, text)
    }

    fn number(&self, text: &str) -> String {
        self.paint(self.number, text)
    }

    fn punctuation(&self, text: &str) -> String {
        self.paint(self.punctuation, text)
    }

    fn operator(&self, text: &str) -> String {
        self.paint(self.operator, text)
    }

    fn paint(&self, style: &str, text: &str) -> String {
        if style.is_empty() {
            text.to_string()
        } else {
            format!("{}{}\x1b[0m", style, text)
        }
    }
}

fn highlight_mag(input: &str, theme: &ReplTheme) -> String {
    let mut highlighted = String::new();
    let chars: Vec<char> = input.chars().collect();
    let mut index = 0;

    while index < chars.len() {
        let ch = chars[index];

        if ch == '"' {
            let start = index;
            index += 1;

            while index < chars.len() {
                let current = chars[index];
                index += 1;

                if current == '"' {
                    break;
                }
            }

            let text = chars[start..index].iter().collect::<String>();
            highlighted.push_str(&theme.string(&text));
        } else if ch.is_ascii_digit() {
            let start = index;
            index += 1;

            while index < chars.len() && (chars[index].is_ascii_digit() || chars[index] == '.') {
                index += 1;
            }

            let text = chars[start..index].iter().collect::<String>();
            highlighted.push_str(&theme.number(&text));
        } else if is_identifier_start(ch) {
            let start = index;
            index += 1;

            while index < chars.len() && is_identifier_continue(chars[index]) {
                index += 1;
            }

            let word: String = chars[start..index].iter().collect();
            if is_keyword(&word) {
                highlighted.push_str(&theme.keyword(&word));
            } else if word.chars().next().map(char::is_uppercase).unwrap_or(false) {
                highlighted.push_str(&theme.type_name(&word));
            } else {
                highlighted.push_str(&word);
            }
        } else if "()[]{}.,:".contains(ch) {
            highlighted.push_str(&theme.punctuation(&ch.to_string()));
            index += 1;
        } else if "+-*/%=!<>^".contains(ch) {
            highlighted.push_str(&theme.operator(&ch.to_string()));
            index += 1;
        } else {
            highlighted.push(ch);
            index += 1;
        }
    }

    highlighted
}

fn is_identifier_start(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphabetic()
}

fn is_identifier_continue(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphanumeric()
}

fn is_keyword(word: &str) -> bool {
    matches!(
        word,
        "and"
            | "as"
            | "catch"
            | "case"
            | "const"
            | "def"
            | "do"
            | "else"
            | "end"
            | "enum"
            | "false"
            | "for"
            | "if"
            | "import"
            | "interface"
            | "it"
            | "match"
            | "or"
            | "return"
            | "then"
            | "this"
            | "true"
            | "var"
            | "while"
            | "with"
    )
}
