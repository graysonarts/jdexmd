/// Manage a Johnny Decimal System of markdown files and directories
mod config;
mod jid;
mod line;
mod markdown;
mod model;
mod notes;

use clap::Parser;
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
    let actions = notes::get_all_actions(&output_config, &system)
        .into_iter()
        .filter(|t| args.dry_run || notes::need_to_apply(t));
    for action in actions {
        if args.dry_run {
            println!("{}", action.dry_run());
        } else {
            action.execute(&formatter)?;
        }
    }
    // println!("{}", formatter.system(&system)?);

    Ok(())
}
