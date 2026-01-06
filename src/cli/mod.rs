use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::version::version::SemanticVersion;

mod commands;
mod global_args;

#[derive(Debug, Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Command,

    #[command(flatten)]
    pub global: global_args::GlobalArgs,
}

impl Cli {
    pub fn execute(&self, metadata: &crate::metadata::Metadata) -> Result<()> {
        self.cmd.execute(metadata, &self.global)
    }
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Bump(commands::BumpArgs),
}

impl Command {
    pub fn execute(
        &self,
        metadata: &crate::metadata::Metadata,
        global: &global_args::GlobalArgs,
    ) -> Result<()> {
        match self {
            Command::Bump(args) => self.bump(metadata, args, global),
        }
    }
    fn bump(
        &self,
        metadata: &crate::metadata::Metadata,
        args: &commands::BumpArgs,
        global: &global_args::GlobalArgs,
    ) -> Result<()> {
        let packages = metadata.select_packages(global.workspace, global.package.as_slice())?;

        for package in packages {
            let curr: SemanticVersion = package.version.clone().try_into()?;
            let transition = args.clone().into();
            let next = curr.apply(transition)?;

            println!(
                "Updated package {} from version {} to {}",
                package.name, curr, next
            )
        }
        Ok(())
    }
}
