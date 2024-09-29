#![deny(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(
    clippy::single_call_fn,
    clippy::implicit_return,
    clippy::question_mark_used,
    clippy::missing_trait_methods,
    clippy::string_slice,
    clippy::pattern_type_mismatch, // This one is giving too many false positives
    clippy::let_underscore_must_use, // This goes against other clippy lints
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "These are ones I don't want to enforce for this project"
)]

/// The configuration for the Johnny Decimal system
mod config;
/// The Johnny Decimal Identifier
mod jid;
/// The line parser for the system configuration
mod line;
/// The markdown formatter for the system
mod markdown;
/// The model for the Johnny Decimal system
mod model;
/// Everything needed for generating the system for a notetaking system
mod notes;

use clap::Parser;
use color_eyre::eyre::Error;
use markdown::MdFormatter;
use std::path::PathBuf;

use crate::model::System;

/// Command line arguments for running the process to generate the Johnny Decimal system
#[derive(Debug, Parser)]
#[clap(version, about, author, long_about=None)]
struct Arguments {
    #[clap(short, long, default_value = "false")]
    /// Preview what actions will be taken
    dry_run: bool,
    #[clap(env = "JDEX_CONFIG", short, long)]
    /// The Path to a toml file that defines the system
    config_file: PathBuf,
}

/// Manage a Johnny Decimal System of markdown files and directories
fn main() -> Result<(), Error> {
    color_eyre::install()?;
    let args: Arguments = Arguments::parse();
    let config = config::JohnnyDecimal::from_file(&args.config_file)?;
    let output_config = config.output_config;
    let system_config = config.system_config;
    let md_format = config.format;
    let system = System::try_from(system_config)?;
    let formatter: MdFormatter = md_format.try_into()?;
    generate_notes(&output_config, &system, &args, &formatter)?;
    generate_archive(&output_config, &system, &args, &formatter)?;

    Ok(())
}

/// Generate the Johnny Decimal notes folder structure for a markdown based note taking system like
/// logseq or obsidian.
fn generate_notes(
    output_config: &config::Output,
    system: &System,
    args: &Arguments,
    formatter: &MdFormatter<'_>,
) -> Result<(), Error> {
    if args.dry_run {
        println!("Notes Folders");
    }

    let actions = notes::get_all_actions(&output_config.base_folder, system)
        .into_iter()
        .filter(|action| args.dry_run || notes::need_to_apply(action));
    for action in actions {
        if args.dry_run {
            print!("{}", action.dry_run());
        } else {
            action.execute(formatter)?;
        }
    }
    Ok(())
}

/// Generate the reference archive folder structure.
fn generate_archive(
    output_config: &config::Output,
    system: &System,
    args: &Arguments,
    formatter: &MdFormatter<'_>,
) -> Result<(), Error> {
    if args.dry_run {
        println!("\nReference Archive");
    }
    let actions = notes::get_all_actions(&output_config.reference_folder, system)
        .into_iter()
        .filter(|action| args.dry_run || notes::need_to_apply(action))
        .filter(|action| matches!(action, &notes::Action::CreateDirectory(_)));
    for action in actions {
        if args.dry_run {
            print!("{}", action.dry_run());
        } else {
            action.execute(formatter)?;
        }
    }
    Ok(())
}
