use anyhow::Result;
use cargo_metadata::MetadataCommand;
use clap::Parser;

use cargo_goosectl::cli::CargoGooseCli;

fn main() -> Result<()> {
    // TODO: complete when we're ready to support configs
    // let config: Config = figment::Figment::new()
    //     .merge(figment::providers::Toml::file("goose.toml"))
    //     .extract()?;

    // get cargo metadata
    let metadata = MetadataCommand::new().exec()?.into();

    // parse args
    let args = CargoGooseCli::parse();

    match args {
        CargoGooseCli::Goose(args) => args.execute(&metadata)?,
    }

    Ok(())
}
