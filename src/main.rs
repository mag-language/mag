pub mod repl;
pub mod runtime;

use repl::Repl;

fn main() {
    let mut repl = Repl::new();
    repl.launch().unwrap();
}