pub mod repl;
pub mod runtime;

use std::fs;
use std::io::{self, IsTerminal, Read};

use clap::Parser;
use colored::*;
use repl::Repl;
use runtime::{Runtime, RuntimeConfig};

use strontium::machine::bytecode::BytecodeError;
use strontium::types::StrontiumError;

#[derive(Parser)]
#[clap(name = "mag", about = "The Mag Language Runtime")]
struct Args {
    /// Source file to execute
    file: Option<String>,

    /// Enable debug output
    #[clap(long)]
    debug: bool,
}

fn main() {
    let args = Args::parse();

    // Check if we have a file argument
    if let Some(file_path) = args.file {
        run_file(&file_path, args.debug);
        return;
    }

    // Check if stdin has data (piped input)
    if !io::stdin().is_terminal() {
        let mut source = String::new();
        io::stdin()
            .read_to_string(&mut source)
            .expect("failed to read stdin");
        run_source(source, args.debug);
        return;
    }

    // Otherwise, launch the REPL
    let mut repl = Repl::new(args.debug);
    repl.launch().unwrap();
}

fn run_file(path: &str, debug: bool) {
    match fs::read_to_string(path) {
        Ok(source) => run_source(source, debug),
        Err(e) => {
            eprintln!(
                "{} failed to read file '{}': {}",
                "error:".bright_red().bold(),
                path,
                e
            );
            std::process::exit(1);
        }
    }
}

fn run_source(source: String, debug: bool) {
    let mut runtime = Runtime::new(RuntimeConfig { debug });

    // Compile the entire source at once - parser now handles multi-line properly
    let result = runtime.compiler.compile(source);

    match result {
        Ok(instructions) => {
            if debug {
                println!(
                    "{}\n{:#?}",
                    "instructions:".bright_blue().bold(),
                    instructions
                );
            }

            // Register all compiled methods in the VM's dispatch table
            for reg in &runtime.compiler.method_registrations {
                runtime.machine.register_method(
                    reg.method_name.clone(),
                    reg.pattern.clone(),
                    reg.address,
                );

                if debug {
                    println!(
                        "Registered method {} with pattern {:?} at address {}",
                        reg.method_name, reg.pattern, reg.address
                    );
                }
            }

            for instruction in instructions.clone() {
                runtime.machine.push_instruction(instruction);
            }

            if !instructions.is_empty() {
                match runtime.machine.execute_until_eof() {
                    Ok(_) => {
                        // Print result from ret register if present
                        if let Some(result) = runtime.machine.registers.get("ret") {
                            if debug
                                && !matches!(
                                    result,
                                    strontium::machine::register::RegisterValue::Empty
                                )
                            {
                                println!("{}", result);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("{} {:?}", "error:".bright_red().bold(), e);
                        match e {
                            StrontiumError::BytecodeError(BytecodeError::UnexpectedEof(_)) => {
                                eprintln!(
                                    "{} {:?}",
                                    "bytecode:".bright_blue().bold(),
                                    runtime.machine.registers.get("bc").unwrap()
                                );
                            }
                            _ => {}
                        }
                        std::process::exit(1);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("{} {}", "error:".bright_red().bold(), e);
            std::process::exit(1);
        }
    }
}
