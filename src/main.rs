mod config;
mod jid;
mod line;
mod markdown;

use clap::Parser;
use markdown::MdFormatter;
/// Manage a Johnny Decimal System of markdown files and directories
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[clap(version, about, author, long_about=None)]
struct Arguments {
    #[clap(env = "JDEX_CONFIG", short, long)]
    /// The Path to a toml file that defines the system
    config_file: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    color_eyre::install()?;
    let args: Arguments = Arguments::parse();
    let config = config::JohnnyDecimalConfig::from_file(&args.config_file)?;
    let system_config = config.system_config;
    let md_format = config.format;
    let system = config::System::try_from(system_config)?;
    let formatter: MdFormatter = md_format.try_into()?;
    println!("{}", formatter.system(&system)?);

    Ok(())
}
