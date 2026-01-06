use anyhow::Result;
use cargo_metadata::MetadataCommand;
use clap::Parser;

use crate::cli::CargoGooseCli;

mod cli;
mod metadata;
mod version;

fn main() -> Result<()> {
    // get cargo metadata
    let metadata = MetadataCommand::new().exec()?.into();

    // parse args
    let args = cli::CargoGooseCli::parse();

    match args {
        CargoGooseCli::Goose(args) => args.execute(&metadata)?,
    }

    Ok(())
}
