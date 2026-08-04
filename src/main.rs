use std::path::PathBuf;

use clap::{Parser, Subcommand};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

use glotta::driver::Driver;
use glotta::parser::TokenStream;

#[derive(Debug, Parser)]
#[clap(version, about = None, long_about = None)]
struct CliArgs {
    #[clap(subcommand)]
    cmd: CliCommand,

    /// Disable logging.
    #[clap(short, long)]
    quiet: bool,
}

#[derive(Debug, Subcommand)]
enum CliCommand {
    /// Print the lexer output.
    Lex {
        /// Input source file.
        input: PathBuf,
    },

    /// Print the parser output.
    Parse {
        /// Input source file.
        input: PathBuf,
    },

    /// Generate assembly code.
    #[clap(alias = "gen")]
    Generate {
        /// Input source file.
        input: PathBuf,

        /// Output assembly file.
        #[clap(short, long)]
        output: Option<Option<PathBuf>>,
    },

    /// Compile and build.
    #[clap(alias = "b")]
    Build {
        /// Input source file.
        input: PathBuf,
    },
}

fn main() {
    miette::set_panic_hook();

    let cli = CliArgs::parse();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_env_var("GLOTTA_LOG")
                .with_default_directive(
                    if cli.quiet {
                        LevelFilter::OFF
                    } else {
                        LevelFilter::INFO
                    }
                    .into(),
                )
                .from_env_lossy(),
        )
        .without_time()
        .compact()
        .init();

    match cli.cmd {
        CliCommand::Lex { input } => {
            let driver = new_driver(input);
            let lexer = driver.lexer();
            for token in TokenStream::new(lexer, vec![]) {
                println!("{} (at {:?})", token.kind, token.span);
            }
        }
        CliCommand::Parse { input } => {
            let driver = new_driver(input);
            let ast = driver.parse();
            match ast {
                Ok(ast) => println!("{ast:#?}"),
                Err(error) => {
                    driver.print_error(error);
                    std::process::exit(1);
                }
            };
        }
        CliCommand::Generate { input, output } => {
            let driver = new_driver(input);
            let asm = driver.codegen();
            match asm {
                Ok(asm) => {
                    if let Some(output) = output {
                        driver
                            .emit_assembly_to_file(output)
                            .unwrap_or_else(|error| {
                                driver.print_error(error);
                                std::process::exit(1);
                            })
                    } else {
                        println!("{asm}")
                    }
                }
                Err(error) => {
                    driver.print_error(error);
                    std::process::exit(1);
                }
            };
        }
        CliCommand::Build { input } => {
            let _driver = new_driver(input);
            todo!()
        }
    }
}

fn new_driver(input: PathBuf) -> Driver {
    match Driver::new(input) {
        Ok(driver) => driver,
        Err(error) => {
            let error: miette::Error = error.into();
            eprintln!("{error:?}");
            std::process::exit(1);
        }
    }
}
