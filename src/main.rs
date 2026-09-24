//! `heyreach` - a CLI over the HeyReach API, built to be driven by an LLM agent.

mod account;
mod cli;
mod client;
mod commands;
mod config;
mod error;
mod output;
mod readme;
mod secrets;

use std::process::ExitCode;

use clap::Parser;
use clap::error::ErrorKind;

use crate::error::Error;

fn main() -> ExitCode {
    // The error envelope can be rendered before parsing succeeds, so the format has to be known
    // from the raw args first.
    let args: Vec<String> = std::env::args().skip(1).take_while(|a| a != "--").collect();
    let has_json = args.iter().any(|a| a == "--json" || a == "--format=json")
        || args.windows(2).any(|w| w[0] == "--format" && w[1].eq_ignore_ascii_case("json"));
    output::set_json(has_json);

    let cli = match cli::Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) => return parse_error(e),
    };

    if cli.json || cli.format.as_deref().map(|f| f.eq_ignore_ascii_case("json")).unwrap_or(false) {
        output::set_json(true);
    }

    match commands::run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => ExitCode::from(output::write_error(&e) as u8),
    }
}

/// Help and version are successes; everything else clap refuses is an `invalid_input` envelope,
/// with clap's own explanation (and usage) as the detail.
fn parse_error(e: clap::Error) -> ExitCode {
    match e.kind() {
        ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => {
            let _ = e.print();
            ExitCode::SUCCESS
        }
        ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand => {
            let _ = e.print();
            ExitCode::from(error::ErrorCode::InvalidInput as u8)
        }
        _ => {
            let rendered = e.render().to_string();
            let (what, usage) = rendered.split_once("\n\n").unwrap_or((&rendered, ""));
            let message = what
                .lines()
                .map(str::trim)
                .filter(|l| !l.is_empty())
                .collect::<Vec<_>>()
                .join(" ")
                .trim_start_matches("error: ")
                .to_string();
            let mut err = Error::invalid(if message.is_empty() { "Invalid arguments.".into() } else { message });
            if !usage.trim().is_empty() {
                err = err.detail(usage.trim());
            }
            ExitCode::from(output::write_error(&err) as u8)
        }
    }
}
