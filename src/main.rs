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

    init_logger(cli.quiet);

    if let Err(err) = run(cli) {
        eprintln!("{err:?}");
        std::process::exit(1);
    }
}

fn run(cli: CliArgs) -> miette::Result<()> {
    match cli.cmd {
        CliCommand::Lex { input } => {
            let driver = Driver::new(input).map_err(miette::Report::from)?;
            let lexer = driver.lexer();
            for token in TokenStream::new(lexer, vec![]) {
                println!("{} (at {:?})", token.kind, token.span);
            }
            Ok(())
        }
        CliCommand::Parse { input } => {
            let driver = Driver::new(input).map_err(miette::Report::from)?;
            let ast = driver.parse().map_err(|err| driver.to_report(err))?;
            println!("{ast:#?}");
            Ok(())
        }
        CliCommand::Generate { input, output } => {
            let driver = Driver::new(input).map_err(miette::Report::from)?;

            match output {
                Some(path) => driver.emit_assembly_to_file(path),
                None => driver.emit_assembly_to_stdout(),
            }
            .map_err(|err| driver.to_report(err))?;

            Ok(())
        }
        CliCommand::Build { input } => {
            let _driver = Driver::new(input).map_err(miette::Report::from)?;
            todo!()
        }
    }
}

fn init_logger(quiet: bool) {
    let default_level_filter = match quiet {
        true => LevelFilter::OFF,
        false => LevelFilter::INFO,
    };

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_env_var("GLOTTA_LOG")
                .with_default_directive(default_level_filter.into())
                .from_env_lossy(),
        )
        .without_time()
        .compact()
        .init();
}
