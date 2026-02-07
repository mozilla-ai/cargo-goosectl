use anyhow::Result;
use clap::{Parser, Subcommand};
use std::collections::HashMap;

use crate::version::semantic_version::SemanticVersion;

mod commands;
mod global_args;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
#[command(name = "cargo-goosectl")]
#[command(bin_name = "cargo")]
pub enum CargoGooseCli {
    Goose(Cli),
}

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
    CurrentVersion(commands::CurrentVersionArgs),
}

impl Command {
    pub fn execute(
        &self,
        metadata: &crate::metadata::Metadata,
        global: &global_args::GlobalArgs,
    ) -> Result<()> {
        match self {
            Command::Bump(args) => self.bump(metadata, args, global),
            Command::CurrentVersion(args) => args.execute(metadata, global),
        }
    }

    fn bump(
        &self,
        metadata: &crate::metadata::Metadata,
        args: &commands::BumpArgs,
        global: &global_args::GlobalArgs,
    ) -> Result<()> {
        // Determine which packages are being directly bumped
        let packages = metadata.select_packages(global.workspace, global.package.as_slice())?;

        // Map of package name -> new version, used later for propagation
        let mut updated_packages = HashMap::new();

        let prefix = if global.dry_run { "[DRY RUN] " } else { "" };

        // Phase 1: apply the version transition to selected packages
        for package in &packages {
            let curr: SemanticVersion = package.version.clone().try_into()?;
            let transition = args.clone().into();
            let next = curr.apply(transition)?;

            // Write the new package version to Cargo.toml
            if !global.dry_run {
                let contents = std::fs::read_to_string(&package.manifest_path)?;
                let mut doc = contents.parse::<toml_edit::DocumentMut>()?;
                doc["package"]["version"] = next.to_string().into();
                std::fs::write(&package.manifest_path, doc.to_string())?;
            }

            println!(
                "{}Updated package {} from version {} to {}",
                prefix, package.name, curr, next
            );

            // Record updated versions for dependency propagation
            updated_packages.insert(package.name.to_string(), next.clone());
        }

        // Propagation is enabled only in workspace mode (explicit or implicit)
        // and can be disabled explicitly via --no-propagate
        let propagate = !args.no_propagate && (global.workspace || packages.len() > 1);

        if !propagate {
            return Ok(());
        }

        // Phase 2: propagate updated versions to all workspace dependents
        for package in metadata.all_packages()? {
            let contents = std::fs::read_to_string(&package.manifest_path)?;
            let mut doc = contents.parse::<toml_edit::DocumentMut>()?;
            let mut changed = false;

            // Inspect all dependency sections that Cargo understands
            for section in ["dependencies", "dev-dependencies", "build-dependencies"] {
                let Some(deps) = doc.get_mut(section).and_then(|v| v.as_table_mut()) else {
                    continue;
                };

                for (dep_name, dep_item) in deps.iter_mut() {
                    let dep_name_str = dep_name.get();

                    // Only consider dependencies whose package was bumped
                    let Some(new_version) = updated_packages.get(dep_name_str) else {
                        continue;
                    };

                    // Only rewrite workspace/path dependencies to avoid touching registry deps
                    let is_path_dep = dep_item.as_table().and_then(|t| t.get("path")).is_some();

                    if !is_path_dep {
                        continue;
                    }

                    // Mutate only the version field, preserving path, features, etc.
                    if let Some(table) = dep_item.as_table_mut() {
                        table["version"] = toml_edit::value(new_version.to_string());
                        changed = true;
                    }

                    println!(
                        "{}Updated dependency {} in package {} to {}",
                        prefix, dep_name, package.name, new_version
                    );
                }
            }

            // Write back the manifest only if something actually changed
            if changed && !global.dry_run {
                std::fs::write(&package.manifest_path, doc.to_string())?;
            }
        }

        Ok(())
    }
}
