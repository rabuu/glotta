use std::path::PathBuf;

use clap::{Parser, Subcommand};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

use glotta::driver::Driver;
use glotta::parser::token_stream::TokenStream;

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
            let lexer = driver.lexer();
            for token in TokenStream::new(lexer, false) {
                println!("{} (at {:?})", token.kind, token.span);
            }
        }
        CliCommand::Parse { input } => {
            let driver = Driver::new(input).unwrap();
            let ast = driver.parse();
            match ast {
                Ok(ast) => println!("{ast:#?}"),
                Err(err) => {
                    let err: miette::Error = err.into();
                    let source = driver.source().to_string();
                    let report: miette::Report = err.with_source_code(source);
                    eprintln!("{report:?}");
                    std::process::exit(1);
                }
            };
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
