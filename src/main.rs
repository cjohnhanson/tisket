use std::process::ExitCode;

use clap::Parser;

fn main() -> ExitCode {
    let args = tisket::cli::Args::parse();

    match tisket::cli::run(args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::from(1)
        }
    }
}
