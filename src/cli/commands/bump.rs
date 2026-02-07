use clap::{Args, Subcommand, ValueEnum};

use crate::version::transition::TransitionInput;

#[derive(Debug, Clone, Args)]
pub struct BumpArgs {
    #[command(subcommand)]
    target: VersionBump,
    #[arg(
        long,
        help = "Do not update workspace dependency versions when bumping a package",
        default_value = "false"
    )]
    pub no_propagate: bool,
}

#[derive(Debug, Clone, Subcommand)]
pub enum VersionBump {
    Prerelease {
        #[arg(
            help = "Increment the current prerelease counter, or transition to a new \
                    prerelease identifier (e.g. `alpha`, `beta`, `rc`).",
            value_name = "PRERELEASE"
        )]
        pre: Option<String>,
        #[arg(long, help = "Build metadata")]
        metadata: Option<String>,
    },
    Release {
        #[arg(long, help = "Build metadata")]
        metadata: Option<String>,
    },
    Version {
        level: ReleaseLevel,
        #[arg(
            help = "Start a prerelease on the new version line using the given identifier \
                    (e.g. `alpha`, `beta`, `rc`).",
            value_name = "PRERELEASE"
        )]
        pre: Option<String>,
        #[arg(long, help = "Build metadata")]
        metadata: Option<String>,
    },
}

impl From<BumpArgs> for TransitionInput {
    fn from(args: BumpArgs) -> TransitionInput {
        match args.target {
            VersionBump::Prerelease { pre, metadata } => {
                match pre {
                    // graduate pre-release to another pre-release (e.g., alpha -> beta)
                    Some(pre) => TransitionInput::TransitionPrerelease { pre, metadata },
                    // increment prerelease (e.g., alpha.1 -> alpha.2)
                    None => TransitionInput::IncrementPrerelease { metadata },
                }
            }
            VersionBump::Release { metadata } => {
                // graduate pre-release to release
                TransitionInput::FinalizeRelease { metadata }
            }
            VersionBump::Version {
                level,
                pre,
                metadata,
            } => match pre {
                Some(pre) => TransitionInput::StartPrerelease {
                    level: level.into(),
                    pre,
                    metadata,
                },
                None => TransitionInput::BumpRelease {
                    level: level.into(),
                    metadata,
                },
            },
        }
    }
}

#[derive(Debug, Clone, ValueEnum)]
pub enum ReleaseLevel {
    Patch,
    Minor,
    Major,
}

impl From<ReleaseLevel> for crate::version::semantic_version::ReleaseLevel {
    fn from(val: ReleaseLevel) -> Self {
        match val {
            ReleaseLevel::Patch => Self::Patch,
            ReleaseLevel::Minor => Self::Minor,
            ReleaseLevel::Major => Self::Major,
        }
    }
}
