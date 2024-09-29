/// Manage a Johnny Decimal System of markdown files and directories
mod config;
mod jid;
mod line;
mod markdown;
mod model;
mod notes;

use clap::Parser;
use color_eyre::eyre::Error;
use markdown::MdFormatter;
use std::path::PathBuf;

use crate::model::System;

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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    color_eyre::install()?;
    let args: Arguments = Arguments::parse();
    let config = config::JohnnyDecimalConfig::from_file(&args.config_file)?;
    let output_config = config.output_config;
    let system_config = config.system_config;
    let md_format = config.format;
    let system = System::try_from(system_config)?;
    let formatter: MdFormatter = md_format.try_into()?;
    generate_notes(&output_config, &system, &args, &formatter)?;
    generate_archive(&output_config, &system, &args, &formatter)?;

    Ok(())
}

fn generate_notes(
    output_config: &config::OutputConfig,
    system: &System,
    args: &Arguments,
    formatter: &MdFormatter<'_>,
) -> Result<(), Error> {
    if args.dry_run {
        println!("Notes Folders");
    }

    let actions = notes::get_all_actions(&output_config.base_folder, &system)
        .into_iter()
        .filter(|t| args.dry_run || notes::need_to_apply(t));
    Ok(for action in actions {
        if args.dry_run {
            print!("{}", action.dry_run());
        } else {
            action.execute(&formatter)?;
        }
    })
}

fn generate_archive(
    output_config: &config::OutputConfig,
    system: &System,
    args: &Arguments,
    formatter: &MdFormatter<'_>,
) -> Result<(), Error> {
    if args.dry_run {
        println!("\nReference Archive")
    }
    let actions = notes::get_all_actions(&output_config.reference_folder, &system)
        .into_iter()
        .filter(|t| args.dry_run || notes::need_to_apply(t))
        .filter(|t| matches!(t, notes::Action::CreateDirectory(_)));
    Ok(for action in actions {
        if args.dry_run {
            print!("{}", action.dry_run());
        } else {
            action.execute(&formatter)?;
        }
    })
}
