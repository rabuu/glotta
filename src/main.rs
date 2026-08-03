use std::path::PathBuf;

use clap::{Parser, Subcommand};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

use glotta::driver::Driver;

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
            let driver = Driver::new(input).unwrap();
            let tokens = driver.lex();
            for token in tokens {
                match token {
                    Ok((token, span)) => println!("{token} (at {span:?})"),
                    Err(err) => println!("{err}"),
                }
            }
        }
        CliCommand::Parse { input } => {
            let _driver = Driver::new(input).unwrap();
            todo!()
        }
        CliCommand::Generate { input } => {
            let _driver = Driver::new(input).unwrap();
            todo!()
        }
        CliCommand::Build { input } => {
            let _driver = Driver::new(input).unwrap();
            todo!()
        }
    }
}
