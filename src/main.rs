pub mod repl;
pub mod runtime;

use clap::Parser;
use repl::Repl;

#[derive(Parser)]
#[clap(name = "mag", about = "The Mag Language Runtime")]
struct Args {
    /// Enable debug output
    #[clap(long)]
    debug: bool,
}

fn main() {
    let args = Args::parse();
    let mut repl = Repl::new(args.debug);
    repl.launch().unwrap();
}