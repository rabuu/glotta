use std::fmt;
use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};
use glotta::span::SourcePosition;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

use glotta::driver::Driver;
use glotta::token_stream::TokenStream;

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
    /// Compile a source file or project.
    #[clap(alias = "b")]
    Build {
        /// Path of input file.
        input: PathBuf,

        /// Path of output file.
        #[clap(short, long)]
        output: Option<PathBuf>,

        /// Output file format.
        #[clap(short, long, default_value_t)]
        format: CliOutputFormat,

        /// Keep all intermediate build artifacts.
        #[clap(short, long)]
        keep_build_artifacts: bool,
    },

    /// Debug a compiler intermediate stage.
    Debug {
        /// Path of input file.
        input: PathBuf,

        /// The intermediate stage to debug.
        #[clap(short, long)]
        stage: CliDebugStage,
    },
}

#[derive(Debug, Clone, Copy, Default, ValueEnum)]
pub enum CliOutputFormat {
    #[value(alias = "asm")]
    Assembly,

    #[value(alias = "obj")]
    Object,

    #[value(alias = "exe")]
    #[default]
    Executable,
}

impl fmt::Display for CliOutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliOutputFormat::Assembly => write!(f, "assembly"),
            CliOutputFormat::Object => write!(f, "object"),
            CliOutputFormat::Executable => write!(f, "executable"),
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum CliDebugStage {
    #[value(alias = "lex")]
    Lexing,

    #[value(alias = "parse")]
    Parsing,

    #[value(alias = "tac")]
    Tacky,

    #[value(alias = "asm")]
    Assembly,
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
        CliCommand::Build {
            input,
            output,
            format,
            keep_build_artifacts,
        } => {
            let driver = Driver::new(input).map_err(miette::Report::from)?;

            match format {
                CliOutputFormat::Assembly => driver.assembly_to_file(output),
                CliOutputFormat::Object => driver.object_to_file(output, keep_build_artifacts),
                CliOutputFormat::Executable => {
                    driver.executable_to_file(output, keep_build_artifacts)
                }
            }
            .map_err(|err| driver.to_report(err))?;

            Ok(())
        }
        CliCommand::Debug { stage: mode, input } => {
            let driver = Driver::new(input).map_err(miette::Report::from)?;
            match mode {
                CliDebugStage::Lexing => {
                    let lexer = driver.lexer();
                    for token in TokenStream::new(lexer, vec![]) {
                        fn display_source_position(pos: Option<SourcePosition>) -> String {
                            match pos {
                                Some(pos) => pos.to_string(),
                                None => String::from("out of bounds"),
                            }
                        }

                        let (start_pos, end_pos) = driver.span_to_source_positions(token.span);
                        println!(
                            "[{} to {}] {}",
                            display_source_position(start_pos),
                            display_source_position(end_pos),
                            token.kind,
                        );
                    }
                    Ok(())
                }
                CliDebugStage::Parsing => {
                    let ast = driver.parse().map_err(|err| driver.to_report(err))?;
                    println!("{ast:#?}");
                    Ok(())
                }
                CliDebugStage::Tacky => driver
                    .tacky_to_stdout()
                    .map_err(|err| driver.to_report(err)),
                CliDebugStage::Assembly => driver
                    .assembly_to_stdout()
                    .map_err(|err| driver.to_report(err)),
            }
        }
    }
}

fn init_logger(quiet: bool) {
    let default_level_filter = match quiet {
        true => LevelFilter::OFF,
        false => LevelFilter::INFO,
    };

    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
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
